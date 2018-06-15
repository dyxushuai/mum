# Mum - distributed kv store base on raft

## Usage

### Server

```sh
# server side 
RUST_LOG=server,mum cargo run --bin server -- --id 1 --snap_dir example_dir/1/snap/ --wal_dir example_dir/1/ --addrs  127.0.0.1:9005 --addrs  127.0.0.1:9006 --addrs  127.0.0.1:9007

RUST_LOG=server,mum cargo run --bin server -- --id 2 --snap_dir example_dir/2/snap/ --wal_dir example_dir/2/ --addrs  127.0.0.1:9005 --addrs  127.0.0.1:9006 --addrs  127.0.0.1:9007

RUST_LOG=server,mum cargo run --bin server -- --id 3 --snap_dir example_dir/3/snap/ --wal_dir example_dir/3/ --addrs  127.0.0.1:9005 --addrs  127.0.0.1:9006 --addrs  127.0.0.1:9007
```

### Client

```sh
# kv --op (get/set/del/scan)
# conf --op (add/remove)
RUST_LOG=ctl,mum ./target/debug/ctl kv --op get --y hello1 --value world1 --kv_addr 127.0.0.1:9005
```

## Features

- Use [Simple Write Ahead Log](#Simple-Write-Ahead-Log) to storing the [Raft][] logs for disaster recovery.
- Use [Simple MVCC K/V Storage](#Simple-MVCC-K/V-Storage)(MVCC based K/V Storage) for Key/Value storing.
- **[Raft][]** consensus algorithm.
- **[Tokio][]** networking framework for rust-lang.
- User interface base on **[Redis][]** protocol.(Maybe)

## Wanted

- [x] Use raft-rs, grpc-rs, tokio to build a simple HA key-value service 
- [x] Provide basic get, set, delete, and scan operations
- [x] The data saved in one node must be persistent in the disk  (restart can’t lose data).
- [x] Need to show - Kill minority node, the service can still work.
- [x] Need to show - Kill majority node, the service can not work.
- [x] Need to support add/remove node dynamically
- [ ] Use a benchmark tool to find some performance problems.

## Simple MVCC K/V Storage

### K/V Storage Features

- A Single-File Presistent Storage
- Copy-On-Write, Read-Lock-Free (MVCC)
- Auto garbage collection
- Snapshot (Maybe)

### K/V Storage TODOs

- [x] Get(Key) -> Option<Value>
- [x] Set(Key, Value)
- [x] Delete(Key) -> Option<Value>
- [x] Scan() -> Iter
- [ ] MVCC support
- [x] Snapshot support


## Simple [Write Ahead Log][WAL]

### WAL Features

- Multi segments
- Reply logs for Disaster Recovery

### WAL TODOs

- [x] Write logs
- [x] Read all logs

[Raft]: https://raft.github.io/
[Tokio]: https://tokio.rs/
[Redis]: https://redis.io/
[WAL]: https://en.wikipedia.org/wiki/Write-ahead_logging