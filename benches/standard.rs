use std::fs;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use triangulate::builders;
use triangulate::tests::util;
use triangulate::TriangulateDefault;

pub fn criterion_benchmark(c: &mut Criterion) {
    // let mut group = c.benchmark_group("geography");

    // for file in fs::read_dir(util::countries_path()).unwrap() {
    //     let file = file.unwrap();
        
    //     let polygon_set = util::load_polygon_set(file.path().to_str().unwrap());

    //     polygon_set.triangulate_default::<def::VecVecFanBuilder<_>>().expect("Triangulation failed");
        
    //     group.throughput(Throughput::Elements(polygon_set.iter().map(|p| p.len() as u64).sum()));
    //     group.bench_with_input(file.file_name().to_string_lossy(), &polygon_set, |b, ps| {
    //         b.iter(|| ps.triangulate_default::<def::VecVecFanBuilder<_>>().expect("Triangulation failed"))
    //     });
    // }

    // group.finish();

    c.bench_function("brazil", |b| b.iter(|| {
        let polygon_set = util::load_polygon_list(util::countries_path().join("brazil.txt").to_str().unwrap());
        polygon_set.triangulate_default::<builders::VecVecIndexedFanBuilder<_>>().expect("Triangulation failed");
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);