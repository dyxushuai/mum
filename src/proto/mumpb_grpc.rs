// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_MUM_OP: ::grpcio::Method<super::mumpb::OpRequest, super::mumpb::OpResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/mumpb.Mum/Op",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_MUM_CONF: ::grpcio::Method<super::mumpb::ConfRequest, super::mumpb::ConfResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/mumpb.Mum/Conf",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_MUM_RAFT: ::grpcio::Method<super::mumpb::RaftMessage, super::mumpb::Done> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/mumpb.Mum/Raft",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct MumClient {
    client: ::grpcio::Client,
}

impl MumClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        MumClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn op_opt(&self, req: &super::mumpb::OpRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::mumpb::OpResponse> {
        self.client.unary_call(&METHOD_MUM_OP, req, opt)
    }

    pub fn op(&self, req: &super::mumpb::OpRequest) -> ::grpcio::Result<super::mumpb::OpResponse> {
        self.op_opt(req, ::grpcio::CallOption::default())
    }

    pub fn op_async_opt(&self, req: &super::mumpb::OpRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::mumpb::OpResponse>> {
        self.client.unary_call_async(&METHOD_MUM_OP, req, opt)
    }

    pub fn op_async(&self, req: &super::mumpb::OpRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::mumpb::OpResponse>> {
        self.op_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn conf_opt(&self, req: &super::mumpb::ConfRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::mumpb::ConfResponse> {
        self.client.unary_call(&METHOD_MUM_CONF, req, opt)
    }

    pub fn conf(&self, req: &super::mumpb::ConfRequest) -> ::grpcio::Result<super::mumpb::ConfResponse> {
        self.conf_opt(req, ::grpcio::CallOption::default())
    }

    pub fn conf_async_opt(&self, req: &super::mumpb::ConfRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::mumpb::ConfResponse>> {
        self.client.unary_call_async(&METHOD_MUM_CONF, req, opt)
    }

    pub fn conf_async(&self, req: &super::mumpb::ConfRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::mumpb::ConfResponse>> {
        self.conf_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn raft_opt(&self, req: &super::mumpb::RaftMessage, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::mumpb::Done> {
        self.client.unary_call(&METHOD_MUM_RAFT, req, opt)
    }

    pub fn raft(&self, req: &super::mumpb::RaftMessage) -> ::grpcio::Result<super::mumpb::Done> {
        self.raft_opt(req, ::grpcio::CallOption::default())
    }

    pub fn raft_async_opt(&self, req: &super::mumpb::RaftMessage, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::mumpb::Done>> {
        self.client.unary_call_async(&METHOD_MUM_RAFT, req, opt)
    }

    pub fn raft_async(&self, req: &super::mumpb::RaftMessage) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::mumpb::Done>> {
        self.raft_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Mum {
    fn op(&self, ctx: ::grpcio::RpcContext, req: super::mumpb::OpRequest, sink: ::grpcio::UnarySink<super::mumpb::OpResponse>);
    fn conf(&self, ctx: ::grpcio::RpcContext, req: super::mumpb::ConfRequest, sink: ::grpcio::UnarySink<super::mumpb::ConfResponse>);
    fn raft(&self, ctx: ::grpcio::RpcContext, req: super::mumpb::RaftMessage, sink: ::grpcio::UnarySink<super::mumpb::Done>);
}

pub fn create_mum<S: Mum + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_MUM_OP, move |ctx, req, resp| {
        instance.op(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_MUM_CONF, move |ctx, req, resp| {
        instance.conf(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_MUM_RAFT, move |ctx, req, resp| {
        instance.raft(ctx, req, resp)
    });
    builder.build()
}
