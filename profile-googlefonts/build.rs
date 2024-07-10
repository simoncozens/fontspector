fn main() {
    println!("Building protos");
    protobuf_codegen::Codegen::new()
        .pure()
        // All inputs and imports from the inputs must reside in `includes` directories.
        .includes(&["src/protos"])
        .input("src/protos/fonts_public.proto")
        // Specify output directory relative to Cargo output directory.
        .cargo_out_dir("protos")
        .run_from_script();
    println!("cargo::rerun-if-changed=build.rs");
}
