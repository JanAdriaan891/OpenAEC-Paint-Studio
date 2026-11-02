//! Each invocation is one isolated technical sample; see scripts/benchmark.py.
use paint_core::{
    codec,
    edit::{self, Align, Editor},
    project,
    selection::{self, PixelRect},
    *,
};
use std::{path::Path, time::Instant};
fn time<T>(f: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<(T, f64)> {
    let at = Instant::now();
    let value = f()?;
    Ok((value, at.elapsed().as_secs_f64() * 1000.))
}
fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("generate") {
        let w: u32 = args[3].parse()?;
        let h: u32 = args[4].parse()?;
        let image = image::RgbaImage::from_fn(w, h, |x, y| {
            image::Rgba([
                ((x * 7 + y * 3) % 256) as u8,
                ((x / 8 + y / 4) % 256) as u8,
                ((x + y * 5) % 256) as u8,
                if x < w / 8 { 128 } else { 255 },
            ])
        });
        image.save(&args[2])?;
        return Ok(());
    }
    let path = Path::new(
        args.get(1)
            .ok_or_else(|| anyhow::anyhow!("input PNG required"))?,
    );
    let input = std::fs::read(path)?;
    let (import, open_ms) = time(|| codec::import_image_with_info(&input, "benchmark.png"))?;
    let source = import.original;
    let resized = import.resized();
    let doc = import.document;
    let (png, export_ms) = time(|| paint_core::png(&doc))?;
    let (_, render_ms) = time(|| paint_core::render(&doc))?;
    let mut editor = Editor::new(doc.clone());
    let (_, annotate_ms) = time(|| {
        // Deterministic automation task, not a human task-time measurement.
        editor.commit(edit::crop(
            &editor.document,
            Bounds {
                x: 10.,
                y: 10.,
                width: doc.width as f32 - 20.,
                height: doc.height as f32 - 20.,
            },
        )?)?;
        for (i, x) in [30., 160., 290.].into_iter().enumerate() {
            editor.add(Operation {
                kind: Kind::Text,
                at: Some(Point::new(x, 50. + i as f32 * 15.)),
                text: Some(format!("Label {}", i + 1)),
                size: 24.,
                ..Default::default()
            })?;
        }
        editor.add(Operation {
            kind: Kind::Rectangle,
            from: Some(Point::new(20., 20.)),
            to: Some(Point::new(420., 120.)),
            color: "#d97706".into(),
            ..Default::default()
        })?;
        let n = editor.document.operations.len();
        editor.selection = vec![n - 4, n - 3, n - 2];
        editor.align(Align::Top)?;
        paint_core::png(&editor.document)?;
        Ok(())
    })?;
    let (project, project_write_ms) = time(|| project::write(&editor.document))?;
    let (restored, project_read_ms) = time(|| project::read(&project))?;
    anyhow::ensure!(restored == editor.document, "project roundtrip mismatch");
    let (preview, pixel_drag_prepare_ms) = time(|| {
        selection::PixelPreview::new(
            &editor.document,
            PixelRect {
                x: 30,
                y: 30,
                width: 120,
                height: 80,
            },
        )
    })?;
    let (_, pixel_drag_preview_ms) = time(|| preview.paint((180, 180)))?;
    let (_, pixel_move_ms) = time(|| {
        selection::edit(
            &editor.document,
            PixelRect {
                x: 30,
                y: 30,
                width: 120,
                height: 80,
            },
            Some((180, 180)),
        )
    })?;
    let (_, undo_redo_ms) = time(|| {
        editor.undo();
        editor.redo();
        Ok(())
    })?;
    #[cfg(target_os = "linux")]
    let peak_kib = std::fs::read_to_string("/proc/self/status")?
        .lines()
        .find_map(|s| {
            s.strip_prefix("VmHWM:")
                .and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
        });
    #[cfg(not(target_os = "linux"))]
    let peak_kib: Option<u64> = None;
    println!(
        "{}",
        serde_json::json!({"version":VERSION,"original":source,"canvas":doc.size(),"resized":resized,"input_bytes":input.len(),"output_bytes":png.len(),"project_bytes":project.len(),"peak_process_rss_kib":peak_kib,"timings_ms":{"open":open_ms,"render":render_ms,"export_png":export_ms,"automated_annotate_crop_align_export":annotate_ms,"project_write":project_write_ms,"project_read_and_validate_render":project_read_ms,"pixel_drag_prepare":pixel_drag_prepare_ms,"pixel_drag_preview":pixel_drag_preview_ms,"pixel_move_flatten":pixel_move_ms,"undo_redo":undo_redo_ms}})
    );
    Ok(())
}
