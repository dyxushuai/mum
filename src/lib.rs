#![feature(integer_atomics)]
extern crate bincode;
extern crate byteorder;
extern crate crc;
extern crate failure;
extern crate fs2;
extern crate futures;
extern crate grpcio;
extern crate indexmap;
extern crate protobuf;
extern crate raft;
extern crate serde;
extern crate tokio;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate itertools;
#[macro_use]
extern crate prometheus;

mod kv;
mod metrics;
mod node;
mod server;
mod snap;
mod transport;
mod util;
mod wal;

pub mod errors;
pub mod proto;

pub mod prelude {
    pub use kv::Store;
    pub use node::Node;
    pub use server::MumServer;
    //pub use raft_server::RaftServer;
}
