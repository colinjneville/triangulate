use std::{error, fmt};

use crate::{FanBuilder, PolygonList, Triangulate, TriangulationError};

use super::util;

#[derive(Debug, PartialEq, Eq)]
enum BuilderError {
    New,
    NewFan,
    ExtendFan,
    Build,
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BuilderError")
    }
}

impl  error::Error for BuilderError { }

struct NewErrorBuilder;

impl<'a, P: PolygonList<'a>> FanBuilder<'a, P> for NewErrorBuilder {
    type Initializer = ();
    type Output = ();
    type Error = BuilderError;

    fn new( _initializer: Self::Initializer,  _polygon_list: P,  _vi0: P::Index,  _vi1: P::Index,  _vi2: P::Index) -> Result<Self, Self::Error>
    where Self: Sized {
        Err(BuilderError::New)
    }

    fn new_fan(&mut self,  _vi0: P::Index,  _vi1: P::Index,  _vi2: P::Index) -> Result<(), Self::Error> {
        Ok(())
    }

    fn extend_fan(&mut self, _vi: P::Index) -> Result<(), Self::Error> {
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn fail(self,  _error: &crate::TriangulationError<Self::Error>) { }
}

struct NewFanErrorBuilder;

impl<'a, P: PolygonList<'a>> FanBuilder<'a, P> for NewFanErrorBuilder {
    type Initializer = ();
    type Output = ();
    type Error = BuilderError;

    fn new( _initializer: Self::Initializer,  _polygon_list: P,  _vi0: P::Index,  _vi1: P::Index,  _vi2: P::Index) -> Result<Self, Self::Error>
    where Self: Sized {
        Ok(Self)
    }

    fn new_fan(&mut self,  _vi0: P::Index,  _vi1: P::Index,  _vi2: P::Index) -> Result<(), Self::Error> {
        Err(BuilderError::NewFan)
    }

    fn extend_fan(&mut self, _vi: P::Index) -> Result<(), Self::Error> {
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn fail(self,  _error: &crate::TriangulationError<Self::Error>) { }
}

struct ExtendFanErrorBuilder;

impl<'a, P: PolygonList<'a>> FanBuilder<'a, P> for ExtendFanErrorBuilder {
    type Initializer = ();
    type Output = ();
    type Error = BuilderError;

    fn new( _initializer: Self::Initializer,  _polygon_list: P,  _vi0: P::Index,  _vi1: P::Index,  _vi2: P::Index) -> Result<Self, Self::Error>
    where Self: Sized {
        Ok(Self)
    }

    fn new_fan(&mut self,  _vi0: P::Index,  _vi1: P::Index,  _vi2: P::Index) -> Result<(), Self::Error> {
        Ok(())
    }

    fn extend_fan(&mut self, _vi: P::Index) -> Result<(), Self::Error> {
        Err(BuilderError::ExtendFan)
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    fn fail(self,  _error: &crate::TriangulationError<Self::Error>) { }
}

struct BuildErrorBuilder;

impl<'a, P: PolygonList<'a>> FanBuilder<'a, P> for BuildErrorBuilder {
    type Initializer = ();
    type Output = ();
    type Error = BuilderError;

    fn new(_initializer: Self::Initializer, _polygon_list: P, _vi0: P::Index, _vi1: P::Index, _vi2: P::Index) -> Result<Self, Self::Error>
    where Self: Sized {
        Ok(Self)
    }

    fn new_fan(&mut self, _vi0: P::Index, _vi1: P::Index, _vi2: P::Index) -> Result<(), Self::Error> {
        Ok(())
    }

    fn extend_fan(&mut self, _vi: P::Index) -> Result<(), Self::Error> {
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::Error> {
        Err(BuilderError::Build)
    }

    fn fail(self,  _error: &crate::TriangulationError<Self::Error>) { }
}

#[test]
fn new_error() {
    if let TriangulationError::FanBuilder(err) = util::polygon::star().triangulate::<NewErrorBuilder>(()).unwrap_err() {
        assert_eq!(err, BuilderError::New.into());
    } else {
        assert!(false);
    }
}

#[test]
fn new_fan_error() {
    if let TriangulationError::FanBuilder(err) = util::polygon::star().triangulate::<NewFanErrorBuilder>(()).unwrap_err() {
        assert_eq!(err, BuilderError::NewFan.into());
    } else {
        assert!(false);
    }
}

#[test]
fn extend_fan_error() {
    if let TriangulationError::FanBuilder(err) = util::polygon::star().triangulate::<ExtendFanErrorBuilder>(()).unwrap_err() {
        assert_eq!(err, BuilderError::ExtendFan.into());
    } else {
        assert!(false);
    }
}

#[test]
fn build_error() {
    if let TriangulationError::FanBuilder(err) = util::polygon::star().triangulate::<BuildErrorBuilder>(()).unwrap_err() {
        assert_eq!(err, BuilderError::Build.into());
    } else {
        assert!(false);
    }
}
