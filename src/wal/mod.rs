mod record;
mod wal_file;

use self::record::*;
use self::wal_file::WalFile;
use bincode::deserialize;
use crc::crc32;
use errors::*;
use protobuf::Message;
use raft::eraftpb::{Entry, HardState};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use util::read_with_ext_and_sort;

const WAL_EXT: &'static str = "wal";
// const BUFFER_LEN: usize = 4 * 1000 * 1000;

// file.seq & raft.index
type Index = (u64, u64);

// raft.term & raft.index
type RaftIndex = (u64, u64);

pub struct Wal {
    dir: PathBuf,
    state: HardState,
    last_wal_index: Index,
    start: RaftIndex,
    segments: Vec<WalFile>,
    enti: u64,
}

impl Wal {
    pub fn create<P: AsRef<Path>>(dir: P) -> Result<Wal> {
        // create first wal file
        let first_wal_path = new_wal_path(&dir, (0, 0));

        Ok(Wal {
            dir: dir.as_ref().to_path_buf(),
            state: HardState::new(),
            last_wal_index: (0, 0),
            start: (0, 0),
            segments: vec![WalFile::create(&first_wal_path)?],
            enti: 0,
        })
    }

    pub fn insert(
        &mut self,
        state: Option<HardState>,
        entries: &[Entry],
        must_sync: bool,
    ) -> Result<()> {
        if state.is_none() && entries.len() == 0 {
            return Ok(());
        }
        for entry in entries {
            self.insert_entry(entry)?;
        }

        self.insert_state(&state)?;
        match self.newest_mut().check_cut()? {
            false => {
                if must_sync {
                    self.sync()?;
                }
            }
            true => self.cut()?,
        }
        Ok(())
    }

    pub fn release_lock_to(&mut self, index: u64) -> Result<()> {
        split_at_index(&mut self.segments, index);
        Ok(())
    }

    pub fn open_at<P: AsRef<Path>>(dir: P, raft_index: RaftIndex) -> Result<Wal> {
        if !dir.as_ref().exists() {
            return Err(Error::MissFilePath(dir.as_ref().to_string_lossy().into()));
        }
        // sort by name
        let mut wpaths = read_with_ext_and_sort(&dir, WAL_EXT, false)?;
        split_at_index(&mut wpaths, raft_index.1);
        valid_seq(&wpaths);
        let last_wal_index = index_from_path(wpaths.last().unwrap());

        let mut segments = vec![];
        for wpath in wpaths {
            segments.push(WalFile::open(&wpath)?);
        }

        Ok(Wal {
            dir: dir.as_ref().to_path_buf(),
            state: HardState::new(),
            start: raft_index,
            last_wal_index: last_wal_index,
            segments: segments,
            enti: 0,
        })
    }

    // todo iterator
    pub fn read_all(&mut self) -> Result<(HardState, Vec<Entry>)> {
        let mut ents = vec![];
        let mut state = HardState::new();
        for segment in &mut self.segments {
            for record in segment.into_iter() {
                match record.ty {
                    RecordType::EntryType => {
                        let mut entry = Entry::new();
                        entry.merge_from_bytes(&record.data).unwrap();
                        self.enti = entry.index;
                        if entry.index > self.start.1 {
                            ents.push(entry);
                        }
                    }
                    RecordType::StateType => {
                        state.merge_from_bytes(&record.data).unwrap();
                    }
                    RecordType::CrcType => {
                        if record.crc != crc32::update(0, &crc32::CASTAGNOLI_TABLE, &record.data) {
                            return Err(Error::CrcMissMatch);
                        }
                    }
                    RecordType::IndexType => {
                        let idx: Index = deserialize(&record.data).unwrap();
                        if idx.1 == self.start.1 {
                            if idx.0 != self.start.0 {
                                return Err(Error::SnapMissMatch);
                            }
                        }
                    }
                }
            }
        }
        Ok((state, ents))
    }

    fn insert_state(&mut self, state: &Option<HardState>) -> Result<()> {
        if state.is_none() {
            return Ok(());
        }
        self.state = state.as_ref().unwrap().clone();
        let record = Record::new(RecordType::StateType, self.state.write_to_bytes()?);
        self.newest_mut().insert_record(record)?;
        Ok(())
    }

    fn insert_entry(&mut self, entry: &Entry) -> Result<()> {
        let record = Record::new(RecordType::EntryType, entry.write_to_bytes()?);
        self.newest_mut().insert_record(record)?;
        self.enti = entry.index;
        Ok(())
    }

    #[allow(dead_code)]
    #[inline]
    fn newest(&self) -> &WalFile {
        self.segments.last().unwrap()
    }

    #[inline]
    fn newest_mut(&mut self) -> &mut WalFile {
        self.segments.last_mut().unwrap()
    }

    fn sync(&mut self) -> Result<()> {
        self.newest_mut().sync()
    }

    fn cut(&mut self) -> Result<()> {
        self.sync()?;
        let wpath = new_wal_path(
            &self.dir,
            (self.last_wal_index.0 + 1, self.last_wal_index.1 + 1),
        );
        self.segments.push(WalFile::create(&wpath)?);
        Ok(())
    }
}

fn split_at_index<P: AsRef<Path>>(paths: &mut Vec<P>, index: u64) {
    if paths.len() == 0 {
        panic!("empty wal file paths")
    }
    paths.reverse();
    let position = paths
        .iter()
        .position(|path| {
            let (_, cur_idx) = index_from_path(&path);
            if index >= cur_idx {
                return true;
            }
            false
        })
        .expect(&format!("index {} less than any of wal files", index));
    paths.drain(position + 1..);
    paths.reverse();
}

pub fn wal_exists<P: AsRef<Path>>(path: P) -> bool {
    read_with_ext_and_sort(&path, WAL_EXT, false).unwrap().len() > 0
}

fn valid_seq<P: AsRef<Path>>(paths: &Vec<P>) {
    let mut last_seq = 0;
    for path in paths {
        let (cur_seq, _) = index_from_path(path);
        if last_seq != 0 && last_seq != cur_seq - 1 {
            panic!("unexpect seq number")
        }
        last_seq = cur_seq;
    }
}

fn new_wal_path<P: AsRef<Path>>(dir: P, idx: Index) -> PathBuf {
    dir.as_ref()
        .join(format!("{:016x}-{:016x}.{}", idx.0, idx.1, WAL_EXT))
}

fn index_from_path<P: AsRef<Path>>(p: P) -> Index {
    let err_msg = format!(
        "invalid format of file name {}",
        p.as_ref().to_string_lossy()
    );
    let stem = p
        .as_ref()
        .file_stem()
        .map(|stem| {
            stem.to_str()
                .expect(&format!("invalid str {}", stem.to_string_lossy()))
        })
        .expect(&err_msg);
    match &*stem.split('-').collect::<Vec<&str>>() {
        &[seq, index] => {
            let seq = u64::from_str(seq).expect(&err_msg);
            let index = u64::from_str(index).expect(&err_msg);
            return (seq, index);
        }
        _ => panic!("invalid format of file name"),
    }
}

#[cfg(test)]
mod tests {}
