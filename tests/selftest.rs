extern crate cargo_metadata;
extern crate semver;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::env::current_dir;
use std::path::{Path, PathBuf};

use semver::Version;

use cargo_metadata::{Error, ErrorKind, MetadataCommand, CargoOpt};

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct TestPackageMetadata {
    some_field: bool,
    other_field: String,
}

#[test]
fn metadata() {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();

    assert_eq!(
        current_dir().unwrap().join("target"),
        Path::new(&metadata.target_directory)
    );

    assert_eq!(metadata.packages[0].name, "cargo_metadata");
    assert_eq!(metadata.packages[0].targets.len(), 2);

    assert_eq!(metadata.packages[0].targets[0].name, "cargo_metadata");
    assert_eq!(metadata.packages[0].targets[0].kind[0], "lib");
    assert_eq!(metadata.packages[0].targets[0].crate_types[0], "lib");

    assert_eq!(metadata.packages[0].targets[1].name, "selftest");
    assert_eq!(metadata.packages[0].targets[1].kind[0], "test");
    assert_eq!(metadata.packages[0].targets[1].crate_types[0], "bin");

    let package_metadata = &metadata.packages[0].metadata.as_object()
        .expect("package.metadata must be a table.");
    assert_eq!(package_metadata.len(), 1);

    let value = package_metadata.get("cargo_metadata_test").unwrap();
    let test_package_metadata: TestPackageMetadata = serde_json::from_value(value.clone())
        .unwrap();
    assert_eq!(test_package_metadata, TestPackageMetadata {
        some_field: true,
        other_field: "foo".into(),
    });
}

#[test]
fn builder_interface() {
    let _ = MetadataCommand::new()
        .manifest_path("Cargo.toml")
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path(String::from("Cargo.toml"))
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path(PathBuf::from("Cargo.toml"))
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path("Cargo.toml")
        .no_deps()
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path("Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .exec()
        .unwrap();
    let _ = MetadataCommand::new()
        .manifest_path("Cargo.toml")
        .current_dir(current_dir().unwrap())
        .exec()
        .unwrap();
}

#[test]
fn error1() {
    match MetadataCommand::new().manifest_path("foo").exec() {
        Err(Error(ErrorKind::CargoMetadata(s), _)) => assert_eq!(
            s.trim(),
            "error: the manifest-path must be a path to a Cargo.toml file"
        ),
        _ => unreachable!(),
    }
}

#[test]
fn error2() {
    match MetadataCommand::new().manifest_path("foo/Cargo.toml").exec() {
        Err(Error(ErrorKind::CargoMetadata(s), _)) => assert_eq!(
            s.trim(),
            "error: manifest path `foo/Cargo.toml` does not exist"
        ),
        _ => unreachable!(),
    }
}

#[test]
fn metadata_deps() {
    let metadata = MetadataCommand::new().manifest_path("Cargo.toml").exec().unwrap();
    let this_id = metadata.workspace_members
        .first()
        .expect("Did not find ourselves");
    let this = &metadata[this_id];

    assert_eq!(this.name, "cargo_metadata");
    assert_eq!(this.targets.len(), 2);

    assert_eq!(this.targets[0].name, "cargo_metadata");
    assert_eq!(this.targets[0].kind[0], "lib");
    assert_eq!(this.targets[0].crate_types[0], "lib");

    assert_eq!(this.targets[1].name, "selftest");
    assert_eq!(this.targets[1].kind[0], "test");
    assert_eq!(this.targets[1].crate_types[0], "bin");

    let dependencies = &this.dependencies;

    let serde = dependencies
        .iter()
        .find(|dep| dep.name == "serde")
        .expect("Did not find serde dependency");

    assert_eq!(serde.kind, cargo_metadata::DependencyKind::Normal);
    assert!(!serde.req.matches(&Version::parse("1.0.0").unwrap()));
    assert!(serde.req.matches(&Version::parse("1.99.99").unwrap()));
    assert!(!serde.req.matches(&Version::parse("2.0.0").unwrap()));
}
