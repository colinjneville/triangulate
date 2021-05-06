mod polygon;
pub use polygon::{PolygonList, Triangulate, TriangulateDefault, PolygonVertex, IndexWithU16, IndexWithU16U16, IndexWithU32, IndexWithU32U32, IndexWithIter};
pub(crate) use polygon::PolygonListExt;
mod vertex;
pub use vertex::{Vertex, VertexIndex};
pub(crate) use vertex::VertexExt;