fn main() {
    let mut cfg = prost_build::Config::new();
    cfg.type_attribute(
        ".",
        "#[derive(serde::Serialize, serde::Deserialize, malloc_size_of_derive::MallocSizeOf)]",
    );
    cfg.compile_protos(
        &[
            "src/network/pubsub/message.proto",
            "src/network/identify/p2p_network_identify_message.proto",
        ],
        &["src/network/pubsub", "src/network/identify"],
    )
    .expect("Proto build failed");
}
