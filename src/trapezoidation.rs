use std::{error, iter};

use rand::prelude::SliceRandom;
use zot::Ot;
use crate::{FanBuilder, FanBuilderState, PolygonList, PolygonListExt, PolygonVertex, Vertex, VertexIndex, errors::{TriangulationError, TriangulationInternalError}, idx::{Idx, VecExt, SliceExt}, math::math_n, monotone::MonotoneBuilder, nexus::{FinalNexusType, Nexus}, querynode::{QueryNode, QueryNodeBranch}, segment::Segment, trapezoid::Trapezoid};

#[cfg(feature = "debugging")]
use std::{marker, fmt};
#[cfg(feature = "debugging")]
use crate::{debug, monotone::Monotone, VertexExt};
#[cfg(feature = "debugging")]
use num_traits::ToPrimitive;

trait TrapezoidationStructure<'a, P: PolygonList<'a>> {
    fn ps(&self) -> PolygonListExt<'a, P>;
    fn ns(&self) -> &[Nexus<P::Vertex, P::Index>];
    fn ss(&self) -> &[Segment<P::Vertex, P::Index>];
    fn ts(&self) -> &[Trapezoid<P::Vertex, P::Index>];
    fn qs(&self) -> &[QueryNode<P::Vertex, P::Index>];

    fn query_node_root(&self) -> Idx<QueryNode<P::Vertex, P::Index>> {
        Idx::new(0)
    }

    fn find_trapezoid(&self, vi: P::Index) -> (Idx<QueryNode<P::Vertex, P::Index>>, Idx<Trapezoid<P::Vertex, P::Index>>) {
        self.find_trapezoid_from_root(vi, self.query_node_root())
    }

    fn find_trapezoid_from_root(&self, vi: P::Index, qi_root: Idx<QueryNode<P::Vertex, P::Index>>) -> (Idx<QueryNode<P::Vertex, P::Index>>, Idx<Trapezoid<P::Vertex, P::Index>>) {
        let mut qi = qi_root;
        loop {
            match &self.qs()[qi] {
                QueryNode::Branch(left, right, branch) => {
                    let use_left = match branch {
                        // The right trapezoid will be chosen if the vertex is one of the edge's endpoints
                        QueryNodeBranch::X(si) => self.ss()[*si].is_on_left(self.ps(), self.ns(), vi.clone()),
                        // Choose the lower trapezoid if this corresponds to an existing vertex (to make horizontal splitting easier)
                        QueryNodeBranch::Y(ni_y) => self.ps()[vi.clone()] <= self.ps()[self.ns()[*ni_y].vertex()],
                    };
                    qi = if use_left { *left } else { *right };
                },
                QueryNode::Sink(ti) => return (qi, *ti),
            }
        }
    }
}

#[cfg(feature = "debugging")]
fn trapezoidation_fmt<'a, P: PolygonList<'a>, T: TrapezoidationStructure<'a, P>>(w: &mut impl std::io::Write, trapezoidation: &T) -> std::io::Result<()> {
    writeln!(w, "nexuses:")?;
    for (i, n) in trapezoidation.ns().iter().enumerate() {
        writeln!(w, "{}:", Idx::<Nexus<P::Vertex, P::Index>>::new(i))?;
        writeln!(w, "{:?}", n)?;
    }

    writeln!(w, "segments:")?;
    write!(w, "[")?;
    if trapezoidation.ss().len() > 0 {
        write!(w, "{:?}", trapezoidation.ss()[0])?;
    }
    for s in trapezoidation.ss().iter().skip(1) {
        write!(w, ", {:?}", s)?;
    }
    writeln!(w, "]")?;

    writeln!(w, "query structure:")?;
    writeln!(w, "{}", trapezoidation.qs()[trapezoidation.query_node_root()].as_text_tree(trapezoidation.query_node_root(), trapezoidation.qs()))?;

    writeln!(w, "trapezoids:")?;
    for (i, t) in trapezoidation.ts().iter().enumerate() {
        writeln!(w, "{}:", Idx::<Trapezoid<P::Vertex, P::Index>>::new(i))?;
        writeln!(w, "{}", t)?;
    }
    Ok(())
}

enum QueryNodeOrNexus<V: Vertex, Index: VertexIndex> {
    QueryNode(Index, Idx<QueryNode<V, Index>>),
    Nexus(Idx<Nexus<V, Index>>),
}

impl<V: Vertex, Index: VertexIndex> Clone for QueryNodeOrNexus<V, Index> {
    fn clone(&self) -> Self {
        match self {
            QueryNodeOrNexus::QueryNode(index, qi) => QueryNodeOrNexus::QueryNode(index.clone(), *qi),
            QueryNodeOrNexus::Nexus(ni) => QueryNodeOrNexus::Nexus(*ni),
        }
    }
}

pub(crate) struct TrapezoidationState<'a, P: PolygonList<'a>> {
    ps: PolygonListExt<'a, P>,
    ns: Vec<Nexus<P::Vertex, P::Index>>,
    ss: Vec<Segment<P::Vertex, P::Index>>,
    ts: Vec<Trapezoid<P::Vertex, P::Index>>,
    qs: Vec<QueryNode<P::Vertex, P::Index>>,
    #[cfg(feature = "debugging")]
    svg_context: Option<debug::svg::SvgContext>,
    #[cfg(feature = "debugging")]
    current_step: u32,
    #[cfg(feature = "debugging")]
    current_substep: u32,
}

impl<'a, P: PolygonList<'a>> TrapezoidationState<'a, P> {
    pub fn new(ps: PolygonListExt<'a, P>) -> Self {
        let vertex_count = ps.vertex_count();

        // TODO What is the upper bound for number of query nodes?
        // Just allocate a large amount for now
        let mut qs = Vec::with_capacity(vertex_count * 4);
        let ti = Idx::new(0);
        let q = QueryNode::root(ti);
        let qi = qs.push_get_index(q);
        let t = Trapezoid::all(qi);

        let mut ts = Vec::with_capacity(vertex_count * 2 + 1);
        ts.push(t);

        #[cfg(feature = "debugging")]
        let svg_context = Self::svg_context(&ps);

        Self {
            ps,
            ns: Vec::with_capacity(vertex_count),
            ss: Vec::with_capacity(vertex_count),
            ts,
            qs,
            #[cfg(feature = "debugging")]
            svg_context,
            #[cfg(feature = "debugging")]
            current_step: 0,
            #[cfg(feature = "debugging")]
            current_substep: 0,
        }
    }

    #[cfg(feature = "debugging")]
    fn svg_context(ps: &PolygonListExt<'a, P>) -> Option<debug::svg::SvgContext> {
        let output_path = debug::env::svg::output_path()?;
        let output_level = debug::env::svg::output_level();
        let show_labels = debug::env::svg::show_labels();

        if output_level == debug::svg::SvgOutputLevel::None {
            return None;
        }

        let min_value = f32::MIN;
        let max_value = f32::MAX;

        let mut view_x_min = max_value;
        let mut view_x_max = min_value;
        let mut view_y_min = max_value;
        let mut view_y_max = min_value;
        for index in ps.iter_polygon_vertices() {
            let index: PolygonVertex<_> = index.into();
            if let PolygonVertex::ContinuePolygon(index) = index {
                let v = &ps[index];
                view_x_min = view_x_min.min(v.x().to_f32().unwrap());
                view_x_max = view_x_max.max(v.x().to_f32().unwrap());
                view_y_min = view_y_min.min(v.y().to_f32().unwrap());
                view_y_max = view_y_max.max(v.y().to_f32().unwrap());
            }
        }

        let w = view_x_max - view_x_min;
        let h = view_y_max - view_y_min;
        let margin_scale = 0.1;
        view_x_min -= w * margin_scale;
        view_x_max += w * margin_scale;
        view_y_min -= h * margin_scale;
        view_y_max += h * margin_scale;

        Some(debug::svg::SvgContext {
            output_path, 
            output_level,
            view_x_min, 
            view_x_max, 
            view_y_min, 
            view_y_max,
            show_labels,
        })
    }

    #[cfg(feature = "debugging")]
    fn output_svg(&mut self, style: debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>, level: debug::svg::SvgOutputLevel) {
        if let Some(svg_context) = &self.svg_context {
            if svg_context.output_level >= level {
                // Make the directory for this step if this is the first svg
                if self.current_substep == 0 {
                    let path = svg_context.output_path.join(format!("{:03}", self.current_step));
                    if std::fs::create_dir(path).is_err() {
                        return;
                    }
                }

                let mut svg = debug::svg::SvgOutput::new(&svg_context, style);
                let _ = svg.append_element(self, &());
                
                let path: std::path::PathBuf = format!("{:03}", self.current_step).into();
                let path = path.join(format!("{:03}.svg", self.current_substep));
                let _ = svg.save(path);

                self.current_substep += 1;
            }
        }
    }

    #[cfg(feature = "debugging")]
    fn advance_step(&mut self) {
        if let Some(svg_context) = &self.svg_context {
            if svg_context.output_level >= debug::svg::SvgOutputLevel::MajorSteps {
                let path = svg_context.output_path.join(format!("{:03}", self.current_step)).join("state.txt");
                if let Ok(f) = std::fs::File::create(path) {
                    let mut w = std::io::BufWriter::new(&f);
                    let _ = trapezoidation_fmt(&mut w, self);
                }

                self.current_step += 1;
                self.current_substep = 0;
            }
        }
    }

    pub fn build<FBError: error::Error>(mut self) -> Result<Trapezoidation<'a, P>, TriangulationError<FBError>> {
        // Track the best-known location of each vertex. Initially, all we have is the root QueryNode.
        // Periodically, for each uninserted vertex, we search for the trapezoid that contains the point and update the QueryNode.
        // Finally, once a vertex is inserted, we replace the QueryNode with the exact Nexus we created for the vertex

        // Allocate as if there is a single polygon (ensuring no reallocations)
        let mut q_lookup: Vec<QueryNodeOrNexus<P::Vertex, P::Index>> = Vec::with_capacity(self.ps.vertex_count());

        // Ensure the iteration ends with NewPolygon
        for polygon_vertex in self.ps.iter_polygon_vertices().map(Into::into).chain(iter::once(PolygonVertex::NewPolygon)) {
            match polygon_vertex {
                PolygonVertex::ContinuePolygon(index) => {
                    q_lookup.push(QueryNodeOrNexus::QueryNode(index, self.query_node_root()));
                }
                PolygonVertex::NewPolygon => {
                    let v_count = q_lookup.len();
                    if v_count > 0 {
                        if v_count < 3 {
                            return Err(TriangulationError::NotEnoughVertices(v_count));
                        } else {
                            self.add_polygon(q_lookup.as_mut_slice()).map_err(TriangulationError::InternalError)?;
                            q_lookup.clear();
                        }
                    }
                }
            }
        }

        Ok(Trapezoidation::new(self))
    }

    fn add_polygon(&mut self, q_lookup: &mut [QueryNodeOrNexus<P::Vertex, P::Index>]) -> Result<(), TriangulationInternalError> {
        fn add_nth_segment<'a, P: PolygonList<'a>>(state: &mut TrapezoidationState<'a, P>, qons: &mut [QueryNodeOrNexus<P::Vertex, P::Index>], si: usize) -> Result<(), TriangulationInternalError> {
            fn add_vertex<'a, P: PolygonList<'a>>(state: &mut TrapezoidationState<'a, P>, qon: &mut QueryNodeOrNexus<P::Vertex, P::Index>, index: P::Index, qi: Idx<QueryNode<P::Vertex, P::Index>>) -> Result<Idx<Nexus<P::Vertex, P::Index>>, TriangulationInternalError> {
                let ni = state.add_vertex(index, qi)?;

                #[cfg(feature = "debugging")]
                state.output_svg(debug::svg::SvgTriangulationStyle::highlight_nexus(ni), debug::svg::SvgOutputLevel::AllSteps);
                
                *qon = QueryNodeOrNexus::Nexus(ni);
                Ok(ni)
            }

            let qoni0 = si;
            let qoni1 = (si + 1) % qons.len();
            let qon0 = qons[qoni0].clone();
            let qon1 = qons[qoni1].clone();

            let (ni0, ni1) = match (qon0, qon1) {
                (QueryNodeOrNexus::QueryNode(index0, qi0), QueryNodeOrNexus::QueryNode(index1, qi1)) => {
                    if state.ps[index0.clone()] < state.ps[index1.clone()] {
                        let ni0 = add_vertex(state, &mut qons[qoni0], index0, qi0)?;
                        let ni1 = add_vertex(state, &mut qons[qoni1], index1, qi1)?;
                        (ni0, ni1)
                    } else {
                        let ni1 = add_vertex(state, &mut qons[qoni1], index1, qi1)?;
                        let ni0 = add_vertex(state, &mut qons[qoni0], index0, qi0)?;
                        (ni0, ni1)
                    }
                }
                (QueryNodeOrNexus::QueryNode(index0, qi0), QueryNodeOrNexus::Nexus(ni1)) => {
                    let ni0 = add_vertex(state, &mut qons[qoni0], index0, qi0)?;
                    (ni0, ni1)
                }
                (QueryNodeOrNexus::Nexus(ni0), QueryNodeOrNexus::QueryNode(index1, qi1)) => {
                    let ni1 = add_vertex(state, &mut qons[qoni1], index1, qi1)?;
                    (ni0, ni1)
                }
                (QueryNodeOrNexus::Nexus(ni0), QueryNodeOrNexus::Nexus(ni1)) => (ni0, ni1),
            };
            
            let (ni_min, ni_max) = if state.ps[state.ns[ni0].vertex()] < state.ps[state.ns[ni1].vertex()] {
                (ni0, ni1)
            } else {
                (ni1, ni0)
            };

            state.add_segment(ni_min, ni_max)?;

            #[cfg(feature = "debugging")]
            state.advance_step();

            Ok(())
        }

        // Random insertion order of the segments avoids constant worst-case scenarios
        let mut segment_order: Vec<_> = (0..q_lookup.len()).collect();
        segment_order[..].shuffle(&mut rand::thread_rng());

        // Periodically, at a decreasing rate, find the trapezoid each uninserted vertex is contained within, based on the current query structure
        // The next search can begin from that query node
        let mut update_count = 1;
        let mut next_update = math_n(q_lookup.len(), update_count);

        for (i, vi0) in segment_order.into_iter().enumerate() {
            add_nth_segment(self, &mut q_lookup[..], vi0)?;
            if i == next_update {
                for qnon in q_lookup.iter_mut() {
                    if let QueryNodeOrNexus::QueryNode(index, qi) = qnon {
                        let (qi_new, _) = self.find_trapezoid_from_root(index.clone(), *qi);
                        *qnon = QueryNodeOrNexus::QueryNode(index.clone(), qi_new);
                    }
                }

                update_count += 1;
                next_update = math_n(q_lookup.len(), update_count);
            }
        }

        Ok(())
    }

    fn add_vertex(&mut self, vi: P::Index, qi_root: Idx<QueryNode<P::Vertex, P::Index>>) -> Result<Idx<Nexus<P::Vertex, P::Index>>, TriangulationInternalError> {
        let (qi_parent, ti) = self.find_trapezoid_from_root(vi.clone(), qi_root);
        let ti_new = self.ts.next_index();

        let qi_down = self.qs.next_index();
        let qi_up = qi_down + 1;

        let ni = self.ns.push_get_index(Nexus::new(vi, ti_new, ti));

        let (q_left, q_right) = self.qs[qi_parent].into_y(qi_down, qi_up, ni, ti_new);
        self.ts[ti].set_sink(qi_down);
        self.qs.push(q_left);
        self.qs.push(q_right);

        let t_new = self.ts[ti].split_horizontal(qi_down, qi_up, ni);

        if let Some(ni_up) = t_new.up() {
            let n_up = &mut self.ns[ni_up];
            n_up.replace_trapezoid(ti, ti_new)?;
        }

        self.ts.push(t_new);

        self.check_consistency();

        Ok(ni)
    }

    pub fn add_segment(&mut self, ni_min: Idx<Nexus<P::Vertex, P::Index>>, ni_max: Idx<Nexus<P::Vertex, P::Index>>) -> Result<(), TriangulationInternalError> {
        let si = self.ss.push_get_index(Segment::new(ni_min, ni_max));

        let ti = self.ns[ni_max].get_down_trapezoid_in_direction(self.ps, &self.ns, &self.ss, &self.ss[si])?;

        // Each segment adds one additional trapezoid
        let qi = self.ts[ti].sink();
        let ti_new = self.ts.next_index();
        let qi_left = self.qs.next_index();
        let qi_right = qi_left + 1;
        let (q_left, q_right) = self.qs[qi].into_x(qi_left, qi_right, si, ti_new);
        self.ts[ti].set_sink(qi_left);
        self.qs.push(q_left);
        self.qs.push(q_right);

        let t_new = self.ts[ti].split_vertical(qi_left, qi_right, si);
        self.ts.push(t_new);

        Nexus::add_segment(&mut self.ns, self.ps, &self.ss, ni_max, si, ti_new)?;

        #[cfg(feature = "debugging")]
        self.output_svg(debug::svg::SvgTriangulationStyle::highlight_segment(si), debug::svg::SvgOutputLevel::AllSteps);

        let t= &self.ts[ti];
        let mut ni = t.down().ok_or_else(|| TriangulationInternalError::new(format!("Segment min nexus not found at {}", ti)))?;
        if ni != ni_min && !self.ss[si].is_on_left(self.ps, &self.ns, self.ns[ni].vertex()) {
            let n = &mut self.ns[ni];
            n.replace_trapezoid(ti, ti_new)?;
        }

        let mut ti_upleft = ti;
        let mut ti_upright = ti_new;

        while ni != ni_min {
            let ti = self.ns[ni].get_down_trapezoid_in_direction(self.ps, &self.ns, &self.ss, &self.ss[si])?;

            ni = self.ts[ti].down().ok_or_else(|| TriangulationInternalError::new(format!("Segment min nexus not found at {}", ti)))?;
            
            let t = &self.ts[ti];
            let t_upleft = &self.ts[ti_upleft];
            let t_upright = &self.ts[ti_upright];
            let qi_sink = t.sink();

            if t.right() == t_upright.right() {
                let qi_left = self.qs.next_index();
                let q_left = self.qs[qi_sink].into_x_merge(qi_left, t_upright.sink(), si);
                self.ts[ti].set_sink(qi_left);
                self.qs.push(q_left);
                self.ts[ti].set_right(si);
                self.ts[ti_upright].set_down(ni);
                
                if ni != ni_min && !self.ss[si].is_on_left(self.ps, &self.ns, self.ns[ni].vertex()) {
                    self.ns[ni].replace_trapezoid(ti, ti_upright)?;
                }
                ti_upleft = ti;
                // ti_upright remains the same
            } else if t.left() == t_upleft.left() {
                let qi_right = self.qs.next_index();
                let q_right = self.qs[qi_sink].into_x_merge(t_upleft.sink(), qi_right, si);
                self.ts[ti].set_sink(qi_right);
                self.qs.push(q_right);
                self.ts[ti].set_left(si);
                self.ts[ti_upleft].set_down(ni);
                
                if ni == ni_min || self.ss[si].is_on_left(self.ps, &self.ns, self.ns[ni].vertex()) {
                    self.ns[ni].replace_trapezoid(ti, ti_upleft)?;
                }
                ti_upright = ti;
                // ti_upleft remains the same
            } else {
                return Err(TriangulationInternalError::new(format!("No matching side segment during split - ti: {}, ti_upleft: {}, ti_upright: {}, t.left: {:?}, t.right: {:?}, t_upleft.left: {:?}, t_upright.right: {:?}", ti, ti_upleft, ti_upright, t.left(), t.right(), t_upleft.left(), t_upright.right())));
            }

            #[cfg(feature = "debugging")]
            self.output_svg(debug::svg::SvgTriangulationStyle::highlight_nexus(ni), debug::svg::SvgOutputLevel::AllSteps);
        }

        Nexus::add_segment(&mut self.ns, self.ps, &self.ss, ni_min, si, ti_upright)?;

        #[cfg(feature = "debugging")]
        self.output_svg(debug::svg::SvgTriangulationStyle::highlight_segment(si), debug::svg::SvgOutputLevel::MajorSteps);

        self.check_consistency();

        Ok(())
    }

    #[cfg(debug_assertions)]
    fn check_consistency(&self) {
        // Trapezoid adjacency
        for ni in self.ns.iter_index() {
            self.check_consistency_nexus(ni);
        }

        for ti in self.ts.iter_index() {
            self.check_consistency_trapezoid(ti);
        }

        self.check_consistency_query_node(self.query_node_root());
    }

    #[cfg(not(debug_assertions))]
    fn check_consistency(&self) { }

    fn check_consistency_query_node(&self, qi: Idx<QueryNode<P::Vertex, P::Index>>) -> usize {
        let q = &self.qs[qi];
        (match q {
            QueryNode::Branch(qi_left, qi_right, _) => {
                self.check_consistency_query_node(*qi_left) + self.check_consistency_query_node(*qi_right)
            },
            QueryNode::Sink(ti) => {
                let _t= &self.ts[*ti];
                // A trapezoid will have multiple sinks after merging, so this needs a more robust check
                // if t.sink() != qi {
                //     panic!("Inconsistent sink trapezoid: {} {}({})", qi, ti, t.sink());
                // }
                0
            },
        }) + 1
    }

    fn check_consistency_nexus(&self, ni: Idx<Nexus<P::Vertex, P::Index>>) {
        let n = &self.ns[ni];

        for ti_up in n.up_trapezoids().iter() {
            let t_up = &self.ts[*ti_up];
            if t_up.down() != Some(ni) {
                panic!("Inconsistent nexus-trapezoid connection: {}->{} (down: {})", ni, ti_up, t_up.down().map_or("None".to_string(), |ti| format!("{}", ti)));
            }
        }

        for ti_down in n.down_trapezoids().iter() {
            let t_down = &self.ts[*ti_down];
            if t_down.up() != Some(ni) {
                panic!("Inconsistent nexus-trapezoid connection: {}->{} (up: {})", ni, ti_down, t_down.up().map_or("None".to_string(), |ti| format!("{}", ti)));
            }
        }
    }

    fn check_consistency_trapezoid(&self, ti: Idx<Trapezoid<P::Vertex, P::Index>>) {
        let t = &self.ts[ti];

        if let Some(ni_down) = t.down() {
            let n = &self.ns[ni_down];
            if !n.iter_up_trapezoids().any(|ti_up| ti == ti_up) {
                panic!("Inconsistent trapezoid-nexus connection: {}->{}", ti, ni_down);
            }
        }
        if let Some(ni_up) = t.up() {
            let n = &self.ns[ni_up];
            if !n.iter_down_trapezoids().any(|ti_down| ti == ti_down) {
                panic!("Inconsistent trapezoid-nexus connection: {}->{}", ti, ni_up);
            }
        }
        let qi = self.ts[ti].sink();
        if let QueryNode::Sink(ti_other) = self.qs[qi] {
            if ti != ti_other {
                panic!("Inconsistent trapezoid-query node connection: {}->{}({})", ti, qi, ti_other);
            }
        } else {
            panic!("Trapezoid points to a non-sink query node: {}->{}", ti, qi);
        }
    }
}

impl<'a, P: PolygonList<'a>> TrapezoidationStructure<'a, P> for TrapezoidationState<'a, P> {
    fn ps(&self) -> PolygonListExt<'a, P> { self.ps }

    fn ns(&self) -> &[Nexus<P::Vertex, P::Index>] { &self.ns }

    fn ss(&self) -> &[Segment<P::Vertex, P::Index>] { &self.ss }

    fn ts(&self) -> &[Trapezoid<P::Vertex, P::Index>] { &self.ts }

    fn qs(&self) -> &[QueryNode<P::Vertex, P::Index>] { &self.qs }
}

pub(crate) struct Trapezoidation<'a, P: PolygonList<'a>> {
    ps: PolygonListExt<'a, P>,
    ns: Vec<Nexus<P::Vertex, P::Index>>,
    ss: Vec<Segment<P::Vertex, P::Index>>,
    ts: Vec<Trapezoid<P::Vertex, P::Index>>,
    qs: Vec<QueryNode<P::Vertex, P::Index>>,
}

impl<'a, P: PolygonList<'a>> Trapezoidation<'a, P> {
    fn new(state: TrapezoidationState<'a, P>) -> Self {
        Self {
            ps: state.ps,
            ns: state.ns,
            ss: state.ss,
            ts: state.ts,
            qs: state.qs,
        }
    }

    pub fn top_trapezoid(&self) -> Result<Idx<Trapezoid<P::Vertex, P::Index>>, TriangulationInternalError> {
        let mut qi = Idx::<QueryNode<P::Vertex, P::Index>>::new(0);
        loop {
            match &self.qs[qi] {
                QueryNode::Branch(_, right, kind) => match kind {
                    QueryNodeBranch::X(_) => return Err(TriangulationInternalError::new("Finding the top trapezoid should not reach an X node")),
                    QueryNodeBranch::Y(_) => qi = *right, // Always take the 'above' branch
                },
                QueryNode::Sink(ti) => return Ok(*ti),
            }
        }
    }

    fn triangulate_inner<FB: FanBuilder<'a, P>>(&self, fbs: &mut FanBuilderState<'a, P, FB>) -> Result<(), TriangulationError<FB::Error>> {
        struct State<V: Vertex, Index: VertexIndex> {
            ti: Idx<Trapezoid<V, Index>>,
            monotones: Option<Ot<MonotoneBuilder<Index>>>,
        }
        impl<V: Vertex, Index: VertexIndex> State<V, Index> {
            pub fn new(ti: Idx<Trapezoid<V, Index>>, monotones: Option<Ot<MonotoneBuilder<Index>>>) -> Self {
                Self { ti, monotones }
            }
        }

        const INNER_POLYGON_ERROR: &str = "A trapezoid inside the polygon must be enclosed";

        let mut ti = self.top_trapezoid().map_err(|e| TriangulationError::InternalError(e))?;
        // If the current trapezoid is inside the polygon, fans is Some, outside it is None
        let mut monotones = Option::<Ot<MonotoneBuilder<P::Index>>>::None;

        // We will treat the graph of trapezoids as a tree and perform a depth-first traversal.
        // Whenever we reach an 'A' nexus, continue traversing the leftmost branch, but store the center
        // and rightmost branches here. Once the left branch hits a dead-end (i.e. 'V' nexus), it will
        // pick up from the next branch in this queue
        let mut branch_stack = Vec::<State<P::Vertex, P::Index>>::new();
        // At 'V' nexuses where the left and right trapezoids are inside the polygon, 
        // the left trapezoid should push its monotone to this stack and yield.
        // Once the right trapezoid reaches this point, it will pop from this stack and combine with its current monotone
        // to have a Ot::Two monotone going down
        let mut monotone_stack = Vec::<MonotoneBuilder<P::Index>>::new();
        
        while let Some(ni_down) = self.ts[ti].down() {
            let t = &self.ts[ti];
            let n_down = &self.ns[ni_down];

            if let Some(mut monotones_some) = monotones.take() {
                // Add this nexus to all monotone chains
                for monotone in monotones_some.iter_mut() {
                    monotone.add_vertex(self.ps, n_down.vertex());
                }

                let s_left = match t.left() {
                    Some(si_left) => &self.ss[si_left],
                    None => return Err(TriangulationError::internal(INNER_POLYGON_ERROR)),
                };
                let s_right = match t.right() {
                    Some(si_right) => &self.ss[si_right],
                    None => return Err(TriangulationError::internal(INNER_POLYGON_ERROR)),
                };

                let ni_up = match t.up() {
                    Some(ni_up) => ni_up,
                    None => return Err(TriangulationError::internal(INNER_POLYGON_ERROR)),
                };

                // If the nexus is part of the left or right segment of the trapezoid,
                // and the previous (upper) nexus is not on the same segment,
                // draw a diagonal, ending one of the monotone chains
                if let Some(monotone_complete) = {
                    if ni_down == s_left.ni_min() && ni_down != s_right.ni_min() && ni_up != s_left.ni_max() {
                        Some(match monotones_some {
                            Ot::One(monotone0) => monotone0,
                            Ot::Two(monotone0, monotone1) => {
                                monotones = Some(monotone1.into());
                                monotone0
                            }
                        })
                    } else if ni_down == s_right.ni_min() && ni_down != s_left.ni_min() && ni_up != s_right.ni_max() {
                        Some(match monotones_some {
                            Ot::One(monotone0) => monotone0,
                            Ot::Two(monotone0, monotone1) => {
                                monotones = Some(monotone0.into());
                                monotone1
                            }
                        })
                    } else {
                        // If the nexuses are on the same segment, there is no more work to be done; restore the monotones as-is
                        monotones = Some(monotones_some);
                        None
                    } 
                } {
                    // Triangulate the completed monotone
                    match monotone_complete.build(self.ps) {
                        Ok(monotone_complete) => {
                            if let Some(monotone_complete) = monotone_complete {
                                monotone_complete.build_fans::<P, FB>(self.ps, fbs)?;
                            }
                        },
                        Err(e) => return Err(TriangulationError::InternalError(e)),
                    }

                    // If that was the only monotone, we need to start a new one
                    if monotones.is_none() {
                        // Begin with the upper and lower nexuses' vertices
                        let mut fan_new = MonotoneBuilder::new(self.ns[ni_up].vertex().clone());
                        fan_new.add_vertex(self.ps, n_down.vertex());
                        monotones = Some(fan_new.into());
                    }
                }
            }

            ti = match n_down.final_type().map_err(|e| TriangulationError::InternalError(e))? {
                FinalNexusType::V { ti_upleft, ti_upcenter, ti_upright, ti_down } => {
                    if let Some(monotones_some) = monotones.take() {
                        if ti == ti_upleft {
                            // Stash the monotone here
                            monotone_stack.push(monotones_some.ok_one_or_else(|_, _| TriangulationError::internal("Expected a single monotone"))?);
                        } else if ti == ti_upcenter {
                            // Finish the monotone(s)
                            for monotone in monotones_some.into_iter() {
                                match monotone.build(self.ps) {
                                    Ok(monotone) => {
                                        if let Some(monotone) = monotone {
                                            monotone.build_fans::<P, FB>(self.ps, fbs)?;
                                        }
                                    }
                                    Err(e) => return Err(TriangulationError::InternalError(e)),
                                }
                            }
                        } else if ti == ti_upright {
                            // Pop the stashed monotone, make it the left monotone
                            let left_monotone = monotone_stack.pop().ok_or_else(|| TriangulationError::internal("Unexpected empty branch stack"))?;
                            let right_monotone = monotones_some.ok_one_or_else(|_, _| TriangulationError::internal("Expected a single monotone"))?;
                            monotones = Some((left_monotone, right_monotone).into());
                        } else {
                            return Err(TriangulationError::internal("Invalid trapezoidation"));
                        }
                    }
                    
                    if ti == ti_upright {
                        // Continue down with zero or two monotones
                        ti_down
                    } else {
                        // Resume from another pending branch
                        let state = branch_stack.pop().ok_or_else(|| TriangulationError::internal("Unexpected empty branch stack"))?;
                        monotones = state.monotones;
                        state.ti
                    }
                }
                FinalNexusType::I { ti_upleft, ti_upright, ti_downleft, ti_downright } => {
                    // Simply continue to the next trapezoid
                    if ti == ti_upleft {
                        ti_downleft
                    } else if ti == ti_upright {
                        ti_downright
                    } else {
                        return Err(TriangulationError::internal("Invalid 'I' nexus type"));
                    }
                }
                FinalNexusType::A { ti_downleft, ti_downcenter, ti_downright, .. } => {
                    if let Some(monotones_some) = monotones.take() {
                        let (monotone_left, monotone_right) = match monotones_some {
                            Ot::One(monotone) => {
                                let ni_up = t.up().ok_or_else(|| TriangulationError::internal(INNER_POLYGON_ERROR))?;

                                // Start a second monotone with the current and previous nexuses' vertices
                                let mut monotone_new = MonotoneBuilder::new(self.ns[ni_up].vertex().clone());
                                monotone_new.add_vertex(self.ps, n_down.vertex());

                                // Put the new monotone on the correct side
                                if ni_up == self.ss[t.left().ok_or_else(|| TriangulationError::internal(INNER_POLYGON_ERROR))?].ni_max() {
                                    (monotone_new, monotone)
                                } else if ni_up == self.ss[t.right().ok_or_else(|| TriangulationError::internal(INNER_POLYGON_ERROR))?].ni_max() {
                                    (monotone, monotone_new)
                                } else {
                                    return Err(TriangulationError::internal("Expected nexus on top of left or right segment"));
                                }
                            }
                            // Simply give one monotone to each side
                            Ot::Two(monotone0, monotone1) => (monotone0, monotone1),
                        };
                        monotones = Some(monotone_left.into());
                        
                        branch_stack.push(State::new(ti_downright, Some(monotone_right.into())));
                        branch_stack.push(State::new(ti_downcenter, None));
                    } else {
                        // The left and right trapezoids are still outside the polygon
                        branch_stack.push(State::new(ti_downright, None));
                        // Start a new monotone from the center trapezoid
                        let monotone_new = MonotoneBuilder::new(n_down.vertex().clone());
                        branch_stack.push(State::new(ti_downcenter, Some(monotone_new.into())));
                    }
                    ti_downleft
                }
            }
        }

        if monotone_stack.len() > 0 {
            Err(TriangulationError::internal("Mismatched monotone stack"))
        } else if branch_stack.len() > 0 {
            Err(TriangulationError::internal("Mismatched branch stack"))
        } else if monotones.is_some() {
            Err(TriangulationError::internal("Unexpected partial monotones"))
        } else {
            Ok(())
        }
    }

    pub fn triangulate<FB: FanBuilder<'a, P>>(&self, initializer: FB::Initializer) -> Result<FB::Output, TriangulationError<FB::Error>> {
        let mut fbs = FanBuilderState::<'a, P, FB>::Uninitialized(initializer);
        // Separate out the actual triangulation logic, so FanBuilder error handling can be consolidated to one location
        let result = self.triangulate_inner(&mut fbs);
        fbs.complete(result)
    }
}

#[cfg(feature = "debugging")]
impl<'a, P: PolygonList<'a>> debug::svg::SvgElement<debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>, ()> for TrapezoidationState<'a, P> {
    fn write_svg<'b>(&self, svg_output: &mut debug::svg::SvgOutput<'b, debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>>, _state: &()) -> fmt::Result {
        use svg_fmt::*;
        use fmt::Write;

        for pv in self.ps.iter_polygon_vertices().map(Into::into).chain(iter::once(PolygonVertex::NewPolygon)) {
            let mut vs = Vec::new();
            match pv {
                PolygonVertex::ContinuePolygon(index) => {
                    let v = &self.ps[index];
                    vs.push([v.x().to_f32().unwrap(), v.y().to_f32().unwrap()]);
                }
                PolygonVertex::NewPolygon => {
                    if vs.len() > 2 {
                        writeln!(svg_output, "{}", 
                            polygon(&vs)
                                .stroke(Stroke::Color(rgb(255, 0, 255), svg_output.context.percent(0.3)))
                        )?;
                    }
                    vs.clear();
                }
            }
        }
        for pv in self.ps.iter_polygon_vertices().map(Into::<PolygonVertex<P::Index>>::into) {
            if let PolygonVertex::ContinuePolygon(index) = pv {
                svg_output.append_element(&IndexWrap(index), self)?;
            }
        }
        for ti in self.ts.iter_index() {
            svg_output.append_element(&ti, self)?;
        }
        for si in self.ss.iter_index() {
            svg_output.append_element(&si, self)?;
        }
        for ni in self.ns.iter_index() {
            svg_output.append_element(&ni, self)?;
        }
        Ok(())
    }
}

#[cfg(feature = "debugging")]
struct IndexWrap<Index>(Index);

#[cfg(feature = "debugging")]
impl<'a, P: PolygonList<'a>> debug::svg::SvgElement<debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>, TrapezoidationState<'a, P>> for IndexWrap<P::Index> {
    fn write_svg<'b>(&self, svg_output: &mut debug::svg::SvgOutput<'b, debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>>, state: &TrapezoidationState<'a, P>) -> std::fmt::Result {
        use svg_fmt::*;
        use fmt::Write;

        let style = svg_output.style.get_v_style(self.0.clone(), &state.ps[self.0.clone()]);

        let v = &state.ps[self.0.clone()];
        
        let r = svg_output.context.percent(0.2);
        let color = match style {
            debug::svg::SvgElementStyle::Hide => rgb(0, 0, 0),
            debug::svg::SvgElementStyle::Standard => green(),
            debug::svg::SvgElementStyle::Highlight => rgb(255, 126, 0),
        };
        writeln!(svg_output, "{}",
            debug::svg::circle(v.x().to_f32().unwrap(), v.y().to_f32().unwrap(), r)
                .fill(Fill::Color(color))
        )?;
        Ok(())
    }
}

#[cfg(feature = "debugging")]
fn get_x_intercept<V: Vertex>(v_min: &VertexExt<V>, v_max: &VertexExt<V>, y: f32) -> f32 {
    let y_diff = (v_max.y() - v_min.y()).to_f32().unwrap();
    if y_diff != 0.0 {
        let slope = (v_max.x() - v_min.x()).to_f32().unwrap() / y_diff;
        v_min.x().to_f32().unwrap() + slope * (y - v_min.y().to_f32().unwrap())
    } else {
        v_min.x().to_f32().unwrap()
    }
}

#[cfg(feature = "debugging")]
impl<'a, P: PolygonList<'a>> debug::svg::SvgElement<debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>, TrapezoidationState<'a, P>> for Idx<Nexus<P::Vertex, P::Index>> {
    fn write_svg<'b>(&self, svg_output: &mut debug::svg::SvgOutput<'b, debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>>, state: &TrapezoidationState<'a, P>) -> fmt::Result {
        use svg_fmt::*;
        use fmt::Write;

        let style = svg_output.style.get_n_style(*self, &state.ns[*self]);

        let n = &state.ns[*self];

        let v = &state.ps[n.vertex()];
        let t_left = &state.ts[*n.up_trapezoids().first()];
        let x_left = if let Some(si_left) = t_left.left() {
            let s_left = &state.ss[si_left];
            get_x_intercept(&state.ps[state.ns[s_left.ni_min()].vertex()], &state.ps[state.ns[s_left.ni_max()].vertex()], v.y().to_f32().unwrap())
        } else {
            v.x().to_f32().unwrap()
        };
        let t_right = &state.ts[*n.up_trapezoids().last()];
        let x_right = if let Some(si_right) = t_right.right() {
            let s_right = &state.ss[si_right];
            get_x_intercept(&state.ps[state.ns[s_right.ni_min()].vertex()], &state.ps[state.ns[s_right.ni_max()].vertex()], v.y().to_f32().unwrap())
        } else {
            v.x().to_f32().unwrap()
        };

        if x_left != x_right {
            let width = svg_output.context.percent(0.25);
            writeln!(svg_output, "{}",
                line_segment(x_left, v.y().to_f32().unwrap(), x_right, v.y().to_f32().unwrap())
                    .color(green())
                    .width(width)
            )?;
        }

        let r = svg_output.context.percent(0.5);
        let color = match style {
            crate::debug::svg::SvgElementStyle::Hide => return Ok(()),
            crate::debug::svg::SvgElementStyle::Standard => blue(),
            crate::debug::svg::SvgElementStyle::Highlight => rgb(255, 126, 0),
        };
        let fill = Fill::Color(color);
        writeln!(svg_output, "{}",
            debug::svg::circle(v.x().to_f32().unwrap(), v.y().to_f32().unwrap(), r)
                .fill(fill)
        )?;

        if debug::env::svg::show_labels() {
            let gap = svg_output.context.percent(1.0);
            writeln!(svg_output, "{}",
                text(v.x().to_f32().unwrap() - gap, v.y().to_f32().unwrap(), self.to_string())
                    .color(black())
                    .align(Align::Right)
                    .size(svg_output.context.percent(1.0))
            )?;
        }
        Ok(())
    }
}

#[cfg(feature = "debugging")]
impl<'a, P: PolygonList<'a>> debug::svg::SvgElement<debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>, TrapezoidationState<'a, P>> for Idx<Segment<P::Vertex, P::Index>> {
    fn write_svg<'b>(&self, svg_output: &mut debug::svg::SvgOutput<'b, debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>>, state: &TrapezoidationState<'a, P>) -> fmt::Result {
        use svg_fmt::*;
        use fmt::Write;

        let style = svg_output.style.get_s_style(*self, &state.ss[*self]);

        let s = &state.ss[*self];
        let n_min = &state.ns[s.ni_min()];
        let n_max = &state.ns[s.ni_max()];
        let v_min = &state.ps[n_min.vertex()];
        let v_max = &state.ps[n_max.vertex()];
        let width = svg_output.context.percent(0.5);

        let color = match style {
            debug::svg::SvgElementStyle::Hide => return Ok(()),
            debug::svg::SvgElementStyle::Standard => red(),
            debug::svg::SvgElementStyle::Highlight => rgb(255, 126, 0),
        };
        writeln!(svg_output, "{}",
            line_segment(v_min.x().to_f32().unwrap(), v_min.y().to_f32().unwrap(), v_max.x().to_f32().unwrap(), v_max.y().to_f32().unwrap())
                .color(color)
                .width(width)
        )?;
        Ok(())
    }
}

#[cfg(feature = "debugging")]
impl<'a, P: PolygonList<'a>> debug::svg::SvgElement<debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>, TrapezoidationState<'a, P>> for Idx<Trapezoid<P::Vertex, P::Index>> {
    fn write_svg<'b>(&self, svg_output: &mut debug::svg::SvgOutput<'b, debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>>, state: &TrapezoidationState<'a, P>) -> fmt::Result {
        use svg_fmt::*;
        use fmt::Write;

        let style = svg_output.style.get_t_style(*self, &state.ts[*self]);
        let width = svg_output.context.percent(0.1);
        let color = match style {
            debug::svg::SvgElementStyle::Hide => return Ok(()),
            debug::svg::SvgElementStyle::Standard => rgb(255, 255, 0),
            debug::svg::SvgElementStyle::Highlight => rgb(255, 126, 0),
        };

        let t = &state.ts[*self];
        let y_min = if let Some(ni_down) = t.down() {
            state.ps[state.ns[ni_down].vertex()].y().to_f32().unwrap()
        } else {
            svg_output.context.view_y_min
        };
        let y_max = if let Some(ni_up) = t.up() {
            state.ps[state.ns[ni_up].vertex()].y().to_f32().unwrap()
        } else {
            svg_output.context.view_y_max
        };
        let y = (y_max + y_min) / 2.0;

        let (x_min, x_topleft, x_bottomleft) = if let Some(si_left) = t.left() {
            let s_left = &state.ss[si_left];

            let v_min = &state.ps[state.ns[s_left.ni_min()].vertex()];
            let v_max = &state.ps[state.ns[s_left.ni_max()].vertex()];
            (get_x_intercept(v_min, v_max, y).to_f32().unwrap(), get_x_intercept(v_min, v_max, y_max).to_f32().unwrap(), get_x_intercept(v_min, v_max, y_min).to_f32().unwrap())
        } else {
            (svg_output.context.view_x_min, svg_output.context.view_x_min, svg_output.context.view_x_min)
        };
        let (x_max, x_topright, x_bottomright) = if let Some(si_right) = t.right() {
            let s_right = &state.ss[si_right];

            let v_min = &state.ps[state.ns[s_right.ni_min()].vertex()];
            let v_max = &state.ps[state.ns[s_right.ni_max()].vertex()];
            (get_x_intercept(v_min, v_max, y).to_f32().unwrap(), get_x_intercept(v_min, v_max, y_max).to_f32().unwrap(), get_x_intercept(v_min, v_max, y_min).to_f32().unwrap())
        } else {
            (svg_output.context.view_x_max, svg_output.context.view_x_max, svg_output.context.view_x_max)
        };
        let x = (x_max + x_min) / 2.0;
        
        writeln!(svg_output, "{}", 
            line_segment(x_topleft, y_max, x_bottomright, y_min)
                .color(color)
                .width(width)
        )?;
        writeln!(svg_output, "{}", 
            line_segment(x_bottomleft, y_min, x_topright, y_max)
                .color(color)
                .width(width)
        )?;
        if debug::env::svg::show_labels() {
            writeln!(svg_output, "{}",
                text(x, y, self.to_string())
                    .size(svg_output.context.percent(0.15))
                    .align(Align::Center)
            )?;
        }

        Ok(())
    }
}

#[cfg(feature = "debugging")]
impl<'a, P: PolygonList<'a>> debug::svg::SvgElement<debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>, TrapezoidationState<'a, P>> for Monotone<P::Index> {
    fn write_svg<'b>(&self, svg_output: &mut debug::svg::SvgOutput<'b, debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>>, state: &TrapezoidationState<'a, P>) -> fmt::Result {
        use svg_fmt::*;
        use fmt::Write;

        let points: Vec<_> = self.skipped_and_pending.iter().map(|vi| &state.ps[vi.clone()]).map(|v| [v.x().to_f32().unwrap(), v.y().to_f32().unwrap()]).collect();
        writeln!(svg_output, "{}",
            polygon(&points)
                .open()
                .stroke(Stroke::Color(black(), svg_output.context.percent(0.1)))
                .stroke_opacity(0.5)
        )?;
        Ok(())
    }
}

#[cfg(feature = "debugging")]
struct PolygonSvgWrap<'a, P: PolygonList<'a>>(P, marker::PhantomData<&'a ()>);

#[cfg(feature = "debugging")]
impl<'a, P: PolygonList<'a>> debug::svg::SvgElement<debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>> for PolygonSvgWrap<'a, P> {
    fn write_svg<'b>(&self, svg_output: &mut debug::svg::SvgOutput<'b, debug::svg::SvgTriangulationStyle<'a, P::Vertex, P::Index>>, _state: &()) -> fmt::Result {
        use svg_fmt::*;
        use fmt::Write;

        let mut points = Vec::new();

        for vertex in self.0.iter_polygon_vertices() {
            let vertex: PolygonVertex<_> = vertex.into();
            match vertex {
                PolygonVertex::ContinuePolygon(vertex) => {
                    points.push([self.0.get_vertex(vertex.clone()).x().to_f32().unwrap(), self.0.get_vertex(vertex).y().to_f32().unwrap()]);
                }
                PolygonVertex::NewPolygon => {
                    writeln!(svg_output, "{}",
                        polygon(&points)
                            .stroke(Stroke::Color(black(), svg_output.context.percent(1.)))
                    )?;            
                }
            }
        }
        Ok(())
    }
}
