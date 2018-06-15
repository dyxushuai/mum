use bincode::{deserialize, serialize};
use errors::*;
use indexmap::IndexMap;
use raft::eraftpb::Snapshot;
use std::sync::Arc;
use std::sync::RwLock;
use util::HandyRwLock;

/// Simple implement of KV Storage by IndexMap
#[derive(Clone)]
pub struct Store {
    inner: Arc<RwLock<IndexMap<Vec<u8>, Vec<u8>>>>,
}

impl Store {
    fn from_map(m: IndexMap<Vec<u8>, Vec<u8>>) -> Store {
        Store {
            inner: Arc::new(RwLock::new(m)),
        }
    }

    pub fn new() -> Store {
        Store::from_map(IndexMap::new())
    }

    pub fn from_snapshot(&mut self, snap: &Snapshot) -> Result<()> {
        debug!("snap data {:?}", snap.get_data());
        let idx_map = deserialize(snap.get_data())?;
        debug!("{:?}", idx_map);
        *self.inner.wl() = idx_map;
        Ok(())
    }

    pub fn get_snapshot(&self) -> Result<Vec<u8>> {
        serialize(&*self.inner.rl()).map_err(|err| err.into())
    }

    pub fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> Option<Vec<u8>> {
        self.inner.wl().insert(k, v)
    }

    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        self.inner.rl().get(k).map(|v| v.clone())
    }

    pub fn delete(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.inner.wl().remove(k)
    }

    pub fn scan(&self, start_key: &[u8], limit: u32) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.inner
            .rl()
            .iter()
            .skip_while(|(k, _)| {
                if k.as_slice() == start_key {
                    return false;
                }
                true
            })
            .take(limit as usize)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_delete_test() {
        let k = vec![1u8, 2u8, 3u8];
        let v = vec![4u8, 5u8, 6u8];
        let mut kv = Store::new();
        kv.set(k.clone(), v.clone());
        assert_eq!(kv.get(&k), Some(v.clone()));
        assert_eq!(kv.delete(&k), Some(v.clone()));
        assert_eq!(kv.get(&k), None);
    }

    #[test]
    fn scan_test() {
        let k1 = vec![1u8, 2u8, 3u8];
        let v1 = vec![4u8, 5u8, 6u8];
        let k2 = vec![11u8, 12u8, 13u8];
        let v2 = vec![4u8, 5u8, 6u8];
        let k3 = vec![21u8, 22u8, 23u8];
        let v3 = vec![4u8, 5u8, 6u8];

        let mut kv = Store::new();
        kv.set(k1.clone(), v1.clone());
        kv.set(k2.clone(), v2.clone());
        kv.set(k3.clone(), v3.clone());
        assert_eq!(kv.scan(&k2, 3), vec![(k2, v2), (k3, v3)])
    }
}
