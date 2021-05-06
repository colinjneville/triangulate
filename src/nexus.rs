use std::marker::PhantomData;

use zot::{Ot, Zot};

use crate::{PolygonList, PolygonListExt, Vertex, VertexIndex, errors::TriangulationInternalError, idx::{Idx, IdxDisplay}, segment::Segment, trapezoid::Trapezoid};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DividerDirection {
    Ascending,
    Descending,
}

#[derive(Debug)]
struct Divider<V: Vertex, Index: VertexIndex> {
    si: Idx<Segment<V, Index>>,
    ti_right: Idx<Trapezoid<V, Index>>,
    direction: DividerDirection,
}

impl<V: Vertex, Index: VertexIndex> Divider<V, Index> {
    pub fn new(si: Idx<Segment<V, Index>>, ti_right: Idx<Trapezoid<V, Index>>, direction: DividerDirection) -> Self {
        Self {
            si, 
            ti_right,
            direction,
        }
    }
}

impl<V: Vertex, Index: VertexIndex> std::fmt::Display for Divider<V, Index> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.si, self.ti_right)
    }
}

pub(crate) enum FinalNexusType<V: Vertex, Index: VertexIndex> {
    V { ti_upleft: Idx<Trapezoid<V, Index>>, ti_upcenter: Idx<Trapezoid<V, Index>>, ti_upright: Idx<Trapezoid<V, Index>>, ti_down: Idx<Trapezoid<V, Index>> },
    I { ti_upleft: Idx<Trapezoid<V, Index>>, ti_upright: Idx<Trapezoid<V, Index>>, ti_downleft: Idx<Trapezoid<V, Index>>, ti_downright: Idx<Trapezoid<V, Index>> },
    A { _ti_up: Idx<Trapezoid<V, Index>>, ti_downleft: Idx<Trapezoid<V, Index>>, ti_downcenter: Idx<Trapezoid<V, Index>>, ti_downright: Idx<Trapezoid<V, Index>> },
}

#[derive(Debug)]
pub(crate) struct Nexus<V: Vertex, Index: VertexIndex> {
    vi: Index,
    ti_upleft: Idx<Trapezoid<V, Index>>,
    ti_downleft: Idx<Trapezoid<V, Index>>,
    dividers: Zot<Divider<V, Index>>,
    _v: PhantomData<V>,
}

impl<V: Vertex + std::fmt::Display, Index: VertexIndex + std::fmt::Display> std::fmt::Display for Nexus<V, Index> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ti_upleft)?;
        for div in self.filter_dividers(DividerDirection::Ascending).iter() {
            write!(f, "[{}]{}", div.si, div.ti_right)?;
        }
        writeln!(f)?;
        writeln!(f, "--{}--", &self.vi)?;
        write!(f, "{}", self.ti_downleft)?;
        for div in self.filter_dividers(DividerDirection::Descending).iter() {
            write!(f, "[{}]{}", div.si, div.ti_right)?;
        }
        Ok(())
    }
}

impl<V: Vertex, Index: VertexIndex> IdxDisplay for Nexus<V, Index> {
    fn fmt(f: &mut std::fmt::Formatter<'_>, idx: usize) -> std::fmt::Result {
        write!(f, "n{}", idx)
    }
}

impl<V: Vertex, Index: VertexIndex> Nexus<V, Index> {
    pub fn new(vi: Index, ti_up: Idx<Trapezoid<V, Index>>, ti_down: Idx<Trapezoid<V, Index>>) -> Self {
        Self {
            vi,
            ti_upleft: ti_up,
            ti_downleft: ti_down,
            dividers: Zot::Zero,
            _v: PhantomData,
        }
    }

    pub fn final_type(&self) -> Result<FinalNexusType<V, Index>, TriangulationInternalError> {
        if let Zot::Two(div0, div1) = &self.dividers {
             Ok(
                 if div0.direction == DividerDirection::Descending {
                    FinalNexusType::A {
                        _ti_up: self.ti_upleft,
                        ti_downleft: self.ti_downleft,
                        ti_downcenter: div0.ti_right,
                        ti_downright: div1.ti_right,
                    }
                } else if div1.direction == DividerDirection::Ascending {
                    FinalNexusType::V {
                        ti_upleft: self.ti_upleft,
                        ti_upcenter: div0.ti_right,
                        ti_upright: div1.ti_right,
                        ti_down: self.ti_downleft,
                    }
                } else {
                    FinalNexusType::I {
                        ti_upleft: self.ti_upleft,
                        ti_upright: div0.ti_right,
                        ti_downleft: self.ti_downleft,
                        ti_downright: div1.ti_right,
                    }
                }
            )
        } else {
            Err(TriangulationInternalError::new("Nexus does not have two joined segments"))
        }
    }

    pub fn vertex(&self) -> Index { self.vi.clone() }

    pub fn replace_trapezoid(&mut self, ti_old: Idx<Trapezoid<V, Index>>, ti_new: Idx<Trapezoid<V, Index>>) -> Result<(), TriangulationInternalError> {
        *self.find_trapezoid(ti_old).ok_or_else(|| TriangulationInternalError::new(format!("Trapezoid {} is not connected to replace with {}", ti_old, ti_new)))? = ti_new;
        Ok(())
    }

    fn find_trapezoid(&mut self, ti: Idx<Trapezoid<V, Index>>) -> Option<&mut Idx<Trapezoid<V, Index>>> {
        if self.ti_upleft == ti {
            Some(&mut self.ti_upleft)
        } else if self.ti_downleft == ti {
            Some(&mut self.ti_downleft)
        } else {
            match &mut self.dividers {
                Zot::One(div0) if div0.ti_right == ti => Some(&mut div0.ti_right),
                Zot::Two(div0, _) if div0.ti_right == ti => Some(&mut div0.ti_right),
                Zot::Two(_, div1) if div1.ti_right == ti => Some(&mut div1.ti_right),
                _ => None,
            }
        }
    }

    pub fn add_segment<'a, P: PolygonList<'a, Vertex=V, Index=Index>>(ns: &mut [Nexus<V, Index>], ps: PolygonListExt<'a, P>, ss: &[Segment<V, Index>], ni: Idx<Nexus<V, Index>>, si: Idx<Segment<V, Index>>, ti_right: Idx<Trapezoid<V, Index>>) -> Result<(), TriangulationInternalError> {
        let dir = Self::get_segment_direction(ss, ni, si)?;
        let div = Divider::new(si, ti_right, dir);
        let mut divs = Zot::Zero;
        std::mem::swap(&mut divs, &mut ns[ni].dividers);
        ns[ni].dividers = match divs {
            Zot::Zero => Zot::One(div),
            Zot::One(div0) => {
                let swap = if div0.direction == div.direction {
                    let s0 = &ss[div0.si];
                    let s1 = &ss[div.si];
                    let ni1 = match div.direction {
                        DividerDirection::Ascending => s1.ni_max(),
                        DividerDirection::Descending => s1.ni_min(),
                    };
                    s0.is_on_left(ps, ns, ns[ni1].vertex())
                } else {
                    div.direction == DividerDirection::Ascending
                };
                if swap {
                    Zot::Two(div, div0)
                } else {
                    Zot::Two(div0, div)
                }
            },
            Zot::Two(div0, div1) => return Err(TriangulationInternalError::new(format!("Nexus already has two segments ({}, {})", div0, div1))),
        };
        Ok(())
    }

    fn get_segment_direction(ss: &[Segment<V, Index>], ni: Idx<Nexus<V, Index>>, si: Idx<Segment<V, Index>>) -> Result<DividerDirection, TriangulationInternalError> {
        let s = &ss[si];
        if ni == s.ni_max() {
            Ok(DividerDirection::Descending)
        } else if ni == s.ni_min() {
            Ok(DividerDirection::Ascending)
        } else {
            Err(TriangulationInternalError::new(format!("{} does not contain {}", si, ni)))
        }
    }

    fn filter_dividers(&self, direction: DividerDirection) -> Zot<&Divider<V, Index>> {
        let a = self.dividers.first().filter(|d| d.direction == direction);
        let b = self.dividers.second().filter(|d| d.direction == direction);
        Zot::from_options(a, b)
    }

    fn filter_trapezoids(&self, direction: DividerDirection) -> Zot<Idx<Trapezoid<V, Index>>> {
        self.filter_dividers(direction).map(|d| d.ti_right)
    }

    pub fn up_trapezoids(&self) -> Ot<Idx<Trapezoid<V, Index>>> {
        (self.ti_upleft, self.filter_trapezoids(DividerDirection::Ascending).last().copied()).into()
    }

    pub fn down_trapezoids(&self) -> Ot<Idx<Trapezoid<V, Index>>> {
        (self.ti_downleft, self.filter_trapezoids(DividerDirection::Descending).last().copied()).into()
    }

    pub fn iter_up_trapezoids(&self) -> impl Iterator<Item=Idx<Trapezoid<V, Index>>> + '_ {
        NexusTrapezoidIter::new(&self, DividerDirection::Ascending)
    }

    pub fn iter_down_trapezoids(&self) -> impl Iterator<Item=Idx<Trapezoid<V, Index>>> + '_ {
        NexusTrapezoidIter::new(&self, DividerDirection::Descending)
    }

    pub fn get_down_trapezoid_in_direction<'a, P: PolygonList<'a, Vertex=V, Index=Index>>(&self, ps: PolygonListExt<'a, P>, ns: &[Nexus<V, Index>], ss: &[Segment<V, Index>], s: &Segment<V, Index>) -> Result<Idx<Trapezoid<V, Index>>, TriangulationInternalError> {
        match self.filter_dividers(DividerDirection::Descending) {
            Zot::Zero => Ok(self.ti_downleft),
            Zot::One(div_r) |
            Zot::Two(_, div_r) => {
                let vi = if ns[s.ni_min()].vertex() == self.vertex() {
                    return Err(TriangulationInternalError::new("Invalid segment/nexus connection"))
                } else if ns[s.ni_max()].vertex() == self.vertex() {
                    ns[ss[div_r.si].ni_min()].vertex()
                } else {
                    self.vertex()
                };
                if s.is_on_left(ps, ns, vi) {
                    Ok(div_r.ti_right)
                } else {
                    Ok(self.ti_downleft)
                }
            }
        }
    }
}

struct NexusTrapezoidIter<'a, V: Vertex, Index: VertexIndex> {
    parent: &'a Nexus<V, Index>,
    dir: DividerDirection,
    state: u8,
}

impl<'a, V: Vertex, Index: VertexIndex> NexusTrapezoidIter<'a, V, Index> {
    fn new(parent: &'a Nexus<V, Index>, dir: DividerDirection) -> Self {
        Self {
            parent,
            dir,
            state: 0,
        }
    }
}

impl<'a, V: Vertex, Index: VertexIndex> Iterator for NexusTrapezoidIter<'a, V, Index> {
    type Item = Idx<Trapezoid<V, Index>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ti = if self.state == 0 {
            match self.dir {
                DividerDirection::Ascending => Some(self.parent.ti_upleft),
                DividerDirection::Descending => Some(self.parent.ti_downleft),
            }
        } else {
            match &self.parent.dividers {
                Zot::One(div0) if self.state == 1 && div0.direction == self.dir => Some(div0.ti_right),
                Zot::Two(div0, _) if self.state == 1 && div0.direction == self.dir => Some(div0.ti_right),
                Zot::Two(_, div1) if self.state >= 1 && self.state <= 2 && div1.direction == self.dir => { 
                    // If we skipped here from state == 1, make sure we don't repeat this div again
                    self.state = 2;
                    Some(div1.ti_right)
                }
                _ => None,
            }
        };
        if ti.is_some() {
            self.state += 1;
        }
        ti
    }
}