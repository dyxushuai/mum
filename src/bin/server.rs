extern crate mum;
#[macro_use]
extern crate structopt;
extern crate futures;
extern crate grpcio;
#[macro_use]
extern crate log;
extern crate env_logger;

use grpcio::Environment;
use mum::prelude::*;
use std::str::FromStr;
use std::sync::Arc;

use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "server")]
struct Opt {
    /// Output file
    #[structopt(short = "wd", long = "wal_dir", parse(from_os_str))]
    wal_dir: PathBuf,

    #[structopt(short = "sd", long = "snap_dir", parse(from_os_str))]
    snap_dir: PathBuf,

    #[structopt(short = "as", long = "addrs")]
    addrs: Vec<String>,

    #[structopt(long = "id")]
    id: u64,
}

fn main() {
    env_logger::init();
    // let _guard = init_log(None);
    let opt = Opt::from_args();
    let env = Arc::new(Environment::new(4));
    let kv_store = Store::new();

    let (mum_grpc, rx) = MumServer::new(kv_store.clone());

    let node = Node::new(
        opt.id,
        env.clone(),
        opt.addrs.clone(),
        kv_store.clone(),
        opt.wal_dir,
        opt.snap_dir,
        rx,
    ).unwrap();

    let mum_addr: Vec<&str> = opt
        .addrs
        .get((opt.id - 1) as usize)
        .unwrap()
        .split(':')
        .collect();
    let env = Arc::new(Environment::new(4));
    let mut server = mum_grpc
        .into_server(
            env.clone(),
            mum_addr[0],
            u16::from_str(mum_addr[1]).unwrap(),
        )
        .unwrap();
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        info!("listening on {}:{}", host, port);
    }

    node.run();
}
