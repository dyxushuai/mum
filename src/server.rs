use errors::Result;
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::Future;
use grpcio::*;
use kv::Store;
use proto::mumpb::*;
use proto::mumpb_grpc::{create_mum, Mum};
use protobuf::RepeatedField;
use std::sync::Arc;

#[derive(Clone)]
pub struct MumServer {
    store: Store,
    tx: Arc<UnboundedSender<(Option<RaftMessage>, Option<OpRequest>, Option<ConfRequest>)>>,
}

impl MumServer {
    pub fn new(
        store: Store,
    ) -> (
        MumServer,
        UnboundedReceiver<(Option<RaftMessage>, Option<OpRequest>, Option<ConfRequest>)>,
    ) {
        let (tx, rx) = unbounded();
        (
            MumServer {
                store: store,
                tx: Arc::new(tx),
            },
            rx,
        )
    }

    pub fn into_server<S: Into<String>>(
        self,
        env: Arc<Environment>,
        host: S,
        port: u16,
    ) -> Result<Server> {
        let service = create_mum(self);
        let server = ServerBuilder::new(env)
            .register_service(service)
            .bind(host, port)
            .build()?;
        Ok(server)
    }
}

impl Mum for MumServer {
    fn raft(&self, ctx: RpcContext, req: RaftMessage, sink: UnarySink<Done>) {
        let tx = self.tx.clone();
        let resp = Done::new();
        tx.unbounded_send((Some(req), None, None)).unwrap();
        ctx.spawn(sink.success(resp).map_err(|_| ()));
    }

    fn op(&self, ctx: RpcContext, req: OpRequest, sink: UnarySink<OpResponse>) {
        let store = self.store.clone();
        let tx = self.tx.clone();
        let mut resp = OpResponse::new();
        let mut kvs = RepeatedField::new();
        match req.field_type {
            Op::Set | Op::Del => {
                tx.unbounded_send((None, Some(req), None)).unwrap();
            }
            Op::Get => match store.get(&req.key) {
                Some(v) => {
                    let mut kv = KvPair::new();
                    kv.set_key(req.key);
                    kv.set_value(v);
                    kvs.push(kv);
                    resp.set_kvs(kvs);
                }
                _ => (),
            },
            Op::Scan => {
                for (k, v) in store.scan(req.get_key(), req.get_limit()) {
                    let mut kv = KvPair::new();
                    kv.set_key(k);
                    kv.set_value(v);
                    kvs.push(kv);
                }
            }
        }
        ctx.spawn(sink.success(resp).map_err(|_| ()));
    }

    fn conf(&self, ctx: RpcContext, req: ConfRequest, sink: UnarySink<ConfResponse>) {
        let tx = self.tx.clone();
        tx.unbounded_send((None, None, Some(req))).unwrap();
        ctx.spawn(sink.success(ConfResponse::new()).map_err(|_| ()));
    }
}
