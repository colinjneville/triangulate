# triangulate

Subdivides a set of non-self-intersecting polygons into a set of non-overlapping triangles. Inputs and outputs can be customized to use your required formats to avoid unnecessary conversions.

## Usage

```
// A hollow square shape
//  ________
// |  ____  |
// | |    | |
// | |____| |
// |________|
let polygons: Vec<Vec<MyVert>> = vec![
	vec![(0., 0.).into(), (0., 1.).into(), (1., 1.).into(), (1., 0.).into()], 
	vec![(0.05, 0.05).into(), (0.05, 0.95).into(), (0.95, 0.95).into(), (0.95, 0.05).into()]
];
// `output` is an arbitrary triangulation of polygons in a format determined by the type parameter (in this case, a `Vec` of triangle fans represented by a `Vec` of the `MyVert` vertices).
let output: Vec<Vec<MyVert>> = polygons.triangulate_default::<builders::VecVecFanBuilder<_>>().unwrap();
```
	
## Input traits

* `VertexIndex`  
Any type which is `Eq` and `Clone`.

* `Vertex`  
A two-dimensional point. 
The coordinate type must implement [`num_traits::real::Real`](https://docs.rs/num-traits/*/num_traits/real/trait.Real.html), reexported as `triangulate::Real`.

* `PolygonList`  
A collection of zero or more polygons which can iterate values convertable to `enum PolygonVertex<T: VertexIndex>`.  
`PolygonVertex::ContinuePolygon` provides the next index in the current polygon and `PolygonVertex::NewPolygon` indicates the end of the current polygon, and subsequent indices belong to a separate polygon. Indices must be ordered by adjacency, but either clockwise or counter-clockwise ordering is accepted.  
Pre-implemented for `Vec<Vec<T>>` (multiple polygons) and `Vec<T>` (single polygon). Wrappers `IndexWithU16`, `IndexWithU16U16`, `IndexWithU32` and `IndexWithU32U32` are also provided to convert `PolygonList` implementations that use `usize` and `(usize, usize)` as indices to use `u16`/`(u16, u16)` and `u32`/`(u32, u32)` for performance reasons when the list of vertices is small enough.

## Output traits

* `FanBuilder`  
Streams the resulting triangulation into the desired output format.  
Triangles are provided as [fans](https://en.wikipedia.org/wiki/Triangle_fan), where a callback to `extend_fan` adds a new vertex (and correspondingly a new triangle) to the existing fan, while `extend_fan` begins a new fan. These callbacks provide `VertexIndex`s, so if the output requires `Vertex`s, the builder is responsible for indexing into the `PolygonList`.  
Since triangle fans are not useful for most applications, also included is the `FanToListAdapter`, which allows you to use a `ListBuilder` as a `FanBuilder`. Like the name implies, `ListBuilder` works with triangle lists, where each included triangle is specified by all 3 vertex indices.  
The output of the included builders is described below:
 * `VecVecIndexedFanBuilder`  
 A vector of vectors of indices. Each inner vector is a single triangle fan, with the first element being the 'central' vertex.
```
let output: Vec<Vec<usize>> = vec![
    vec![0, 1, 2, 3, 4],
    vec![2, 3, 5]
];
```
 * `VecVecFanBuilder`  
A vector of vectors of vertices. Like `VecVecIndexedFanBuilder`, but the indices have been used to clone the vertices' values.
```
let output: Vec<Vec<Vec2>> = vec![
    vec![Vec2::new(0., 0.), Vec2::new(1., 0.), Vec2::new(1., 0.5), Vec2::new(0.5, 1.), Vec2::new(0., 1.)],
    vec![Vec2::new(0., 0.), Vec2::new(0., 1.), Vec2::new(-1., 2.)]
];
```
 * `VecDelimitedIndexedFanBuilder`  
A flat vector of indices, where fans are delimited by a given sentinel value.  
```
let sentinel: usize = usize::MAX;
let output: Vec<usize> = vec![0, 1, 2, 3, 4, sentinel, 2, 3, 5];
```
 * `VecIndexedListBuilder`  
A flat vector of indices. Each chunk of 3 vertices represents 1 triangle.
```
let output: Vec<usize> = vec![0, 1, 2, 0, 2, 3, 1, 3, 4];
```
 * `VecListBuilder`  
Like `VecIndexedListBuilder`, but with vertices deindexed.
```
let output: Vec<Vec2> = vec![Vec2::new(0., 0.), Vec2::new(1., 0.), Vec2::new(1., 1.), Vec2::new(0., 0.), Vec2::new(1., 1.), Vec2::new(0., 1.), Vec2::new(0., 0.), Vec2::new(0., 1.), Vec2::new(-1., 2.)];
```


## Preconditions  
* No edge can cross any other edge, whether it is on the same polygon or not.
* Each vertex must be part of exactly two edges. Polygons cannot 'share' vertices with each other.
* Each vertex must be distinct - no vertex can have x and y coordinates that both compare equal to another vertex's.
These preconditions are not explicitly checked, but an invalid polygon set will likely yield `TriangulationError::InternalError`.

## Results
Because the algorithm involves random ordering, the exact triangulation is not guaranteed to be same between invocations.

## Algorithm
This library is based on [Raimund Seidel's randomized algorithm for triangulating polygons](https://www.cs.princeton.edu/courses/archive/fall05/cos528/handouts/A%20Simple%20and%20fast.pdf). The expected runtime for each polygon or hole with *n* vertices is O(*n* [log\*](https://en.wikipedia.org/wiki/Iterated_logarithm) *n*), a near-linear runtime.