use bincode::Error as BincodeError;
use grpcio::Error as GrpcIoError;
use protobuf::error::ProtobufError;
use raft::Error as RaftError;
use std::io::Error as IoError;
use std::result::Result as StdResult;
use std::str::Utf8Error;
use tokio::timer::Error as TokioTimerError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Bincode(#[cause] BincodeError),
    #[fail(display = "miss metadata in snapshot message")]
    MissMetadata,
    #[fail(display = "{}", _0)]
    Protbuf(#[cause] ProtobufError),
    #[fail(display = "{}", _0)]
    Io(#[cause] ::std::io::Error),
    #[fail(display = "crc miss match")]
    CrcMissMatch,
    #[fail(display = "snapshot miss match")]
    SnapMissMatch,
    #[fail(display = "file path {} already exists", _0)]
    FilePathExists(String),
    #[fail(display = "file path {} not found", _0)]
    MissFilePath(String),
    #[fail(display = "{}", _0)]
    GrpcIo(#[cause] GrpcIoError),
    #[fail(display = "sink error")]
    Sink,
    #[fail(display = "future error")]
    Future,
    #[fail(display = "{}", _0)]
    Raft(#[cause] RaftError),
    #[fail(display = "{}", _0)]
    Utf8(#[cause] Utf8Error),
    #[fail(display = "unbound receiver {} error", _0)]
    UnboundReceiverError(String),
    #[fail(display = "{}", _0)]
    TokioTimer(#[cause] TokioTimerError),
}

impl From<TokioTimerError> for Error {
    fn from(e: TokioTimerError) -> Error {
        Error::TokioTimer(e)
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Error {
        Error::Utf8(e)
    }
}

impl From<RaftError> for Error {
    fn from(e: RaftError) -> Error {
        Error::Raft(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<GrpcIoError> for Error {
    fn from(e: GrpcIoError) -> Error {
        Error::GrpcIo(e)
    }
}

impl From<ProtobufError> for Error {
    fn from(e: ProtobufError) -> Error {
        Error::Protbuf(e)
    }
}

impl From<BincodeError> for Error {
    fn from(e: BincodeError) -> Error {
        Error::Bincode(e)
    }
}

pub type Result<T> = StdResult<T, Error>;
