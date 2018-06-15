use super::record::*;
use bincode::{deserialize, serialize, serialized_size};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use errors::*;
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const SEGMENT_SIZE: u64 = 32 * 1000 * 1000;

pub struct WalFile {
    path: PathBuf,
    fd: BufWriter<File>,
}

impl AsRef<Path> for WalFile {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl WalFile {
    pub fn create<P: AsRef<Path>>(wal_file_path: P) -> Result<WalFile> {
        let wal_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(&wal_file_path)?;
        // check the size of disk
        //wal_file.allocate(SEGMENT_SIZE as u64)?;
        wal_file.try_lock_exclusive()?;

        Ok(WalFile {
            fd: BufWriter::new(wal_file),
            path: wal_file_path.as_ref().to_path_buf(),
        })
    }

    pub fn open<P: AsRef<Path>>(wal_file_path: P) -> Result<WalFile> {
        let wal_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(&wal_file_path)?;
        wal_file.try_lock_exclusive()?;
        Ok(WalFile {
            fd: BufWriter::new(wal_file),
            path: wal_file_path.as_ref().to_path_buf(),
        })
    }

    pub fn insert_record(&mut self, r: Record) -> Result<()> {
        self.write_len(serialized_size(&r)?)?;
        self.fd.write_all(serialize(&r)?.as_slice())?;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<()> {
        self.fd.flush()?;
        self.fd.get_ref().sync_all()?;
        Ok(())
    }

    pub fn size(&mut self) -> Result<u64> {
        Ok(self.fd.seek(SeekFrom::Current(0))?)
    }

    pub fn check_cut(&mut self) -> Result<bool> {
        let size = self.size()?;
        if size >= SEGMENT_SIZE {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn file_size(&self) -> Result<u64> {
        Ok(self.fd.get_ref().metadata()?.len())
    }

    fn write_len(&mut self, len: u64) -> Result<()> {
        self.fd.write_u64::<LittleEndian>(len)?;
        Ok(())
    }
}

impl<'a> IntoIterator for &'a mut WalFile {
    type Item = Record;
    type IntoIter = WalFileIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        // create our iterator
        let fd = self.fd.get_mut();
        fd.seek(SeekFrom::Start(0)).unwrap();
        WalFileIterator {
            fd: BufReader::new(fd),
        }
    }
}

pub struct WalFileIterator<'a> {
    fd: BufReader<&'a File>,
}

impl<'a> WalFileIterator<'a> {
    fn read_len(&mut self) -> Result<u64> {
        let len = self.fd.read_u64::<LittleEndian>()?;
        Ok(len)
    }

    fn read_record(&mut self) -> Result<Record> {
        let len = self.read_len()?;
        let mut buffer = vec![0; len as usize];
        self.fd.read(&mut buffer)?;
        let record: Record = deserialize(&buffer)?;
        Ok(record)
    }
}

impl<'a> Iterator for WalFileIterator<'a> {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_record() {
            Ok(record) => {
                debug!("read record {:?}", record);
                Some(record)
            }
            Err(err) => {
                warn!("read record from file error {}", err);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_test() {
        let wal_path = "./file_tests/wal/test.wal";
        let mut wal_file = WalFile::create(wal_path).unwrap();
        wal_file.fd.get_ref().set_len(0).unwrap();
        let r = Record::new(RecordType::EntryType, vec![1, 2, 3, 4, 5]);
        wal_file.insert_record(r.clone()).unwrap();
        let r = Record::new(RecordType::EntryType, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
        wal_file.insert_record(r.clone()).unwrap();
        let r = Record::new(
            RecordType::EntryType,
            vec![
                1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0,
            ],
        );
        wal_file.insert_record(r.clone()).unwrap();
        wal_file.sync().unwrap();
        let mut iter = wal_file.into_iter();
        println!("{:?}", iter.next().unwrap());
        println!("{:?}", iter.next().unwrap());
        println!("{:?}", iter.next().unwrap());
        // let rr = wal_file.into_iter().next().unwrap();
        // assert_eq!(r, rr);
        // let r = Record::new(RecordType::EntryType, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
        // wal_file.insert_record(r.clone()).unwrap();
        // wal_file.sync().unwrap();
        // wal_file.into_iter().next().unwrap();
        // let rr = wal_file.into_iter().next().unwrap();
        // assert_eq!(r, rr);
    }

    #[test]
    fn iter_test() {}
}
