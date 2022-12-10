use crate::{formats, ListFormat, Polygon};

#[test]
// Prevous solution to incomplete monotone triangulation caused a non-terminating loop in rare cases
fn france_nonterminating() {
    // Some vertices commented to make minimal test case
    let polygon = vec![
        [-1.189062499999949, 45.16147460937498],
        [-0.691113281249926, 45.09345703124998],
        // [-0.633984374999926, 45.04711914062497],
        // [-0.548486328124994, 45.00058593749998],
        [6.724707031250006, 44.97299804687506],
        // [6.73818359375008, 44.92138671875003],
        // [6.801074218750017, 44.883154296875034],
        // [6.88935546875004, 44.86030273437501],
        // [6.939843750000023, 44.858740234375034],
        // [6.972851562500011, 44.84501953124999],
        [6.992675781250057, 44.82729492187502],
        [-1.15288085937496, 44.764013671875006],
        // [-1.200390624999955, 44.726464843749994],
        // [-1.2203125, 44.68662109374998],
        [-1.24521484374992, 44.66669921875001],
    ];
    let mut output = Vec::<usize>::new();
    let format = formats::IndexedListFormat::new(&mut output).into_fan_format();
    polygon.triangulate(format).expect("Triangulation failed");
}