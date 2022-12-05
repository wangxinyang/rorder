use std::{fs, process::Command};

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .type_attribute("rsvp.ReservationStatus", "#[derive(sqlx::Type)]")
        .compile(&["proto/rsvp.proto"], &["proto"])
        .unwrap();

    fs::remove_file("src/pb/google.protobuf.rs")
        .unwrap_or_else(|_| println!("the path is not existed"));

    Command::new("cargo").arg("fmt").output().unwrap();

    println!("cargo:rerun-if-changed=proto/rsvp.proto");
    println!("cargo:rerun-if-changed=./build.rs");
}
