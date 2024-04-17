fn main() {
    prost_build::compile_protos(
        &["src/network/pubsub/message.proto"],
        &["src/network/pubsub"],
    )
    .unwrap();
}
