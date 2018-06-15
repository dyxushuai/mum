use std::time::{Duration, Instant};

use errors::*;
use futures::sync::mpsc::UnboundedReceiver;
use futures::Stream;
use grpcio::Environment;
use kv::Store;
use proto::mumpb::*;
use protobuf::Message;
use raft::is_empty_snap;
use raft::prelude::*;
use raft::storage::MemStorage;
use snap::Snapshotter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::run;
use tokio::timer::Interval;
use transport::RaftClients;
use util::create_dir;
use wal::{wal_exists, Wal};

const SNAPSHOT_TRIG_COUNT: u64 = 1;
const SNAPSHOT_CATCH_UP_ENTRIES_N: u64 = 1;

pub struct Node {
    id: u64,

    applied_index: u64,
    snapshot_index: u64,
    conf_state: Option<ConfState>,

    wal_dir: PathBuf,
    raft_node: RawNode<MemStorage>,
    snapshotter: Snapshotter,
    kv_store: Store,
    wal: Option<Wal>,
    last_index: u64,
    rx: Option<UnboundedReceiver<(Option<RaftMessage>, Option<OpRequest>, Option<ConfRequest>)>>,
    raft_clients: RaftClients,
}

impl Node {
    pub fn new<P: AsRef<Path>>(
        id: u64,
        env: Arc<Environment>,
        addrs: Vec<String>,
        kv_store: Store,
        wal_dir: P,
        snap_dir: P,
        rx: UnboundedReceiver<(Option<RaftMessage>, Option<OpRequest>, Option<ConfRequest>)>,
    ) -> Result<Node> {
        let mut cfg = Config::default();
        cfg.id = id;
        cfg.heartbeat_tick = 10;
        cfg.election_tick = cfg.heartbeat_tick * 10;
        cfg.max_inflight_msgs = 10;
        cfg.validate()?;
        create_dir(&wal_dir)?;
        create_dir(&snap_dir)?;
        // peers
        let mut raft_clients = RaftClients::new(env);
        let peers: Vec<Peer> = addrs
            .iter()
            .enumerate()
            .map(|(idx, addr)| {
                let p = Peer {
                    id: (idx + 1) as u64,
                    context: None,
                };
                raft_clients.upsert_peer(p.id, addr);
                p
            })
            .collect();
        // snapshot
        let snapshotter = Snapshotter::new(&snap_dir);
        let raft_node = RawNode::new(&cfg, MemStorage::default(), peers)?;

        //
        let mut n = Node {
            id: id,
            applied_index: 0,
            snapshot_index: 0,
            conf_state: None,
            wal_dir: wal_dir.as_ref().to_path_buf(),
            raft_node: raft_node,
            raft_clients: raft_clients,
            snapshotter: snapshotter,
            kv_store: kv_store,
            wal: None,
            last_index: 0,
            rx: Some(rx),
        };
        n.reply_wal();
        Ok(n)
    }

    pub fn run(mut self) {
        let rx = self.rx.take().unwrap();
        let t_rx = Interval::new(Instant::now(), Duration::from_millis(100));
        let mut conf_id = 0;

        let f = rx
            .map(|x| (Some(x), None))
            .map_err(|_| ())
            .select(t_rx.map(|_| (None, Some(()))).map_err(|_| ()))
            .for_each(move |x| {
                match x {
                    (Some(kv), None) => match kv {
                        (Some(mut raft), None, None) => {
                            if let Err(e) = self.raft_node.step(raft.take_message()) {
                                error!("raft step error {}", e);
                            }
                        }
                        (None, Some(op), None) => {
                            if let Err(e) =
                                self.raft_node.propose(vec![], op.write_to_bytes().unwrap())
                            {
                                error!("raft propose error {}", e);
                            }
                        }
                        (None, None, Some(mut conf)) => {
                            let mut change = conf.take_change();
                            conf_id += 1;
                            change.set_id(conf_id);
                            if let Err(e) = self.raft_node.propose_conf_change(vec![], change) {
                                error!("raft propose conf change error {}", e);
                            }
                        }
                        _ => (),
                    },
                    (None, Some(())) => {
                        self.raft_node.tick();
                    }
                    _ => (),
                }

                if self.raft_node.has_ready() {
                    self.on_ready();
                }
                Ok(())
            });
        run(f);
    }

    fn save_snap(&mut self, snap: &Snapshot) -> Result<()> {
        self.snapshotter.save(&snap)?;
        self.wal
            .as_mut()
            .unwrap()
            .release_lock_to(snap.get_metadata().get_index())
    }

    fn ents_to_apply(&self, ents: &mut Vec<Entry>) {
        if ents.len() == 0 {
            return;
        }
        let first_idx = ents.first().unwrap().get_index();
        if first_idx > self.applied_index + 1 {
            panic!(
                "first index of committed entry [{}] should <= self.applied_index[{}]+1",
                first_idx, self.applied_index
            );
        }
        let new_idx = (self.applied_index + 1 - first_idx) as usize;
        if new_idx < ents.len() {
            ents.drain(..new_idx);
        }
    }

    fn publish_entries(&mut self, ents: &Vec<Entry>) -> Result<()> {
        for entry in ents {
            match entry.get_entry_type() {
                EntryType::EntryNormal => {
                    let mut op = OpRequest::new();
                    op.merge_from_bytes(entry.get_data()).unwrap();
                    match op.field_type {
                        Op::Set => {
                            self.kv_store.set(op.key, op.value);
                        }
                        Op::Del => {
                            self.kv_store.delete(&op.key);
                        }
                        _ => (),
                    }
                }
                EntryType::EntryConfChange => {
                    let mut change = ConfChange::new();
                    change.merge_from_bytes(entry.get_data())?;
                    self.raft_node.apply_conf_change(&change);
                    match change.get_change_type() {
                        ConfChangeType::AddNode => {
                            if change.get_context().len() > 0 {
                                self.raft_clients.upsert_peer(
                                    change.get_node_id(),
                                    ::std::str::from_utf8(change.get_context())?,
                                );
                            }
                        }
                        ConfChangeType::RemoveNode => {
                            if change.get_node_id() == self.id {
                                ::std::process::abort();
                            }
                            self.raft_clients.delete_peer(change.get_node_id());
                        }
                        ConfChangeType::AddLearnerNode => (),
                    }
                }
            }
            self.applied_index = entry.get_index();
        }
        Ok(())
    }

    fn load_snapshot(&mut self) -> Option<Snapshot> {
        match self.snapshotter.load() {
            Ok(v) => v,
            Err(err) => {
                panic!("error loading snapshot {}", err);
            }
        }
    }

    fn open_wal(&mut self, snap: &Option<Snapshot>) {
        let wal = if wal_exists(&self.wal_dir) && snap.is_some() {
            let md = snap.as_ref().unwrap().get_metadata();
            Wal::open_at(&self.wal_dir, (md.term, md.index))
        } else {
            Wal::create(&self.wal_dir)
        };
        self.wal = Some(wal.expect("wal dir opened failed"));
    }

    fn reply_wal(&mut self) {
        info!("replayint wal of member {}", self.id);
        let snap = self.load_snapshot();
        self.open_wal(&snap);
        let (hs, ents) = self
            .wal
            .as_mut()
            .unwrap()
            .read_all()
            .expect("failed to read wal");
        if snap.is_some() {
            self.raft_node
                .mut_store()
                .wl()
                .apply_snapshot(snap.as_ref().unwrap().clone())
                .unwrap();
            debug!("get snapshot");
            self.kv_store.from_snapshot(snap.as_ref().unwrap()).unwrap();
        }
        self.raft_node.mut_store().wl().set_hardstate(hs);
        self.raft_node.mut_store().wl().append(&ents).unwrap();
        if ents.len() > 0 {
            self.last_index = ents.last().unwrap().get_index();
        }

        let snap = self.raft_node.get_store().snapshot().unwrap();
        self.conf_state = Some(snap.get_metadata().get_conf_state().clone());
        self.snapshot_index = snap.get_metadata().get_index();
        self.applied_index = snap.get_metadata().get_index();
    }

    fn publish_snapshot(&mut self, snap: &Snapshot) {
        let idx = snap.get_metadata().get_index();
        info!("publishing snapshot at index {}", self.snapshot_index);
        if idx <= self.applied_index {
            panic!(
                "snapshot index [{}] should > self.applied_index [{}] + 1",
                snap.get_metadata().get_index(),
                self.snapshot_index
            );
        }
        self.kv_store.from_snapshot(snap).unwrap();
        self.conf_state = Some(snap.get_metadata().get_conf_state().clone());
        self.snapshot_index = idx;
        self.applied_index = idx;
        info!(
            "finished publishing snapshot at index {}",
            self.snapshot_index
        );
    }

    fn maybe_trigger_snapshot(&mut self) {
        if self.applied_index - self.snapshot_index <= SNAPSHOT_TRIG_COUNT {
            return;
        }
        info!(
            "start snapshot [applied index: {} | last snpshot index: {}]",
            self.applied_index, self.snapshot_index
        );
        let data = self.kv_store.get_snapshot().unwrap();
        debug!("snapshot data {:?}", data);
        let snap = {
            self.raft_node
                .mut_store()
                .wl()
                .create_snapshot(self.applied_index, self.conf_state.clone(), data)
                .unwrap()
                .clone()
        };

        let compact_index = if self.applied_index > SNAPSHOT_CATCH_UP_ENTRIES_N {
            self.applied_index - SNAPSHOT_CATCH_UP_ENTRIES_N
        } else {
            1
        };
        {
            self.raft_node
                .mut_store()
                .wl()
                .compact(compact_index)
                .unwrap();
        }
        self.save_snap(&snap).unwrap();
        info!("compacted log at index {}", compact_index);
        self.snapshot_index = self.applied_index;
    }

    fn on_ready(&mut self) {
        let mut ready = self.raft_node.ready();
        // save to wal
        self.wal
            .as_mut()
            .unwrap()
            .insert(ready.hs.clone(), &ready.entries, ready.must_sync)
            .unwrap();
        // Append entries to the Raft log
        self.raft_node
            .mut_store()
            .wl()
            .append(&ready.entries)
            .unwrap();
        if let Some(ref hs) = ready.hs {
            // Raft HardState changed, and we need to persist it.
            self.raft_node.mut_store().wl().set_hardstate(hs.clone());
        }

        // handle snapshot
        if !is_empty_snap(&ready.snapshot) {
            self.save_snap(&ready.snapshot).unwrap();
            self.raft_node
                .mut_store()
                .wl()
                .apply_snapshot(ready.snapshot.clone())
                .unwrap();
            self.publish_snapshot(&ready.snapshot);
        }

        {
            // send msgs
            let msgs = ready.messages.drain(..);
            for msg in msgs {
                let mut r_msg = RaftMessage::new();
                r_msg.set_message(msg);
                self.raft_clients.send(r_msg).unwrap();
            }
            //self.raft_clients.flush();
        }

        // handle commited entries
        if let Some(mut committed_entries) = ready.committed_entries.take() {
            self.ents_to_apply(&mut committed_entries);
            self.publish_entries(&committed_entries).unwrap();
        }

        self.maybe_trigger_snapshot();
        // Advance the Raft
        self.raft_node.advance(ready);
    }
}
