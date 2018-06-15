use crc::crc32;
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum RecordType {
    EntryType,
    StateType,
    CrcType,
    IndexType,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Record {
    pub ty: RecordType,
    pub crc: u32,
    pub data: Vec<u8>,
}

impl Record {
    pub fn new(ty: RecordType, data: Vec<u8>) -> Record {
        Record {
            ty: ty,
            crc: crc32::update(0, &crc32::CASTAGNOLI_TABLE, &data),
            data: data,
        }
    }
}
