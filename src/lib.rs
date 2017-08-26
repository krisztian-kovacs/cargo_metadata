#![deny(missing_docs)]
//! Structured access to the output of `cargo metadata`
//! Usually used from within a `cargo-*` executable
//!
//! ```rust
//! # extern crate cargo_metadata;
//! let manifest_path_arg = std::env::args()
//!     .skip(2)
//!     .find(|val| val.starts_with("--manifest-path="));
//! let metadata = cargo_metadata::metadata(manifest_path_arg.as_ref().map(AsRef::as_ref)).unwrap();
//! ```

#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

pub use errors::{Error, Result};
pub use dependency::{Dependency, DependencyKind};

mod dependency;

#[derive(Clone, Deserialize, Debug)]
/// Starting point for metadata returned by `cargo metadata`
pub struct Metadata {
    /// A list of all crates referenced by this crate (and the crate itself)
    pub packages: Vec<Package>,
    /// A list of all workspace members
    #[serde(default)]
    pub workspace_members: Vec<String>,
    /// Dependencies graph
    pub resolve: Option<Resolve>,
    version: usize,
}

#[derive(Clone, Deserialize, Debug)]
/// A dependency graph
pub struct Resolve {
    /// Nodes in a dependencies graph
    pub nodes: Vec<Node>,
}

#[derive(Clone, Deserialize, Debug)]
/// A node in a dependencies graph
pub struct Node {
    /// An opaque identifier for a package
    pub id: String,
    /// List of opaque identifiers for this node's dependencies
    pub dependencies: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
/// A crate
pub struct Package {
    /// Name as given in the `Cargo.toml`
    pub name: String,
    /// Version given in the `Cargo.toml`
    pub version: String,
    /// An opaque identifier for a package
    pub id: String,
    source: Option<String>,
    /// List of dependencies of this particular package
    pub dependencies: Vec<Dependency>,
    /// Targets provided by the crate (lib, bin, example, test, ...)
    pub targets: Vec<Target>,
    features: HashMap<String, Vec<String>>,
    /// Path containing the `Cargo.toml`
    pub manifest_path: String,
}

#[derive(Clone, Deserialize, Debug)]
/// A single target (lib, bin, example, ...) provided by a crate
pub struct Target {
    /// Name as given in the `Cargo.toml` or generated from the file name
    pub name: String,
    /// Kind of target ("bin", "example", "test", "bench", "lib")
    pub kind: Vec<String>,
    /// Almost the same as `kind`, except when an example is a library instad of an executable.
    /// In that case `crate_types` contains things like `rlib` and `dylib` while `kind` is `example`
    #[serde(default)]
    pub crate_types: Vec<String>,
    /// Path to the main source file of the target
    pub src_path: String,
}

mod errors {
    //! Create the `Error`, `ErrorKind`, `ResultExt`, and `Result` types
    error_chain!{
        foreign_links {
            // Error during execution of `cargo metadata`
            Io(::std::io::Error);
            // Output of `cargo metadata` was not valid utf8
            Utf8(::std::str::Utf8Error);
            // Deserialization error (structure of json did not match expected structure)
            Json(::serde_json::Error);
        }
    }
}

/// Obtain metadata only about the root package and don't fetch dependencies
///
/// # Parameters
///
/// - `manifest_path`: Path to the manifest.
pub fn metadata(manifest_path: Option<&Path>) -> Result<Metadata> {
    metadata_deps(manifest_path, false)
}

/// The main entry point to obtaining metadata
///
/// # Parameters
///
/// - `manifest_path`: Path to the manifest.
/// - `deps`: Whether to include dependencies.
pub fn metadata_deps(manifest_path: Option<&Path>, deps: bool) -> Result<Metadata> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let mut cmd = Command::new(cargo);
    cmd.arg("metadata");

    if !deps {
        cmd.arg("--no-deps");
    }

    cmd.args(&["--format-version", "1"]);
    if let Some(manifest_path) = manifest_path {
        cmd.arg("--manifest-path").arg(manifest_path.as_os_str());
    }
    let output = cmd.output()?;
    let stdout = from_utf8(&output.stdout)?;
    let meta = serde_json::from_str(stdout)?;
    Ok(meta)
}
