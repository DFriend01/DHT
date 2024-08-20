use protobuf_codegen::Codegen;

fn main() {
    const CARGO_OUT_DIR: &str = "src/comm/protos";
    match std::fs::create_dir_all(CARGO_OUT_DIR) {
        Ok(_) => (),
        Err(_e) => eprintln!("{} already exists", CARGO_OUT_DIR),
    }

    Codegen::new()
        .protoc()
        .includes(&["proto"])
        .input("proto/udp_message.proto")
        .input("proto/request.proto")
        .input("proto/reply.proto")
        .out_dir(CARGO_OUT_DIR)
        .run_from_script();
}
