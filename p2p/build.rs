fn main() {
    let mut cfg = prost_build::Config::new();
    cfg.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    cfg.compile_protos(
        &["src/network/pubsub/message.proto"],
        &["src/network/pubsub"],
    )
    .unwrap();
}
