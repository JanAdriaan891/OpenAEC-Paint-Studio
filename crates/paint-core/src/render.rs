use crate::model::*;
use anyhow::{anyhow, ensure, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::sync::OnceLock;
use tiny_skia::{BlendMode, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

fn transform(m: Matrix) -> Transform {
    Transform::from_row(m[0], m[1], m[2], m[3], m[4], m[5])
}
fn paint(op: &Operation) -> Result<Paint<'static>> {
    let c = parse_color(&op.color)?;
    let mut p = Paint::default();
    p.set_color_rgba8(c[0], c[1], c[2], (op.opacity * 255.).round() as u8);
    p.anti_alias = true;
    if op.kind == Kind::Eraser {
        p.blend_mode = BlendMode::DestinationOut;
    }
    Ok(p)
}
pub fn render_transparent(doc: &Document) -> Result<Pixmap> {
    doc.validate()?;
    let mut image =
        Pixmap::new(doc.width, doc.height).ok_or_else(|| anyhow!("Canvas allocation failed"))?;
    for op in &doc.operations {
        draw(&mut image, op, doc.size())?;
    }
    Ok(image)
}
pub fn render(doc: &Document) -> Result<Pixmap> {
    let layer = render_transparent(doc)?;
    with_background(layer, &doc.background)
}
pub fn with_background(layer: Pixmap, background: &str) -> Result<Pixmap> {
    if background == "transparent" {
        return Ok(layer);
    }
    let c = parse_color(background)?;
    let mut out = Pixmap::new(layer.width(), layer.height())
        .ok_or_else(|| anyhow!("Canvas allocation failed"))?;
    out.fill(tiny_skia::Color::from_rgba8(c[0], c[1], c[2], 255));
    out.draw_pixmap(
        0,
        0,
        layer.as_ref(),
        &Default::default(),
        Transform::identity(),
        None,
    );
    Ok(out)
}
pub fn png(doc: &Document) -> Result<Vec<u8>> {
    Ok(render(doc)?.encode_png()?)
}
pub fn rgba(pixmap: &Pixmap) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixmap.data().len());
    for pixel in pixmap.pixels() {
        let c = pixel.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    out
}
pub fn pixmap_from_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Pixmap> {
    ensure!(
        rgba.len() == width as usize * height as usize * 4,
        "Invalid image buffer"
    );
    let mut p = Pixmap::new(width, height).ok_or_else(|| anyhow!("Image allocation failed"))?;
    for (source, target) in rgba.chunks_exact(4).zip(p.pixels_mut()) {
        *target =
            tiny_skia::ColorU8::from_rgba(source[0], source[1], source[2], source[3]).premultiply();
    }
    Ok(p)
}
fn draw(image: &mut Pixmap, op: &Operation, size: Size) -> Result<()> {
    let matrix = op.transform.unwrap_or(IDENTITY);
    let t = transform(matrix);
    let p = paint(op)?;
    if op.kind == Kind::Fill {
        flood_fill(
            image,
            transform_point(op.at.unwrap(), matrix),
            parse_color(&op.color)?,
        );
        return Ok(());
    }
    if op.kind == Kind::Image {
        let data = STANDARD.decode(
            op.data
                .as_deref()
                .unwrap()
                .trim_start_matches("data:image/png;base64,"),
        )?;
        let source = crate::codec::decode_png(&data)?;
        let actual = Size {
            width: source.width(),
            height: source.height(),
        };
        let s = op.source_size.unwrap_or(size);
        let scale = Transform::from_scale(
            s.width as f32 / actual.width as f32,
            s.height as f32 / actual.height as f32,
        );
        image.draw_pixmap(
            0,
            0,
            source.as_ref(),
            &tiny_skia::PixmapPaint {
                opacity: op.opacity,
                quality: tiny_skia::FilterQuality::Bicubic,
                ..Default::default()
            },
            t.pre_concat(scale),
            None,
        );
        return Ok(());
    }
    if op.kind == Kind::Text {
        return text(image, op, t);
    }
    let mut path = PathBuilder::new();
    match op.kind {
        Kind::Brush | Kind::Eraser => {
            let a = op.points[0];
            if op.points.len() == 1 {
                path.push_circle(a.x, a.y, op.size / 2.);
                if let Some(path) = path.finish() {
                    image.fill_path(&path, &p, tiny_skia::FillRule::Winding, t, None);
                }
                return Ok(());
            }
            path.move_to(a.x, a.y);
            for pair in op.points[1..].windows(2) {
                path.quad_to(
                    pair[0].x,
                    pair[0].y,
                    (pair[0].x + pair[1].x) / 2.,
                    (pair[0].y + pair[1].y) / 2.,
                );
            }
            let b = op.points.last().unwrap();
            path.line_to(b.x, b.y);
        }
        Kind::Line => {
            let a = op.from.unwrap();
            let b = op.to.unwrap();
            path.move_to(a.x, a.y);
            path.line_to(b.x, b.y);
        }
        Kind::Rectangle | Kind::Ellipse => {
            let a = op.from.unwrap();
            let b = op.to.unwrap();
            if let Some(r) = Rect::from_xywh(
                a.x.min(b.x),
                a.y.min(b.y),
                (b.x - a.x).abs().max(0.01),
                (b.y - a.y).abs().max(0.01),
            ) {
                if op.kind == Kind::Ellipse {
                    path.push_oval(r);
                } else {
                    path.push_rect(r);
                }
            }
        }
        _ => {}
    }
    if let Some(path) = path.finish() {
        if op.filled && matches!(op.kind, Kind::Rectangle | Kind::Ellipse) {
            image.fill_path(&path, &p, tiny_skia::FillRule::Winding, t, None);
        } else {
            let stroke = Stroke {
                width: op.size,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..Default::default()
            };
            image.stroke_path(&path, &p, &stroke, t, None);
        }
    }
    Ok(())
}
fn text(image: &mut Pixmap, op: &Operation, t: Transform) -> Result<()> {
    static REGULAR: OnceLock<fontdue::Font> = OnceLock::new();
    static BOLD: OnceLock<fontdue::Font> = OnceLock::new();
    static MONO: OnceLock<fontdue::Font> = OnceLock::new();
    static SERIF: OnceLock<fontdue::Font> = OnceLock::new();
    static SERIF_BOLD: OnceLock<fontdue::Font> = OnceLock::new();
    let (cache, bytes): (&OnceLock<fontdue::Font>, &[u8]) =
        match (op.font_family.as_deref(), op.bold) {
            (Some("monospace"), _) => (
                &MONO,
                include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf"),
            ),
            (Some("serif"), false) => (
                &SERIF,
                include_bytes!("../../../assets/fonts/DejaVuSerif.ttf"),
            ),
            (Some("serif"), true) => (
                &SERIF_BOLD,
                include_bytes!("../../../assets/fonts/DejaVuSerif-Bold.ttf"),
            ),
            (_, true) => (
                &BOLD,
                include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf"),
            ),
            _ => (
                &REGULAR,
                include_bytes!("../../../assets/fonts/Inter-Regular.ttf"),
            ),
        };
    let font = cache.get_or_init(|| {
        fontdue::Font::from_bytes(bytes, Default::default()).expect("bundled font")
    });
    let at = op.at.unwrap();
    let color = parse_color(&op.color)?;
    let mut y = at.y;
    for line in op.text.as_deref().unwrap_or("").split('\n') {
        let mut x = at.x;
        for c in line.chars() {
            let (m, bitmap) = font.rasterize(c, op.size);
            if m.width > 0 && m.height > 0 {
                let mut glyph = Pixmap::new(m.width as u32, m.height as u32)
                    .ok_or_else(|| anyhow!("Glyph allocation failed"))?;
                for (pixel, alpha) in glyph.pixels_mut().iter_mut().zip(bitmap) {
                    *pixel = tiny_skia::ColorU8::from_rgba(
                        color[0],
                        color[1],
                        color[2],
                        (f32::from(alpha) * op.opacity).round() as u8,
                    )
                    .premultiply();
                }
                image.draw_pixmap(
                    0,
                    0,
                    glyph.as_ref(),
                    &Default::default(),
                    t.pre_translate(
                        x + m.xmin as f32,
                        y + op.size - m.height as f32 - m.ymin as f32,
                    ),
                    None,
                );
            }
            x += m.advance_width;
        }
        y += op.size * 1.3;
    }
    Ok(())
}
pub fn flood_fill(image: &mut Pixmap, at: Point, color: [u8; 4]) {
    let x = at.x.floor() as i32;
    let y = at.y.floor() as i32;
    let w = image.width() as i32;
    let h = image.height() as i32;
    if x < 0 || y < 0 || x >= w || y >= h {
        return;
    }
    let replacement =
        tiny_skia::ColorU8::from_rgba(color[0], color[1], color[2], color[3]).premultiply();
    let start = (y * w + x) as usize;
    let target = image.pixels()[start];
    if target == replacement {
        return;
    }
    let pixels = image.pixels_mut();
    let mut queue = vec![start];
    pixels[start] = replacement;
    let mut next = 0;
    while next < queue.len() {
        let p = queue[next];
        next += 1;
        let col = p % w as usize;
        for n in [
            if col > 0 { Some(p - 1) } else { None },
            if col + 1 < w as usize {
                Some(p + 1)
            } else {
                None
            },
            p.checked_sub(w as usize),
            if p + (w as usize) < pixels.len() {
                Some(p + w as usize)
            } else {
                None
            },
        ]
        .into_iter()
        .flatten()
        {
            if pixels[n] == target {
                pixels[n] = replacement;
                queue.push(n);
            }
        }
    }
}
