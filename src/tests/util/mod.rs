pub mod polygon;
mod load_polygon_list;
mod vtest;
use std::{env, path};

pub use vtest::VTest;
pub use load_polygon_list::load_polygon_list;

pub fn countries_path() -> path::PathBuf {
    path::Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("resources").join("geometry").join("countries")
}