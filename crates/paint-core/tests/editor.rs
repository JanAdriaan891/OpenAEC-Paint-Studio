use paint_core::{codec::*, edit::*, *};
fn rect() -> Operation {
    Operation {
        kind: Kind::Rectangle,
        from: Some(Point::new(5., 5.)),
        to: Some(Point::new(50., 50.)),
        color: "#000000".into(),
        size: 2.,
        ..Default::default()
    }
}
fn pixel(doc: &Document, x: u32, y: u32) -> [u8; 4] {
    let p = render(doc).unwrap();
    let data = paint_core::render::rgba(&p);
    data[((y * doc.width + x) * 4) as usize..((y * doc.width + x) * 4 + 4) as usize]
        .try_into()
        .unwrap()
}
#[test]
fn validation_matches_existing_document_limits() {
    let mut d = Document::new(4096, 4096, "too big");
    assert!(d.validate().is_err());
    d.width = 100;
    assert!(d.validate().is_ok());
    let mut o = rect();
    o.size = f32::NAN;
    d.operations.push(o);
    assert!(d.validate().is_err());
    assert!(parse_color("#0x0000").is_err());
}
#[test]
fn closed_shape_contains_fill_and_eraser_reveals_background() {
    let mut d = Document::new(64, 64, "fill");
    d.operations = vec![
        rect(),
        Operation {
            kind: Kind::Fill,
            at: Some(Point::new(20., 20.)),
            color: "#d97706".into(),
            ..Default::default()
        },
    ];
    assert_eq!(pixel(&d, 20, 20), [217, 119, 6, 255]);
    assert_eq!(pixel(&d, 60, 60), [255, 255, 255, 255]);
    d.operations.push(Operation {
        kind: Kind::Eraser,
        points: vec![Point::new(20., 20.)],
        size: 10.,
        ..Default::default()
    });
    assert_eq!(pixel(&d, 20, 20), [255, 255, 255, 255]);
}
#[test]
fn transparent_png_and_single_dots_preserve_alpha() {
    let mut d = Document::new(64, 64, "alpha");
    d.background = "transparent".into();
    d.operations.push(Operation {
        points: vec![Point::new(20., 20.)],
        opacity: 0.5,
        color: "#2563eb".into(),
        ..Default::default()
    });
    let bytes = png(&d).unwrap();
    let p = decode_png(&bytes).unwrap();
    assert_eq!(p.pixel(0, 0).unwrap().alpha(), 0);
    assert!((126..=129).contains(&p.pixel(20, 20).unwrap().alpha()));
}
#[test]
fn raster_containers_roundtrip_pixels_and_alpha() {
    let mut d = Document::new(64, 64, "Raster test");
    d.background = "transparent".into();
    d.operations.push(rect());
    for binary in [false, true] {
        let bytes = write_ifcx(&d, binary).unwrap();
        let restored = read_ifcx(&bytes).unwrap();
        assert_eq!(restored.name, d.name);
        assert_eq!(png(&d).unwrap(), png(&restored).unwrap());
    }
}
#[test]
fn ifcx_rejects_metadata_mismatch_and_missing_image() {
    let d = Document::new(16, 16, "test");
    let mut v: serde_json::Value = serde_json::from_slice(&write_ifcx(&d, false).unwrap()).unwrap();
    v["data"][0]["attributes"][RASTER_ATTRIBUTE]["width"] = serde_json::json!(20);
    assert!(read_ifcx(&serde_json::to_vec(&v).unwrap()).is_err());
    v["data"] = serde_json::json!([]);
    assert!(read_ifcx(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn old_json_document_remains_compatible() {
    let d=Document::from_json(br##"{"format":"openaec-paint","version":1,"name":"Existing","width":64,"height":64,"background":"#ffffff","operations":[{"type":"text","at":{"x":5,"y":5},"text":"OpenAEC","size":16,"opacity":1,"color":"#36363e","fontFamily":"sans-serif","bold":true}]}"##).unwrap();
    assert!(d.retain_vectors);
    assert!(d.operations[0].bold);
    assert!(png(&d).is_ok());
}
#[test]
fn selection_moves_every_object_and_undo_restores_them() {
    let mut e = Editor::new(Document::new(64, 64, "select"));
    e.add(rect()).unwrap();
    e.add(Operation {
        kind: Kind::Text,
        at: Some(Point::new(10., 10.)),
        text: Some("Test".into()),
        ..Default::default()
    })
    .unwrap();
    let before = e.document.clone();
    e.select_all();
    e.transform_selection([1., 0., 0., 1., 10., 12.]).unwrap();
    assert!(e
        .document
        .operations
        .iter()
        .all(|o| o.transform.unwrap()[4] == 10.));
    e.undo();
    assert_eq!(e.document, before);
    e.select_all();
    e.delete_selection().unwrap();
    assert!(e.document.operations.is_empty());
    e.undo();
    assert_eq!(e.document, before);
}
#[test]
fn geometry_and_svg_keep_vector_operations() {
    let mut d = Document::new(64, 80, "vector");
    d.operations.push(rect());
    d = transform_document(&d, TransformAction::RotateRight).unwrap();
    assert_eq!((d.width, d.height), (80, 64));
    assert_eq!(d.operations[0].kind, Kind::Rectangle);
    let out = svg(&d).unwrap();
    assert!(out.contains("<rect"));
    assert!(!out.contains("<image"));
}
#[test]
fn cropping_keeps_fill_if_seed_is_outside_new_canvas() {
    let mut d = Document::new(64, 64, "crop");
    d.operations.push(Operation {
        kind: Kind::Fill,
        at: Some(Point::new(2., 2.)),
        color: "#d97706".into(),
        ..Default::default()
    });
    let next = crop(
        &d,
        Bounds {
            x: 16.,
            y: 16.,
            width: 32.,
            height: 32.,
        },
    )
    .unwrap();
    assert_eq!(pixel(&next, 1, 1), [217, 119, 6, 255]);
}
#[test]
fn filters_preserve_transparency() {
    let mut d = Document::new(64, 64, "invert");
    d.background = "transparent".into();
    let mut op = rect();
    op.filled = true;
    op.color = "#d97706".into();
    d.operations.push(op);
    let next = filter(&d, Filter::Invert).unwrap();
    assert_eq!(pixel(&next, 20, 20), [38, 136, 249, 255]);
    assert_eq!(pixel(&next, 60, 60), [0, 0, 0, 0]);
}
#[test]
fn image_filters_read_their_own_exports() {
    let d = Document::new(73, 41, "image");
    for ext in ["png", "jpeg", "webp"] {
        let bytes = export_image(&d, ext).unwrap();
        let next = import_image(&bytes, &format!("image.{ext}")).unwrap();
        assert_eq!((next.width, next.height), (73, 41));
    }
}
#[test]
fn image_png_dimensions_are_bounded_before_allocation() {
    let mut d = Document::new(64, 64, "source");
    let mut p = rect();
    p.filled = true;
    d.operations.push(p);
    let bytes = png(&d).unwrap();
    let mut malicious = bytes.clone();
    malicious[16..20].copy_from_slice(&100000u32.to_be_bytes());
    assert!(decode_png(&malicious).is_err());
}
#[test]
fn rust_decoders_open_independently_generated_image_formats() {
    for (name, bytes) in [
        ("blue.heic", include_bytes!("fixtures/blue.heic").as_slice()),
        ("blue.tiff", include_bytes!("fixtures/blue.tiff").as_slice()),
        ("blue.psd", include_bytes!("fixtures/blue.psd").as_slice()),
        ("blue.avif", include_bytes!("fixtures/blue.avif").as_slice()),
        ("blue.jxl", include_bytes!("fixtures/blue.jxl").as_slice()),
        ("blue.jp2", include_bytes!("fixtures/blue.jp2").as_slice()),
    ] {
        let doc = import_image(bytes, name).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!((doc.width, doc.height), (32, 24), "{name}");
        let p = pixel(&doc, 16, 12);
        assert!(p[2] > 220 && p[0] < 30 && p[1] < 30, "{name}: {p:?}");
    }
}

#[test]
fn develops_a_generated_camera_raw_dng() {
    let doc = import_image(include_bytes!("fixtures/sensor.dng"), "sensor.dng").unwrap();
    assert_eq!((doc.width, doc.height), (32, 24));
    let p = pixel(&doc, 16, 12);
    assert_eq!(p[3], 255);
    assert!(p[0] > 20 && p[1] > 20 && p[2] > 20, "{p:?}");
}

#[test]
fn image_object_opacity_is_composited() {
    let mut source = Document::new(16, 16, "source");
    source.background = "#0000ff".into();
    let mut doc = import_image(&export_image(&source, "png").unwrap(), "blue.png").unwrap();
    doc.operations[0].opacity = 0.5;
    let p = pixel(&doc, 8, 8);
    assert_eq!(p[2], 255);
    assert!((127..=128).contains(&p[3]));
}
