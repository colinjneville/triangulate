use std::{fmt, mem};

use crate::{FanBuilder, PolygonList, TriangulationError};


pub(crate) enum FanBuilderState<'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>> {
    Uninitialized(FB::Initializer),
    Initialized(FB),
    Error(Option<FB>),
}

impl<'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>> FanBuilderState<'a, P, FB> {
    pub(crate) fn new_fan(&mut self, polygon_list: P, vi0: P::Index, vi1: P::Index, vi2: P::Index) -> Result<&mut FB, TriangulationError<FB::Error>> {
        fn set_initialized<'b, 'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>>(s: &'b mut FanBuilderState<'a, P, FB>, fb: FB) -> Result<&'b mut FB, TriangulationError<FB::Error>> {
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
            Self::Uninitialized(initializer) => {
                let fb = FB::new(initializer, polygon_list, vi0, vi1, vi2).map_err(TriangulationError::from)?;
                set_initialized(self, fb)
            }
            Self::Error(_) => Err(TriangulationError::internal("msg")),
        }
    }

    pub(crate) fn complete(self, result: Result<(), TriangulationError<FB::Error>>) -> Result<FB::Output, TriangulationError<FB::Error>> {
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

impl<'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>> From<FanBuilderState<'a, P, FB>> for Option<FB> {
    fn from(fbs: FanBuilderState<'a, P, FB>) -> Self {
        match fbs {
            FanBuilderState::Uninitialized(_) => None,
            FanBuilderState::Initialized(fb) => Some(fb),
            FanBuilderState::Error(ofb) => ofb,
        }
    }
}

impl<'z, 'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>> From<&'z mut FanBuilderState<'a, P, FB>> for Option<&'z mut FB> {
    fn from(fbs: &'z mut FanBuilderState<'a, P, FB>) -> Self {
        match fbs {
            FanBuilderState::Uninitialized(_) => None,
            FanBuilderState::Initialized(fb) => Some(fb),
            FanBuilderState::Error(ofb) => ofb.as_mut(),
        }
    }
}

impl<'a, P: PolygonList<'a>, FB: FanBuilder<'a, P>> fmt::Debug for FanBuilderState<'a, P, FB> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FanBuilderState::Uninitialized(_) => "FanBuilderState::Uninitialized",
            FanBuilderState::Initialized(_) => "FanBuilderState::Initialized",
            FanBuilderState::Error(_) => "FanBuilderState::Error",
        })
    }
}