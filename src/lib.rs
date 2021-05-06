mod idx;
mod segment;
mod trapezoid;
mod querynode;
mod trapezoidation;
mod nexus;
mod monotone;
mod math;
mod fan_builder_state;
mod inputs;
mod outputs;
#[macro_use]
mod errors;

#[cfg(feature = "debugging")]
pub mod debug;

#[cfg(any(test, feature = "benchmarking"))]
pub mod tests;

pub use errors::TriangulationError;
use trapezoidation::TrapezoidationState;

pub(crate) use fan_builder_state::FanBuilderState;

pub use inputs::*;
pub use outputs::*;

pub use num_traits::real::Real;

fn do_triangulate<'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>>(polygon_list: PolygonListExt<'a, P>, initializer: FB::Initializer) -> Result<FB::Output, TriangulationError<FB::Error>> {
    TrapezoidationState::new(polygon_list).build()?.triangulate::<FB>(initializer)
}
