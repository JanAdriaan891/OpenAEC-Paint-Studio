use paint_core::{
    codec,
    edit::{Align, Editor},
    project,
    selection::{self, PixelRect},
    *,
};
fn scene() -> Document {
    let mut doc = Document::new(160, 100, "Roundtrip");
    doc.background = "transparent".into();
    doc.operations.push(Operation {
        kind: Kind::Text,
        text: Some("First\nSecond".into()),
        at: Some(Point::new(10., 10.)),
        opacity: 0.7,
        size: 18.,
        transform: Some([1., 0.1, 0., 1., 2., 3.]),
        ..Default::default()
    });
    for (x, y) in [(30., 40.), (90., 20.)] {
        doc.operations.push(Operation {
            kind: Kind::Rectangle,
            from: Some(Point::new(x, y)),
            to: Some(Point::new(x + 25., y + 25.)),
            color: "#d97706".into(),
            filled: true,
            ..Default::default()
        });
    }
    doc
}
#[test]
fn portable_project_preserves_every_object_setting_and_pixel() {
    let mut doc = scene();
    let mut source = Document::new(16, 12, "Image");
    source.background = "#2563eb".into();
    let mut op = codec::import_image(&png(&source).unwrap(), "photo.png")
        .unwrap()
        .operations
        .remove(0);
    op.opacity = 0.5;
    op.transform = Some([1., 0., 0., 1., 120., 70.]);
    doc.operations.push(op);
    let restored = project::read(&project::write(&doc).unwrap()).unwrap();
    assert_eq!(doc, restored);
    assert_eq!(png(&doc).unwrap(), png(&restored).unwrap());
    assert_eq!(
        restored.operations[0].text.as_deref(),
        Some("First\nSecond")
    );
    let mut v: serde_json::Value = serde_json::from_slice(&project::write(&doc).unwrap()).unwrap();
    v["version"] = serde_json::json!(2);
    assert!(project::read(&serde_json::to_vec(&v).unwrap()).is_err());
    assert!(project::read(b"{\"format\":\"openaec-paint-project\"").is_err());
    assert_eq!(
        codec::read_ifcx(&codec::write_ifcx(&doc, false).unwrap())
            .unwrap()
            .operations
            .len(),
        1
    );
}
#[test]
fn import_reports_original_and_result_without_changing_limits() {
    let bytes = image::DynamicImage::new_rgb8(6000, 4000);
    let mut out = std::io::Cursor::new(Vec::new());
    bytes.write_to(&mut out, image::ImageFormat::Png).unwrap();
    let result = codec::import_image_with_info(out.get_ref(), "large.png").unwrap();
    assert_eq!(
        result.original,
        Size {
            width: 6000,
            height: 4000
        }
    );
    assert!(result.resized());
    result.document.validate().unwrap();
    assert_eq!(
        result.document.size(),
        Size {
            width: 3546,
            height: 2364
        }
    );
    let small = codec::import_image_with_info(&png(&scene()).unwrap(), "small.png").unwrap();
    assert!(!small.resized());
}
#[test]
fn pixel_move_clear_alpha_clipping_and_single_step_undo() {
    let doc = scene();
    let rect = PixelRect {
        x: 30,
        y: 40,
        width: 25,
        height: 25,
    };
    let moved = selection::edit(&doc, rect, Some((60, 60))).unwrap();
    assert_eq!(moved.operations.len(), 1);
    assert_eq!(render(&moved).unwrap().pixel(40, 50).unwrap().alpha(), 0);
    assert!(render(&moved).unwrap().pixel(70, 70).unwrap().alpha() > 0);
    let mut editor = Editor::new(doc.clone());
    editor.commit(moved.clone()).unwrap();
    editor.undo();
    assert_eq!(editor.document, doc);
    editor.redo();
    assert_eq!(editor.document, moved);
    let deleted = selection::edit(&doc, rect, None).unwrap();
    assert_eq!(render(&deleted).unwrap().pixel(40, 50).unwrap().alpha(), 0);
    assert!(selection::edit(&doc, rect, Some((-10, 95))).is_ok());
    assert!(selection::copy(
        &doc,
        PixelRect {
            x: 159,
            y: 99,
            width: 2,
            height: 2
        }
    )
    .is_err());
    assert_eq!(
        PixelRect::between(&doc, Point::new(-100., -100.), Point::new(200., 200.)).unwrap(),
        PixelRect {
            x: 0,
            y: 0,
            width: 160,
            height: 100
        }
    );
}
#[test]
fn all_six_alignments_preserve_transformed_subset_and_history() {
    for align in [
        Align::Left,
        Align::Center,
        Align::Right,
        Align::Top,
        Align::Middle,
        Align::Bottom,
    ] {
        let mut doc = scene();
        doc.operations[1].transform = Some([0., 1., -1., 0., 100., 0.]);
        let mut editor = Editor::new(doc.clone());
        editor.selection = vec![0, 1, 2];
        editor.align(align).unwrap();
        let after = editor.document.clone();
        let bounds: Vec<_> = after
            .operations
            .iter()
            .map(|o| o.bounds(after.size()).unwrap())
            .collect();
        let coordinate = |b: Bounds| match align {
            Align::Left => b.x,
            Align::Center => b.x + b.width / 2.,
            Align::Right => b.x + b.width,
            Align::Top => b.y,
            Align::Middle => b.y + b.height / 2.,
            Align::Bottom => b.y + b.height,
        };
        for b in &bounds[1..] {
            assert!((coordinate(*b) - coordinate(bounds[0])).abs() < 0.01);
        }
        for (a, b) in after.operations.iter().zip(&doc.operations) {
            let mut a = a.clone();
            a.transform = b.transform;
            assert_eq!(&a, b);
        }
        editor.undo();
        assert_eq!(editor.document, doc);
        editor.redo();
        assert_eq!(editor.document, after);
        editor.selection = vec![0];
        assert!(editor.align(align).is_err());
        editor.selection = vec![1, 2];
        editor.align(align).unwrap();
        assert_eq!(editor.document.operations[0], after.operations[0]);
    }
}

#[test]
fn cached_pixel_preview_matches_committed_pixels_on_both_backgrounds() {
    for background in ["transparent", "#ffffff"] {
        let mut doc = scene();
        doc.background = background.into();
        let rect = PixelRect {
            x: 30,
            y: 40,
            width: 25,
            height: 25,
        };
        let cache = selection::PixelPreview::new(&doc, rect).unwrap();
        for dest in [(60, 60), (-10, 95)] {
            let preview = cache.paint(dest).unwrap();
            let edited = selection::edit(&doc, rect, Some(dest)).unwrap();
            assert_eq!(preview.data(), render(&edited).unwrap().data());
        }
    }
}
