extern crate protoc_grpc;

fn build_proto() {
    protoc_grpc::run(protoc_grpc::Args {
        out_dir: "src/proto/",
        input: &["proto/mumpb.proto"],
        includes: &["proto"],
        rust_protobuf: true,
    }).expect("protoc");
}

fn main() {
    build_proto()
}
