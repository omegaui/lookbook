//! Exits with an error when the app's catalog model cannot read the file:
//! an unknown enum value, a missing field or a wrong type.

#![allow(dead_code)]

use std::process::ExitCode;

#[path = "../../../src/catalog/model.rs"]
mod model;
#[path = "../../../src/catalog/protocols.rs"]
mod protocols;

// model.rs and protocols.rs refer to each other through their parent module.
pub use model::*;
pub use protocols::ProtocolInfo;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: catalog-model-check CATALOG_JSON");
        return ExitCode::FAILURE;
    };
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("Could not read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    match serde_json::from_slice::<Catalog>(&bytes) {
        Ok(catalog) => {
            println!("The app can read {path} (generated {}).", catalog.generated_at);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("The app would reject {path}: {error}");
            ExitCode::FAILURE
        }
    }
}
