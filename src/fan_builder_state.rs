use std::{fmt, mem};

use crate::{FanFormat, PolygonList, TriangulationError, FanBuilder};


pub(crate) enum FanBuilderState<'p, P: PolygonList<'p> + ?Sized, FB: FanFormat<'p, P>> {
    Uninitialized(FB),
    Initialized(FB::Builder),
    Error(Option<FB::Builder>),
}

impl<'p, P: PolygonList<'p> + ?Sized, FB: FanFormat<'p, P>> FanBuilderState<'p, P, FB> {
    pub(crate) fn new_fan(&mut self, polygon_list: &'p P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<&mut FB::Builder, TriangulationError<<FB::Builder as FanBuilder<'p, P>>::Error>> {
        fn set_initialized<'f, 'pp, P: PolygonList<'pp> + ?Sized, FB: FanFormat<'pp, P>>(s: &'f mut FanBuilderState<'pp, P, FB>, fb: FB::Builder) -> Result<&'f mut FB::Builder, TriangulationError<<FB::Builder as FanBuilder<'pp, P>>::Error>> {
            *s = FanBuilderState::Initialized(fb);
            if let FanBuilderState::Initialized(fb) = s {
                Ok(fb)
            } else {
                Err(TriangulationError::internal("msg"))
            }
        }

        match mem::replace(self, Self::Error(None)) {
            Self::Initialized(mut fb) => {
                match fb.new_fan(vi0, vi1, vi2).map_err(TriangulationError::from) {
                    Ok(_) => set_initialized(self, fb),
                    Err(err) => {
                        *self = Self::Error(Some(fb));
                        Err(err)
                    }
                }
            }
            Self::Uninitialized(fb) => {
                let fb = fb.initialize(polygon_list, vi0, vi1, vi2).map_err(TriangulationError::from)?;
                set_initialized(self, fb)
            }
            Self::Error(_) => Err(TriangulationError::internal("msg")),
        }
    }

    pub(crate) fn complete(self, result: Result<(), TriangulationError<<FB::Builder as FanBuilder<'p, P>>::Error>>) -> Result<<FB::Builder as FanBuilder<'p, P>>::Output, TriangulationError<<FB::Builder as FanBuilder<'p, P>>::Error>> {
        match (self, result) {
            // Success
            (FanBuilderState::Initialized(fb), Ok(())) => fb.build().map_err(Into::into),
            // Failure, before FanBuilder initialized
            (FanBuilderState::Uninitialized(_), result) => Err(result.err().unwrap_or(TriangulationError::NoVertices)),
            (FanBuilderState::Error(None), Err(err)) => Err(err),
            // Failure, after FanBuilder initialized
            (FanBuilderState::Initialized(fb), Err(err)) |
            (FanBuilderState::Error(Some(fb)), Err(err)) => {
                fb.fail(&err);
                Err(err)
            }
            // Something went wrong (result should always be Err for FBS::Error)
            (FanBuilderState::Error(fb), Ok(())) => {
                debug_assert!(false);

                let err = TriangulationError::internal("Unknown error");
                if let Some(fb) = fb {
                    fb.fail(&err);
                }
                Err(err)
            }
            
        }
    }
}

impl<'a, P: PolygonList<'a> + ?Sized, FB: FanFormat<'a, P>> fmt::Debug for FanBuilderState<'a, P, FB> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FanBuilderState::Uninitialized(_) => "FanBuilderState::Uninitialized",
            FanBuilderState::Initialized(_) => "FanBuilderState::Initialized",
            FanBuilderState::Error(_) => "FanBuilderState::Error",
        })
    }
}