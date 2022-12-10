# triangulate
Subdivides a set of non-self-intersecting polygons into a set of non-overlapping triangles. 
Inputs and outputs can be customized to use your required formats to avoid unnecessary conversions.

Note: While the big-O measure of the algorithm this crate uses is very good, 
in practice the constant factors mean it will run slower than crates such as [earcutr](https://crates.io/crates/earcutr),
unless the polygons are *extremely* large.


```
# use triangulate::formats;
# use triangulate::{PolygonList, ListFormat};

// A hollow square shape
//  ________
// |  ____  |
// | |    | |
// | |____| |
// |________|
let polygons = vec![
    vec![[0f32, 0f32], [0., 1.], [1., 1.], [1., 0.]], 
    vec![[0.05, 0.05], [0.05, 0.95], [0.95, 0.95], [0.95, 0.05]]
];
let mut triangulated_indices = Vec::<[usize; 2]>::new();
polygons.triangulate(formats::IndexedListFormat::new(&mut triangulated_indices).into_fan_builder()).expect("Triangulation failed");
println!("First triangle: {:?}, {:?}, {:?}", 
    polygons.get_vertex(triangulated_indices[0]), 
    polygons.get_vertex(triangulated_indices[1]), 
    polygons.get_vertex(triangulated_indices[2]));
```

Any type that implements [Polygon] or [PolygonList] can be triangulated. Most commonly that would be [Vec<_>] or [Vec<Vec<_>>] (where `_`: [Vertex], such as `[f32; 2]`), 
but you can implement the trait on your own types.

The output format is also customizable. [PolygonList::triangulate] takes a [FanFormat], which determines the resulting output. 
Most commonly, you would want [formats::IndexedListFormat], which stores three indices for every generated triangle in a [List] (like [Vec]).
However, this is a [ListFormat] (it takes individual triangles instead of triangle fans), so it must be converted to a [FanFormat] by calling [ListFormat::into_fan_format] (see the example above).
Another useful format is [formats::DeindexedListFormat], which deindexes each triangle point to create a [List] of the actual vertices.

## Input traits
* [Vertex]
* [VertexIndex]
* [Polygon]
* [PolygonList]
## Output traits
* [Fan]
* [Fans]
* [List]
* [FanFormat]
* [FanBuilder]
* [ListFormat]
* [ListBuilder]

## Preconditions  
* No edge can cross any other edge, whether it is on the same polygon or not.
* Each vertex must be part of exactly two edges. Polygons cannot 'share' vertices with each other.
* Each vertex must be distinct - no vertex can have x and y coordinates that both compare equal to another vertex's.

These preconditions are not explicitly checked, but an invalid polygon set will likely yield `TriangulationError::InternalError`.

## Results
Because the algorithm involves random ordering, the exact triangulation is not guaranteed to be same between invocations.

## Algorithm
This library is based on [Raimund Seidel's randomized algorithm for triangulating polygons](https://www.cs.princeton.edu/courses/archive/fall05/cos528/handouts/A%20Simple%20and%20fast.pdf). 
The expected runtime for each polygon or hole with *n* vertices is O(*n* [log\*](https://en.wikipedia.org/wiki/Iterated_logarithm) *n*), a near-linear runtime.