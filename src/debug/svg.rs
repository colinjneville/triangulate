use std::{fmt, io, path};

use crate::{Vertex, VertexExt, VertexIndex, idx::Idx, nexus::Nexus, segment::Segment, trapezoid::Trapezoid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SvgOutputLevel {
    None,
    ResultOnly,
    MajorSteps,
    AllSteps,
}

pub(crate) struct SvgOutput<'a, Style> {
    pub context: &'a SvgContext,
    pub style: Style,
    content: String,
}

impl<'a, Style> SvgOutput<'a, Style> {
    pub fn new(context: &'a SvgContext, style: Style) -> Self {
        Self {
            context,
            style,
            content: String::new(),
        }
    }

    pub fn append_element<State, E: SvgElement<Style, State>>(&mut self, element: &E, state: &State) -> fmt::Result {
        element.write_svg(self, state)
    }

    pub fn save<P: AsRef<path::Path>>(self, file_name: P) -> io::Result<()> {
        use std::io::Write;

        let path = self.context.output_path.join(file_name);
        let f = std::fs::File::create(path)?;
        let mut w = io::BufWriter::new(&f);
        
        writeln!(w, "<svg viewBox=\"{}, {}, {}, {}\" xmlns=\"http://www.w3.org/2000/svg\">", self.context.view_x_min, self.context.view_y_min, self.context.view_x_max - self.context.view_x_min, self.context.view_y_max - self.context.view_y_min)?;
        writeln!(w, "{}", self.content)?;
        writeln!(w, "</svg>")?;
        Ok(())
    }
}

impl<'a, Style> fmt::Write for SvgOutput<'a, Style> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.content.write_str(s)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SvgElementStyle {
    Hide,
    Standard,
    Highlight,
}
pub(crate) struct SvgTriangulationStyle<'a, V: 'a + Vertex, Index: 'a + VertexIndex> {
    v_style: Option<Box<dyn 'a + Fn(Index, &VertexExt<V>) -> SvgElementStyle>>,
    n_style: Option<Box<dyn 'a + Fn(Idx<Nexus<V, Index>>, &Nexus<V, Index>) -> SvgElementStyle>>,
    s_style: Option<Box<dyn 'a + Fn(Idx<Segment<V, Index>>, &Segment<V, Index>) -> SvgElementStyle>>,
    t_style: Option<Box<dyn 'a + Fn(Idx<Trapezoid<V, Index>>, &Trapezoid<V, Index>) -> SvgElementStyle>>,
    pub add_labels: bool,
}

impl<'a, V: 'a + Vertex, Index: 'a + VertexIndex> Default for SvgTriangulationStyle<'a, V, Index> {
    fn default() -> Self {
        Self {
            v_style: None,
            n_style: None,
            s_style: None,
            t_style: None,
            add_labels: true,
        }
    }
}

impl<'a, V: 'a + Vertex, Index: 'a + VertexIndex> SvgTriangulationStyle<'a, V, Index> {
    pub fn highlight_vertex(vi: Index) -> Self {
        Self {
            v_style: Some(Self::match_index(vi)),
            ..Self::default()
        }
    }

    pub fn highlight_nexus(ni: Idx<Nexus<V, Index>>) -> Self {
        Self {
            n_style: Some(Self::match_index(ni)),
            ..Self::default()
        }
    }

    pub fn highlight_segment(si: Idx<Segment<V, Index>>) -> Self {
        Self {
            s_style: Some(Self::match_index(si)),
            ..Self::default()
        }
    }

    pub fn highlight_trapezoid(ti: Idx<Trapezoid<V, Index>>) -> Self {
        Self {
            t_style: Some(Self::match_index(ti)),
            ..Self::default()
        }
    }

    fn evaluate_style<T, TI>(style: &Option<Box<dyn 'a + Fn(TI, T) -> SvgElementStyle>>, xi: TI, x: T) -> SvgElementStyle {
        if let Some(style) = style {
            style(xi, x)
        } else {
            SvgElementStyle::Standard
        }
    }

    pub fn get_v_style(&self, vi: Index, v: &VertexExt<V>) -> SvgElementStyle {
        Self::evaluate_style(&self.v_style, vi, v)
    }

    pub fn get_n_style(&self, ni: Idx<Nexus<V, Index>>, n: &Nexus<V, Index>) -> SvgElementStyle {
        Self::evaluate_style(&self.n_style, ni, n)
    }

    pub fn get_s_style(&self, si: Idx<Segment<V, Index>>, s: &Segment<V, Index>) -> SvgElementStyle {
        Self::evaluate_style(&self.s_style, si, s)
    }

    pub fn get_t_style(&self, ti: Idx<Trapezoid<V, Index>>, t: &Trapezoid<V, Index>) -> SvgElementStyle {
        Self::evaluate_style(&self.t_style, ti, t)
    }

    fn match_index<T, TI: 'a + PartialEq>(index: TI) -> Box<dyn 'a + Fn(TI, &T) -> SvgElementStyle> {
        Box::new(move |ti, _| if ti == index { 
            SvgElementStyle::Highlight
        } else {
            SvgElementStyle::Standard
        })
    }
}

pub(crate) struct SvgContext {
    pub output_path: path::PathBuf,
    pub output_level: SvgOutputLevel,
    pub view_x_min: f32,
    pub view_x_max: f32,
    pub view_y_min: f32,
    pub view_y_max: f32,
    pub show_labels: bool,
}

impl SvgContext {
    pub fn view_w(&self) -> f32 { self.view_x_max - self.view_x_min }
    pub fn view_h(&self) -> f32 { self.view_y_max - self.view_y_min }

    pub fn view_min_size(&self) -> f32 { self.view_w().min(self.view_h()) }
    pub fn view_max_size(&self) -> f32 { self.view_w().max(self.view_h()) }

    pub fn percent(&self, p: f32) -> f32 { self.view_min_size() * p / 100.0 }
}

pub(crate) trait SvgElement<Style, State=()> {
    fn write_svg<'a>(&self, svg_output: &mut SvgOutput<'a, Style>, state: &State) -> fmt::Result;
}

// svg_fmt is missing a function for Circle
pub(crate) fn circle(x: f32, y: f32, r: f32) -> svg_fmt::Circle {
    svg_fmt::Circle {
        x,
        y,
        radius: r,
        style: svg_fmt::Style::default(),
    }
}
