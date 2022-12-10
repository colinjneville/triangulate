use std::hint;

use criterion::{Criterion, criterion_group, criterion_main};
use triangulate::ListFormat;
use triangulate::formats;
use triangulate::tests::util;
use triangulate::PolygonList;

// cargo bench --profile=release-symbols -F "_benchmarking"
pub fn criterion_benchmark(c: &mut Criterion) {
    let polygon_list = util::load_polygon_list(util::countries_path().join("russia.txt").to_str().unwrap()).unwrap();
    let polygon_list = polygon_list.index_with::<usize, u16>();

    c.bench_function("russia", |b| b.iter(|| {
        let mut output = Vec::<[_; 3]>::new();
        let builder = formats::IndexedListFormat::new(&mut output).into_fan_format();
        polygon_list.triangulate(builder).expect("Triangulation failed");

        hint::black_box(output);
    }));
}

pub fn criterion_benchmark_earcutr(c: &mut Criterion) {
    use triangulate::Vertex;

    let polygon_list = util::load_polygon_list(util::countries_path().join("russia.txt").to_str().unwrap()).unwrap();
    let polygon_list: Vec<Vec<f32>> = polygon_list.into_iter().map(|p| p.iter().flat_map(|v| [v.x(), v.y()].into_iter()).collect()).collect();

    c.bench_function("russia_earcutr", |b| b.iter(|| {
        for polygon in polygon_list.iter() {
            let triangles = earcutr::earcut(polygon, &[], 2);
            hint::black_box(triangles);
        }
    }));
}

criterion_group!(benches, criterion_benchmark, criterion_benchmark_earcutr);
criterion_main!(benches);