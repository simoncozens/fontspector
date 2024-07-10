fn main() {
    protobuf_codegen::Codegen::new()
        .pure()
        .includes(&["src/protos"])
        .input("src/protos/fonts_public.proto")
        .cargo_out_dir("protos")
        .run_from_script();
}
