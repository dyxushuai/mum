use std::sync::Arc;

use futures::Future;
use grpcio::{ChannelBuilder, Environment};
use proto::mumpb::RaftMessage;
use proto::mumpb_grpc::MumClient;
use std::collections::HashMap;

use errors::*;
// use metrics::*;

pub struct RaftClients {
    env: Arc<Environment>,
    addrs: HashMap<u64, String>,
    conns: HashMap<String, MumClient>,
}

impl RaftClients {
    pub fn new(env: Arc<Environment>) -> RaftClients {
        RaftClients {
            env,
            addrs: HashMap::default(),
            conns: HashMap::default(),
        }
    }

    pub fn upsert_peer(&mut self, store_id: u64, addr: &str) {
        if addr.is_empty() {
            error!("empty addr for peer id: {}", store_id);
        }
        self.addrs.insert(store_id, addr.to_owned());
    }

    pub fn delete_peer(&mut self, store_id: u64) {
        self.addrs.remove(&store_id);
    }

    fn get_conn(&mut self, msg: &RaftMessage) -> Option<(&mut MumClient, &str)> {
        let key = msg.get_message().get_to();
        let env = &self.env;
        match self.addrs.get(&key) {
            Some(v) => Some((
                self.conns.entry(v.to_owned()).or_insert_with(|| {
                    let channel = ChannelBuilder::new(env.clone()).connect(v);
                    let client = MumClient::new(channel);
                    client
                }),
                v,
            )),
            None => None,
        }
    }

    pub fn send(&mut self, msg: RaftMessage) -> Result<()> {
        match self.get_conn(&msg) {
            Some((client, addr)) => match client.raft_async(&msg) {
                Ok(r) => {
                    let addr = addr.to_owned();
                    client.spawn(
                        r.map_err(move |e| {
                            error!("recevier from remote {} error {}", addr, e);
                        }).map(|_| ()),
                    );
                }
                Err(e) => {
                    error!("send message to {} error {}", addr.to_owned(), e);
                }
            },
            None => warn!("miss connections {}", msg.get_message().get_to()),
        }

        Ok(())
    }
}
