use std::{error, fmt};

use crate::{FanFormat, Polygon, PolygonList, TriangulationError, formats, FanBuilder, ListFormat};

use super::util;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BuilderError {
    Initialize,
    NewFan,
    ExtendFan,
    Build,
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BuilderError")
    }
}

impl error::Error for BuilderError { }

struct ErrorFormat<'b>(BuilderError, &'b mut bool);

impl<'b> ErrorFormat<'b> {
    fn new(raised_error: BuilderError, failed_flag: &'b mut bool) -> Self {
        *failed_flag = false;
        Self(raised_error, failed_flag)
    }

    fn check_error(&self, error: BuilderError) -> Result<(), BuilderError> {
        if self.0 == error {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn set_failed_flag(&mut self) {
        *self.1 = true;
    }
}

impl<'p, P: PolygonList<'p> + ?Sized> FanFormat<'p, P> for ErrorFormat<'p> {
    type Builder = Self;

    fn initialize(self, _polygon_list: &'p P, _vi0: <P as PolygonList<'p>>::Index, _vi1: <P as PolygonList<'p>>::Index, _vi2: <P as PolygonList<'p>>::Index) -> Result<Self::Builder, <Self::Builder as FanBuilder<'p, P>>::Error> {
        self.check_error(BuilderError::Initialize)?;
        Ok(self)
    }
}

impl<'p, P: PolygonList<'p> + ?Sized> FanBuilder<'p, P> for ErrorFormat<'p> {
    type Output = ();
    type Error = BuilderError;

    fn new_fan(&mut self, _vi0: <P as PolygonList<'p>>::Index, _vi1: <P as PolygonList<'p>>::Index, _vi2: <P as PolygonList<'p>>::Index) -> Result<(), Self::Error> {
        self.check_error(BuilderError::NewFan)?;
        Ok(())
    }

    fn extend_fan(&mut self, _vi: <P as PolygonList<'p>>::Index) -> Result<(), Self::Error> {
        self.check_error(BuilderError::ExtendFan)?;
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        self.check_error(BuilderError::Build)?;
        Ok(())
    }

    fn fail(mut self, _error: &TriangulationError<Self::Error>) { 
        self.set_failed_flag();
    }
}

#[test]
fn error_propagation() {
    // Ensure Builder-raised errors are propagated back to the original `triangulate` call, and that `FanBuilder::fail` is called if applicable
    for builder_error in [BuilderError::Initialize, BuilderError::NewFan, BuilderError::ExtendFan, BuilderError::Build] {
        let mut failed = false;
        match util::polygon::star().triangulate(ErrorFormat::new(builder_error, &mut failed)).expect_err("Triangulation completed successfully") {
            TriangulationError::FanBuilder(err) => {
                assert_eq!(err, builder_error);
                // For `initialize` and `build`, `fail` will not be called, because the Builder does not yet exist, or has been moved into `build`, respectively
                if builder_error != BuilderError::Initialize && builder_error != BuilderError::Build {
                    assert!(failed);
                }
            },
            err @ _ => panic!("Unexpected non-builder error: {:?}", err),
        }
    }
    
}

#[test]
fn index_wrapper() {
    fn require_u16(_i: u16) { }

    let polygon = util::polygon::star();

    let mut output = Vec::<Vec<_>>::new();
    let builder = formats::IndexedFanFormat::new(&mut output);

    let polygon_ref = &*polygon;
    let polygon_ref = polygon_ref.index_with::<u16>();
    let result = polygon_ref.triangulate(builder).expect("Triangulation failed");
    require_u16(result[0][0]);

    let builder = formats::IndexedFanFormat::new(&mut output);

    let polygon = polygon.index_with::<u16>();
    let result = polygon.triangulate(builder).expect("Triangulation failed");
    require_u16(result[0][0]);
}

#[test]
fn index_wrapper_polygons() {
    fn require_u16(_i: u16) { }

    let polygons = vec![
        vec![[0f32, 0f32], [0., 1.], [1., 1.], [1., 0.]], 
    	vec![[0.05, 0.05], [0.05, 0.95], [0.95, 0.95], [0.95, 0.05]]
    ];

    let mut output = Vec::<Vec<_>>::new();
    let builder = formats::IndexedFanFormat::new(&mut output);

    let polygons_ref = &*polygons;
    let polygons_ref = polygons_ref.index_with::<usize, u16>();
    let result = polygons_ref.triangulate(builder).expect("Triangulation failed");
    require_u16(result[0][0][0]);
    
    let builder = formats::IndexedFanFormat::new(&mut output);

    let polygons = polygons.index_with::<usize, u16>();
    let result = polygons.triangulate(builder).expect("Triangulation failed");
    require_u16(result[0][0][0]);
}

#[test]
fn separate_trapezoidation() {
    let mut output = Vec::<Vec<_>>::new();
    let builder = formats::IndexedFanFormat::new(&mut output);
    let polygon = util::polygon::star();
    let traps = polygon.trapezoidize().expect("Trapezoidation failed");
    traps.triangulate(builder).expect("Triangulation failed");
}

#[test]
fn fan_to_list() {
    let mut output = Vec::<[_; 3]>::new();
    let builder = formats::IndexedListFormat::new(&mut output).into_fan_format();
    let polygon = util::polygon::star();
    let result = polygon.triangulate(builder).expect("Triangulation failed");
    assert_eq!(result.len(), 6);
}

#[test]
fn reverse_winding() {
    let polygon = util::polygon::star();

    let mut output0 = Vec::<Vec<_>>::new();
    let mut output1 = Vec::<Vec<_>>::new();
    let builder0 = formats::IndexedFanFormat::new(&mut output0);
    let builder1 = formats::IndexedFanFormat::new(&mut output1).reverse_winding();
    let result0 = polygon.triangulate(builder0).expect("Triangulation failed");
    let result1 = polygon.triangulate(builder1).expect("Triangulation failed");

    assert_ne!(result0, result1);
}

#[test]
fn deindexed_fan() {
    fn require_f32_2(_i: [f32; 2]) { }

    let polygon = util::polygon::star();

    let mut output = Vec::<Vec<_>>::new();
    let builder = formats::DeindexedFanFormat::new(&mut output);
    let result = polygon.triangulate(builder).expect("Triangulation failed");

    require_f32_2(result[0][0]);
}

#[test]
fn deindexed_list() {
    fn require_f32_2(_i: [f32; 2]) { }

    let polygon = util::polygon::star();

    let mut output = Vec::<[f32; 2]>::new();
    let builder = formats::DeindexedListFormat::new(&mut output).into_fan_format();
    let result = polygon.triangulate(builder).expect("Triangulation failed");

    require_f32_2(result[0]);
}

#[test]
fn delimited_fan() {
    let delimiter = usize::MAX;

    let polygon = util::polygon::half_frame();

    let mut output = Vec::new();
    let builder = formats::DelimitedFanFormat::new(&mut output, delimiter);
    polygon.triangulate(builder).expect("Triangulation failed");

    assert!(output.into_iter().filter(|i| *i == delimiter).count() > 0);
}
