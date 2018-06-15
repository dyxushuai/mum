extern crate mum;
#[macro_use]
extern crate structopt;
extern crate grpcio;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate raft;

use grpcio::{ChannelBuilder, EnvBuilder};
use mum::proto::{mumpb::*, mumpb_grpc::*};
use raft::eraftpb::{ConfChange, ConfChangeType};
use std::sync::Arc;

use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "ctl")]
enum Opt {
    #[structopt(name = "kv")]
    KV {
        #[structopt(short = "op", long = "op")]
        op: String,
        #[structopt(short = "y", long = "y")]
        key: String,
        #[structopt(short = "val", long = "value")]
        value: String,
        #[structopt(short = "l", long = "limit")]
        limit: Option<u32>,
        #[structopt(short = "ka", long = "kv_addr")]
        kv_addr: String,
    },

    #[structopt(name = "conf")]
    Conf {
        #[structopt(short = "op", long = "op")]
        op: String,
        #[structopt(short = "ni", long = "node_id")]
        node_id: u64,
        #[structopt(short = "url", long = "url")]
        url: String,
        #[structopt(short = "ka", long = "kv_addr")]
        kv_addr: String,
    },
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    let env = Arc::new(EnvBuilder::new().build());
    match opt {
        Opt::KV {
            op,
            key,
            value,
            limit,
            kv_addr,
        } => {
            let ch = ChannelBuilder::new(env).connect(&kv_addr);
            let client = MumClient::new(ch);
            let o = match op.as_str() {
                "set" => Op::Set,
                "get" => Op::Get,
                "delete" => Op::Del,
                "scan" => Op::Scan,
                _ => panic!("unexpect op {}, wanted: set/get/delete/scan", op),
            };

            let req = make_kv_request(o, key, value, limit);
            let reply = client.op(&req).expect("rpc");
            for kv in reply.get_kvs() {
                info!(
                    "key: {} / value: {}",
                    String::from_utf8(kv.get_key().to_vec()).unwrap(),
                    String::from_utf8(kv.get_value().to_vec()).unwrap()
                );
            }
        }
        Opt::Conf {
            op,
            node_id,
            url,
            kv_addr,
        } => {
            let ch = ChannelBuilder::new(env).connect(&kv_addr);
            let client = MumClient::new(ch);
            let o = match op.as_str() {
                "add" => ConfChangeType::AddNode,
                "remove" => ConfChangeType::RemoveNode,
                _ => panic!("unexpect op {}, wanted: add/remove", op),
            };

            let req = make_conf_request(o, node_id, url);
            client.conf(&req).expect("rpc");
        }
    }
}

fn make_conf_request(t: ConfChangeType, node_id: u64, url: String) -> ConfRequest {
    let mut req = ConfRequest::new();
    let mut raft_conf = ConfChange::new();
    raft_conf.set_change_type(t);
    raft_conf.set_node_id(node_id);
    raft_conf.set_context(url.into_bytes());
    req.set_change(raft_conf);
    req
}

fn make_kv_request(op: Op, key: String, value: String, limit: Option<u32>) -> OpRequest {
    let mut req = OpRequest::new();
    req.set_field_type(op);
    req.set_key(key.into_bytes());
    req.set_value(value.into_bytes());
    if let Some(v) = limit {
        req.set_limit(v);
    }
    req
}
