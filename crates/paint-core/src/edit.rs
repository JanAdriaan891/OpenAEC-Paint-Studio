use crate::{
    codec::image_operation,
    model::*,
    render::{pixmap_from_rgba, render_transparent, rgba},
};
use anyhow::{ensure, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Brush,
    Eraser,
    Fill,
    Picker,
    Line,
    Rectangle,
    Ellipse,
    Text,
    Select,
    Pixels,
    Crop,
}
impl Tool {
    pub fn kind(self) -> Option<Kind> {
        Some(match self {
            Self::Brush => Kind::Brush,
            Self::Eraser => Kind::Eraser,
            Self::Fill => Kind::Fill,
            Self::Line => Kind::Line,
            Self::Rectangle => Kind::Rectangle,
            Self::Ellipse => Kind::Ellipse,
            Self::Text => Kind::Text,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub struct Editor {
    pub document: Document,
    pub selection: Vec<usize>,
    past: Vec<Document>,
    future: Vec<Document>,
    pub dirty: bool,
}
impl Default for Editor {
    fn default() -> Self {
        Self::new(Document::default())
    }
}
impl Editor {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            selection: vec![],
            past: vec![],
            future: vec![],
            dirty: false,
        }
    }
    pub fn commit(&mut self, mut next: Document) -> Result<()> {
        next.validate()?;
        if !next.retain_vectors && next.operations.len() > 1 {
            let layer = render_transparent(&next)?;
            next.operations = vec![image_operation(&layer)?];
        }
        if next == self.document {
            return Ok(());
        }
        self.past.push(self.document.clone());
        if self.past.len() > 40 {
            self.past.remove(0);
        }
        self.future.clear();
        self.document = next;
        self.selection
            .retain(|i| *i < self.document.operations.len());
        self.dirty = true;
        Ok(())
    }
    pub fn add(&mut self, op: Operation) -> Result<()> {
        op.validate()?;
        let mut next = self.document.clone();
        next.operations.push(op);
        self.commit(next)?;
        self.selection = vec![self.document.operations.len() - 1];
        Ok(())
    }
    pub fn undo(&mut self) {
        if let Some(doc) = self.past.pop() {
            self.future.push(std::mem::replace(&mut self.document, doc));
            self.selection.clear();
            self.dirty = true;
        }
    }
    pub fn redo(&mut self) {
        if let Some(doc) = self.future.pop() {
            self.past.push(std::mem::replace(&mut self.document, doc));
            self.selection.clear();
            self.dirty = true;
        }
    }
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
    pub fn select_all(&mut self) {
        self.selection = (0..self.document.operations.len()).collect();
    }
    pub fn selected_bounds(&self) -> Option<Bounds> {
        if self.selection.is_empty() {
            return None;
        }
        Some(
            self.selection
                .iter()
                .filter_map(|i| {
                    self.document
                        .operations
                        .get(*i)?
                        .bounds(self.document.size())
                })
                .reduce(Bounds::union)
                .unwrap_or(Bounds {
                    x: 0.,
                    y: 0.,
                    width: self.document.width as f32,
                    height: self.document.height as f32,
                }),
        )
    }
    pub fn hit(&self, at: Point) -> Option<usize> {
        self.document
            .operations
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, op)| {
                let mut b = op.bounds(self.document.size())?;
                b.x -= 5.;
                b.y -= 5.;
                b.width += 10.;
                b.height += 10.;
                b.contains(at).then_some(i)
            })
    }
    pub fn transform_selection(&mut self, matrix: Matrix) -> Result<()> {
        let mut next = self.document.clone();
        for &i in &self.selection {
            if let Some(op) = next.operations.get_mut(i) {
                *op = op.transformed(matrix, self.document.size());
            }
        }
        self.commit(next)
    }
    pub fn delete_selection(&mut self) -> Result<()> {
        if self.selection.is_empty() {
            return Ok(());
        }
        let mut next = self.document.clone();
        next.operations = next
            .operations
            .into_iter()
            .enumerate()
            .filter_map(|(i, op)| (!self.selection.contains(&i)).then_some(op))
            .collect();
        self.commit(next)?;
        self.selection.clear();
        Ok(())
    }
    pub fn align(&mut self, alignment: Align) -> Result<()> {
        ensure!(
            self.document.retain_vectors,
            "Enable Keep vectors before aligning objects"
        );
        ensure!(
            self.selection.len() >= 2,
            "Select at least two text or shape objects"
        );
        let mut indices = self.selection.clone();
        indices.sort_unstable();
        indices.dedup();
        ensure!(indices.len() >= 2, "Select at least two different objects");
        let mut objects = Vec::new();
        for &i in &indices {
            let op = self
                .document
                .operations
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("Invalid selection"))?;
            ensure!(
                matches!(
                    op.kind,
                    Kind::Text | Kind::Line | Kind::Rectangle | Kind::Ellipse
                ),
                "Alignment supports text and shapes"
            );
            objects.push((
                i,
                op.bounds(self.document.size())
                    .ok_or_else(|| anyhow::anyhow!("Object has no bounds"))?,
            ));
        }
        let reference = objects
            .iter()
            .map(|(_, b)| *b)
            .reduce(Bounds::union)
            .unwrap();
        let mut next = self.document.clone();
        for (i, b) in objects {
            let (dx, dy) = match alignment {
                Align::Left => (reference.x - b.x, 0.),
                Align::Center => (reference.x + reference.width / 2. - b.x - b.width / 2., 0.),
                Align::Right => (reference.x + reference.width - b.x - b.width, 0.),
                Align::Top => (0., reference.y - b.y),
                Align::Middle => (
                    0.,
                    reference.y + reference.height / 2. - b.y - b.height / 2.,
                ),
                Align::Bottom => (0., reference.y + reference.height - b.y - b.height),
            };
            next.operations[i] =
                next.operations[i].transformed([1., 0., 0., 1., dx, dy], next.size());
        }
        self.commit(next)
    }
    pub fn replace_selected(&mut self, op: Operation) -> Result<()> {
        ensure!(self.selection.len() == 1, "Select one object");
        let mut next = self.document.clone();
        next.operations[self.selection[0]] = op;
        self.commit(next)
    }
}
#[derive(Debug, Clone, Copy)]
pub enum TransformAction {
    RotateRight,
    RotateLeft,
    FlipHorizontal,
    FlipVertical,
}
pub fn transform_document(doc: &Document, action: TransformAction) -> Result<Document> {
    let w = doc.width as f32;
    let h = doc.height as f32;
    let (matrix, width, height) = match action {
        TransformAction::RotateRight => ([0., 1., -1., 0., h, 0.], doc.height, doc.width),
        TransformAction::RotateLeft => ([0., -1., 1., 0., 0., w], doc.height, doc.width),
        TransformAction::FlipHorizontal => ([-1., 0., 0., 1., w, 0.], doc.width, doc.height),
        TransformAction::FlipVertical => ([1., 0., 0., -1., 0., h], doc.width, doc.height),
    };
    transformed(doc, matrix, width, height)
}
pub fn resize(doc: &Document, width: u32, height: u32) -> Result<Document> {
    validate_dimensions(width, height)?;
    transformed(
        doc,
        [
            width as f32 / doc.width as f32,
            0.,
            0.,
            height as f32 / doc.height as f32,
            0.,
            0.,
        ],
        width,
        height,
    )
}
pub fn crop(doc: &Document, bounds: Bounds) -> Result<Document> {
    let x = bounds
        .x
        .round()
        .max(0.)
        .min(doc.width.saturating_sub(1) as f32);
    let y = bounds
        .y
        .round()
        .max(0.)
        .min(doc.height.saturating_sub(1) as f32);
    let w = bounds.width.round().max(1.).min(doc.width as f32 - x) as u32;
    let h = bounds.height.round().max(1.).min(doc.height as f32 - y) as u32;
    transformed(doc, [1., 0., 0., 1., -x, -y], w, h)
}
fn transformed(doc: &Document, matrix: Matrix, width: u32, height: u32) -> Result<Document> {
    let mut next = doc.clone();
    // A fill's region depends on the entire old canvas, so resolve it before clipping.
    if next.operations.iter().any(|o| o.kind == Kind::Fill) {
        next.operations = vec![image_operation(&render_transparent(doc)?)?];
    }
    next.operations = next
        .operations
        .iter()
        .map(|op| op.transformed(matrix, doc.size()))
        .collect();
    next.width = width;
    next.height = height;
    next.validate()?;
    Ok(next)
}
#[derive(Debug, Clone, Copy)]
pub enum Filter {
    Grayscale,
    Invert,
    Sepia,
    Adjust {
        brightness: f32,
        contrast: f32,
        saturation: f32,
    },
}
pub fn filter(doc: &Document, kind: Filter) -> Result<Document> {
    let source = render_transparent(doc)?;
    let mut bytes = rgba(&source);
    for p in bytes.chunks_exact_mut(4) {
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;
        let values = match kind {
            Filter::Grayscale => {
                let v = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                [v, v, v]
            }
            Filter::Invert => [255. - r, 255. - g, 255. - b],
            Filter::Sepia => [
                0.393 * r + 0.769 * g + 0.189 * b,
                0.349 * r + 0.686 * g + 0.168 * b,
                0.272 * r + 0.534 * g + 0.131 * b,
            ],
            Filter::Adjust {
                brightness,
                contrast,
                saturation,
            } => {
                let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                let s = saturation.clamp(0., 200.) / 100.;
                let c = contrast.clamp(-100., 100.);
                let f = 259. * (c + 255.) / (255. * (259. - c));
                [r, g, b].map(|v| {
                    f * (gray + (v - gray) * s - 128.) + 128. + brightness.clamp(-100., 100.) * 2.55
                })
            }
        };
        for i in 0..3 {
            p[i] = values[i].round().clamp(0., 255.) as u8;
        }
    }
    let image = pixmap_from_rgba(doc.width, doc.height, &bytes)?;
    let mut next = doc.clone();
    next.operations = vec![image_operation(&image)?];
    Ok(next)
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
pub fn svg(doc: &Document) -> Result<String> {
    doc.validate()?;
    // Resolve raster-dependent erase/fill compositing; preserve vector objects otherwise.
    if doc
        .operations
        .iter()
        .any(|op| matches!(op.kind, Kind::Eraser | Kind::Fill))
    {
        use base64::Engine;
        let png = crate::png(doc)?;
        return Ok(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"><image width=\"100%\" height=\"100%\" href=\"data:image/png;base64,{}\"/></svg>",doc.width,doc.height,base64::engine::general_purpose::STANDARD.encode(png)));
    }
    let mut out=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",doc.width,doc.height,doc.width,doc.height);
    if doc.background != "transparent" {
        out += &format!(
            "<rect width=\"100%\" height=\"100%\" fill=\"{}\"/>",
            doc.background
        );
    }
    for op in &doc.operations {
        let m = op.transform.unwrap_or(IDENTITY);
        out += &format!(
            "<g transform=\"matrix({} {} {} {} {} {})\" opacity=\"{}\">",
            m[0], m[1], m[2], m[3], m[4], m[5], op.opacity
        );
        let style=format!("stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" fill=\"{}\"",op.color,op.size,if op.filled{op.color.as_str()}else{"none"});
        match op.kind {
            Kind::Image => {
                let s = op.source_size.unwrap_or(doc.size());
                out += &format!(
                    "<image width=\"{}\" height=\"{}\" href=\"{}\"/>",
                    s.width,
                    s.height,
                    op.data.as_deref().unwrap_or("")
                );
            }
            Kind::Brush => {
                if op.points.len() == 1 {
                    let p = op.points[0];
                    out += &format!(
                        "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>",
                        p.x,
                        p.y,
                        op.size / 2.,
                        op.color
                    );
                } else {
                    let mut path = format!("M{} {}", op.points[0].x, op.points[0].y);
                    for pair in op.points[1..].windows(2) {
                        path += &format!(
                            " Q{} {} {} {}",
                            pair[0].x,
                            pair[0].y,
                            (pair[0].x + pair[1].x) / 2.,
                            (pair[0].y + pair[1].y) / 2.
                        );
                    }
                    let end = op.points.last().unwrap();
                    path += &format!(" L{} {}", end.x, end.y);
                    out+=&format!("<path d=\"{path}\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" fill=\"none\"/>",op.color,op.size);
                }
            }
            Kind::Line | Kind::Rectangle | Kind::Ellipse => {
                let a = op.from.unwrap();
                let b = op.to.unwrap();
                match op.kind {
                    Kind::Line => {
                        out += &format!(
                            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" {style}/>",
                            a.x, a.y, b.x, b.y
                        )
                    }
                    Kind::Rectangle => {
                        out += &format!(
                            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" {style}/>",
                            a.x.min(b.x),
                            a.y.min(b.y),
                            (b.x - a.x).abs(),
                            (b.y - a.y).abs()
                        )
                    }
                    _ => {
                        out += &format!(
                            "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" {style}/>",
                            (a.x + b.x) / 2.,
                            (a.y + b.y) / 2.,
                            (b.x - a.x).abs() / 2.,
                            (b.y - a.y).abs() / 2.
                        )
                    }
                }
            }
            Kind::Text => {
                let a = op.at.unwrap();
                out+=&format!("<text x=\"{}\" y=\"{}\" font-size=\"{}\" font-family=\"{}\" font-weight=\"{}\" fill=\"{}\" dominant-baseline=\"hanging\">",a.x,a.y,op.size,escape(op.font_family.as_deref().unwrap_or("sans-serif")),if op.bold{"bold"}else{"normal"},op.color);
                for (i, line) in op.text.as_deref().unwrap_or("").split('\n').enumerate() {
                    out += &format!(
                        "<tspan x=\"{}\" dy=\"{}\">{}</tspan>",
                        a.x,
                        if i == 0 { 0. } else { op.size * 1.3 },
                        escape(line)
                    );
                }
                out += "</text>";
            }
            _ => {}
        }
        out += "</g>";
    }
    out += "</svg>";
    Ok(out)
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}
