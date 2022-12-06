use std::{fs, process::Command};
use tonic_build::Builder;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        // enhance the prost_build with trait BuilderExt
        .with_sql_type(&["rsvp.ReservationStatus"])
        // import the derive_builder crate for builder mode
        .with_derive_build(&["rsvp.ReservationQuery"])
        // avoid to warp the Option type for start parameter
        .with_builder_option("rsvp.ReservationQuery", &["start", "end"])
        .with_builder_into(
            "rsvp.ReservationQuery",
            &[
                "resource_id",
                "user_id",
                "status",
                "page",
                "page_size",
                "desc",
            ],
        )
        .compile(&["proto/rsvp.proto"], &["proto"])
        .unwrap();

    fs::remove_file("src/pb/google.protobuf.rs")
        .unwrap_or_else(|_| println!("the path is not existed"));

    Command::new("cargo").arg("fmt").output().unwrap();

    println!("cargo:rerun-if-changed=proto/rsvp.proto");
    println!("cargo:rerun-if-changed=./build.rs");
}

trait BuilderExt {
    fn with_sql_type(self, paths: &[&str]) -> Self;

    fn with_derive_build(self, paths: &[&str]) -> Self;

    fn with_builder_into(self, path: &str, fields: &[&str]) -> Self;

    fn with_builder_option(self, path: &str, fields: &[&str]) -> Self;
}

impl BuilderExt for Builder {
    fn with_sql_type(self, paths: &[&str]) -> Self {
        paths.iter().fold(self, |acc, path| {
            acc.type_attribute(path, "#[derive(sqlx::Type)]")
        })
    }

    fn with_derive_build(self, paths: &[&str]) -> Self {
        paths.iter().fold(self, |acc, path| {
            acc.type_attribute(path, "#[derive(derive_builder::Builder)]")
        })
    }

    fn with_builder_into(self, path: &str, fields: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            acc.field_attribute(
                format!("{}.{}", path, field),
                "#[builder(setter(into), default)]",
            )
        })
    }

    fn with_builder_option(self, path: &str, fields: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            acc.field_attribute(
                format!("{}.{}", path, field),
                "#[builder(setter(into, strip_option))]",
            )
        })
    }
}
