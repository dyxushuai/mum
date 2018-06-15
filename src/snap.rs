use bincode::{deserialize_from, serialize_into};
use crc::crc32;
use errors::*;
use protobuf::Message;
use raft::eraftpb::Snapshot as RaftSnapshot;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use util::read_with_ext_and_sort;

const SNAP_EXT: &'static str = "snap";
const SNAP_BROKEN_EXT: &'static str = "broken";

pub struct Snapshotter {
    dir: PathBuf,
}

impl Snapshotter {
    pub fn new<P: AsRef<Path>>(p: P) -> Snapshotter {
        Snapshotter {
            dir: p.as_ref().to_path_buf(),
        }
    }

    pub fn save(&self, snapshot: &RaftSnapshot) -> Result<()> {
        if snapshot.metadata.is_none() {
            debug!("snapshot with empty metadata");
            return Ok(());
        }
        let md = snapshot.metadata.get_ref();
        let spath = self.new_snap_path(md.index, md.term);
        let b = snapshot.write_to_bytes()?;
        let crc = crc32::update(0, &crc32::CASTAGNOLI_TABLE, &b);
        match serialize_into(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&spath)?,
            &(crc, b),
        ) {
            Err(err) => {
                warn!(
                    "failed to write a snap file {} error {}",
                    spath.to_string_lossy(),
                    err
                );
                match fs::remove_file(spath.as_path()) {
                    Err(err) => {
                        warn!(
                            "failed to remove a broken snap file {} error {}",
                            spath.to_string_lossy(),
                            err
                        );
                    }
                    _ => (),
                }
                Err(err.into())
            }
            Ok(v) => Ok(v),
        }
    }

    pub fn load(&self) -> Result<Option<RaftSnapshot>> {
        for spath in read_with_ext_and_sort(&self.dir, SNAP_EXT, true)? {
            match self.load_snap(&spath) {
                Ok(v) => return Ok(Some(v)),
                Err(err) => {
                    warn!(
                        "failed to load snap from {} error {}",
                        &spath.to_string_lossy(),
                        err
                    );
                    self.broken(&spath);
                }
            }
        }
        Ok(None)
    }

    fn load_snap(&self, p: &PathBuf) -> Result<RaftSnapshot> {
        debug!("load snapshot form path {}", p.to_string_lossy());
        let snapshot: (u32, Vec<u8>) = deserialize_from(OpenOptions::new().read(true).open(p)?)?;
        if snapshot.0 != crc32::update(0, &crc32::CASTAGNOLI_TABLE, &snapshot.1) {
            return Err(Error::CrcMissMatch);
        }
        let mut raft_snapshot = RaftSnapshot::new();
        raft_snapshot.merge_from_bytes(&snapshot.1)?;
        return Ok(raft_snapshot);
    }

    fn broken(&self, p: &PathBuf) {
        let broken_name = Snapshotter::snap_broken_path(p);
        match fs::rename(p, &broken_name) {
            Err(err) => {
                warn!(
                    "failed to rename {} to a broken file {}  error {}",
                    p.to_string_lossy(),
                    &broken_name.to_string_lossy(),
                    err
                );
            }
            _ => (),
        }
    }

    fn snap_broken_path(snap_path: &PathBuf) -> PathBuf {
        snap_path.join(format!(".{}", SNAP_BROKEN_EXT))
    }

    fn new_snap_path(&self, index: u64, term: u64) -> PathBuf {
        self.dir
            .join(format!("{:016x}-{:016x}.{}", term, index, SNAP_EXT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raft::eraftpb::SnapshotMetadata;
    fn new_snapshot(index: u64, term: u64) -> RaftSnapshot {
        let mut snap = RaftSnapshot::new();
        let mut meta = SnapshotMetadata::new();
        meta.set_index(index);
        meta.set_term(term);
        snap.set_metadata(meta);
        snap
    }
    #[test]
    fn snapshot_save_test() {
        let index = 1;
        let term = 1;
        let snap_dir = "./file_tests/snap_save";
        let snap_shotter = Snapshotter::new(snap_dir);
        let snap_path = snap_shotter.new_snap_path(index, term);
        let snap = new_snapshot(index, term);
        snap_shotter.save(&snap).unwrap();
        assert!(snap_path.exists());
        fs::remove_file(snap_path).unwrap();
    }

    #[test]
    fn snapshot_load_test() {
        let index = 1;
        let term = 1;
        let snap_dir = "./file_tests/snap_load";
        let snap_shotter = Snapshotter::new(snap_dir);
        let snap_path = snap_shotter.new_snap_path(index, term);
        let mut snap = new_snapshot(index, term);
        let data = vec![1u8, 2u8, 3u8];
        snap.set_data(data.clone());
        snap_shotter.save(&snap).unwrap();
        assert!(snap_path.exists());
        let loaded = snap_shotter.load().unwrap().unwrap();
        assert_eq!(loaded.data, data);
        fs::remove_file(snap_path).unwrap();
    }
}
