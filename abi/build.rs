use std::{fs, process::Command};

fn main() {
    std::env::set_var("PROTOC", protobuf_src::protoc());
    tonic_build::configure()
        .out_dir("src/pb")
        .compile(&["proto/rsvp.proto"], &["proto"])
        .unwrap();

    fs::remove_file("src/pb/google.protobuf.rs").unwrap();

    Command::new("cargo").arg("fmt").output().unwrap();

    println!("cargo:rerun-if-changed=proto/rsvp.proto");
    println!("cargo:rerun-if-changed=./build.rs");
}
