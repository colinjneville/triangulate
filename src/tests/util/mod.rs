//! Testing and benchmark utilities

pub mod polygon;
mod load_polygon_list;
use std::{env, path};

pub use load_polygon_list::load_polygon_list;

/// Returns a directory containing sample polygon lists
pub fn countries_path() -> path::PathBuf {
    path::Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("resources").join("geometry").join("countries")
}