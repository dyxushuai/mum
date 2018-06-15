#![allow(dead_code)]
// Copyright 2016 PingCAP, Inc.
use prometheus::*;

lazy_static! {
    pub static ref SEND_SNAP_HISTOGRAM: Histogram = register_histogram!(
        "mum_server_send_snapshot_duration_seconds",
        "Bucketed histogram of server send snapshots duration",
        exponential_buckets(0.05, 2.0, 20).unwrap()
    ).unwrap();
    pub static ref SNAP_TASK_COUNTER: IntCounterVec = register_int_counter_vec!(
        "mum_server_snapshot_task_total",
        "Total number of snapshot task",
        &["type"]
    ).unwrap();
    pub static ref GRPC_MSG_HISTOGRAM_VEC: HistogramVec = register_histogram_vec!(
        "mum_grpc_msg_duration_seconds",
        "Bucketed histogram of grpc server messages",
        &["type"],
        exponential_buckets(0.0005, 2.0, 20).unwrap()
    ).unwrap();
    pub static ref GRPC_MSG_FAIL_COUNTER: IntCounterVec = register_int_counter_vec!(
        "mum_grpc_msg_fail_total",
        "Total number of handle grpc message failure",
        &["type"]
    ).unwrap();
    pub static ref RAFT_MESSAGE_RECV_COUNTER: IntCounter = register_int_counter!(
        "mum_server_raft_message_recv_total",
        "Total number of raft messages received"
    ).unwrap();
    pub static ref RESOLVE_STORE_COUNTER: IntCounterVec = register_int_counter_vec!(
        "mum_server_resolve_store_total",
        "Total number of resolving store",
        &["type"]
    ).unwrap();
    pub static ref REPORT_FAILURE_MSG_COUNTER: IntCounterVec = register_int_counter_vec!(
        "mum_server_report_failure_msg_total",
        "Total number of reporting failure messages",
        &["type", "store_id"]
    ).unwrap();
    pub static ref RAFT_MESSAGE_FLUSH_COUNTER: IntCounter = register_int_counter!(
        "mum_server_raft_message_flush_total",
        "Total number of raft messages flushed"
    ).unwrap();
}
