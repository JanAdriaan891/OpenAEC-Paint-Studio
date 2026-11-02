//! Rectangular pixel edits operate on the transparent composition, keeping the
//! document background separate. Object editing is a different selection mode.
use crate::{codec::image_operation, render_transparent, Bounds, Document, Point};
use anyhow::{ensure, Result};
use tiny_skia::{Pixmap, Transform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl PixelRect {
    pub fn between(doc: &Document, a: Point, b: Point) -> Option<Self> {
        let x = a.x.min(b.x).floor().clamp(0., doc.width as f32) as u32;
        let y = a.y.min(b.y).floor().clamp(0., doc.height as f32) as u32;
        let right = a.x.max(b.x).ceil().clamp(0., doc.width as f32) as u32;
        let bottom = a.y.max(b.y).ceil().clamp(0., doc.height as f32) as u32;
        (right > x && bottom > y).then_some(Self {
            x,
            y,
            width: right - x,
            height: bottom - y,
        })
    }
    pub fn bounds(self) -> Bounds {
        Bounds {
            x: self.x as f32,
            y: self.y as f32,
            width: self.width as f32,
            height: self.height as f32,
        }
    }
    fn validate(self, doc: &Document) -> Result<()> {
        ensure!(
            self.width > 0
                && self.height > 0
                && self
                    .x
                    .checked_add(self.width)
                    .is_some_and(|v| v <= doc.width)
                && self
                    .y
                    .checked_add(self.height)
                    .is_some_and(|v| v <= doc.height),
            "Invalid pixel selection"
        );
        Ok(())
    }
}
pub fn copy(doc: &Document, rect: PixelRect) -> Result<Pixmap> {
    rect.validate(doc)?;
    let source = render_transparent(doc)?;
    Ok(source
        .clone_rect(
            tiny_skia::IntRect::from_xywh(rect.x as i32, rect.y as i32, rect.width, rect.height)
                .unwrap(),
        )
        .ok_or_else(|| anyhow::anyhow!("Cannot copy selection"))?)
}
#[derive(Debug, Clone)]
pub struct PixelPreview {
    base: Pixmap,
    patch: Pixmap,
    background: String,
}
impl PixelPreview {
    pub fn new(doc: &Document, rect: PixelRect) -> Result<Self> {
        rect.validate(doc)?;
        let mut base = render_transparent(doc)?;
        let patch = base
            .clone_rect(
                tiny_skia::IntRect::from_xywh(
                    rect.x as i32,
                    rect.y as i32,
                    rect.width,
                    rect.height,
                )
                .unwrap(),
            )
            .unwrap();
        for y in rect.y..rect.y + rect.height {
            let start = (y * doc.width + rect.x) as usize;
            base.pixels_mut()[start..start + rect.width as usize]
                .fill(tiny_skia::PremultipliedColorU8::TRANSPARENT);
        }
        Ok(Self {
            base,
            patch,
            background: doc.background.clone(),
        })
    }
    pub fn transparent(&self, destination: Option<(i32, i32)>) -> Result<Pixmap> {
        let mut pixels = self.base.clone();
        if let Some((x, y)) = destination {
            ensure!(
                (-16384..=16384).contains(&x) && (-16384..=16384).contains(&y),
                "Invalid destination"
            );
            pixels.draw_pixmap(
                x,
                y,
                self.patch.as_ref(),
                &Default::default(),
                Transform::identity(),
                None,
            );
        }
        Ok(pixels)
    }
    /// Pointer-move preview avoids PNG encoding and repeated source decoding.
    pub fn paint(&self, destination: (i32, i32)) -> Result<Pixmap> {
        crate::render::with_background(self.transparent(Some(destination))?, &self.background)
    }
}
pub fn edit(doc: &Document, rect: PixelRect, destination: Option<(i32, i32)>) -> Result<Document> {
    let pixels = PixelPreview::new(doc, rect)?.transparent(destination)?;
    let mut next = doc.clone();
    next.operations = vec![image_operation(&pixels)?];
    next.validate()?;
    Ok(next)
}
