use std::fs;

use crate::{Triangulate, builders};

use super::util;

#[test]
fn triangulate() {
    for polygon in util::polygon::all() {
        polygon.triangulate::<builders::VecVecFanBuilder<_>>(&mut Vec::new()).expect("Triangulation failed");
    }
}

#[test]
fn triangulate_hollow() {
    let polygon: Vec<Vec<util::VTest>> = vec![
        vec![(0., 0.).into(), (0., 1.).into(), (1., 1.).into(), (1., 0.).into()], 
        vec![(0.05, 0.05).into(), (0.05, 0.95).into(), (0.95, 0.95).into(), (0.95, 0.05).into()]
    ];
    polygon.triangulate::<builders::VecVecFanBuilder<_>>(&mut Vec::new()).expect("Triangulation failed");
}

#[test]
fn triangulate_geography() {
    for file in fs::read_dir(util::countries_path()).unwrap() {
        let file = file.unwrap();
        let polygon_list = util::load_polygon_list(file.path().to_str().unwrap()).unwrap();
        // A few countries have a lot of vertices, so for the sake of time, skip those
        if crate::PolygonList::vertex_count(&polygon_list) <= 4000 {
            polygon_list.triangulate::<builders::VecVecFanBuilder<_>>(&mut Vec::new()).expect("Triangulation failed");
            println!("'{}' completed", file.file_name().to_str().unwrap());
        }
    }
}