use std::fs;

use crate::{formats, Polygon, PolygonList, ListFormat};

use super::util;

#[test]
fn triangulate() {
    for polygon in util::polygon::all() {
        polygon.triangulate(formats::IndexedFanFormat::new(&mut Vec::<Vec<_>>::new())).expect("Triangulation failed");
    }
}

#[test]
fn triangulate_hollow() {
    let polygon = vec![
        vec![[0f32, 0f32], [0., 1.], [1., 1.], [1., 0.]], 
        vec![[0.05, 0.05], [0.05, 0.95], [0.95, 0.95], [0.95, 0.05]]
    ];
    polygon.triangulate(formats::IndexedFanFormat::new(&mut Vec::<Vec<_>>::new())).expect("Triangulation failed");
}

#[test]
fn triangulate_geography() {
    for file in fs::read_dir(util::countries_path()).unwrap() {
        let file = file.unwrap();
        let polygon_list = util::load_polygon_list(file.path().to_str().unwrap()).unwrap();
        // A few countries have a lot of vertices, so for the sake of time, skip those
        if polygon_list.vertex_count() <= 4000 {
            polygon_list.triangulate(formats::IndexedFanFormat::new(&mut Vec::<Vec<_>>::new())).expect("Triangulation failed");
            println!("'{}' completed", file.file_name().to_str().unwrap());
        }
    }
}

#[test]
fn regular_polygons() {
    for n in 3..=500 {
        let mut p = Vec::new();
        for nn in 0..n {
            let scalar = 100.;

            let theta = std::f64::consts::PI * 2. * (nn as f64) / (n as f64);
            let (x, y) = theta.sin_cos();
            let v = [x * scalar, y * scalar];
            p.push(v);
        }

        let mut output = Vec::<usize>::new();
        let format = formats::IndexedListFormat::new(&mut output).into_fan_format();
        p.triangulate(format).expect("Triangulation failed");
        std::hint::black_box(output);
    }
}
