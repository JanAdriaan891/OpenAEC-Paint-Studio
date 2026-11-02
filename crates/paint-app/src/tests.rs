use super::*;
use iced::advanced::{
    layout::{self, Layout},
    renderer,
    widget::Tree,
};
use iced::{mouse, Color, Rectangle};
use openaec_ui::design::a11y::Collect;
fn fonts() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        openaec_ui::fonts::install_fallback();
        let mut s = iced_tiny_skia::graphics::text::font_system()
            .write()
            .unwrap();
        for b in openaec_ui::fonts::FILES {
            s.load_font(std::borrow::Cow::Borrowed(b));
        }
    });
}
fn example() -> App {
    let mut doc = Document::new(1200, 800, "OpenAEC Paint");
    doc.operations.push(Operation {
        kind: Kind::Rectangle,
        from: Some(paint_core::Point::new(180., 170.)),
        to: Some(paint_core::Point::new(920., 590.)),
        color: "#d97706".into(),
        size: 5.,
        ..Default::default()
    });
    doc.operations.push(Operation {
        kind: Kind::Text,
        at: Some(paint_core::Point::new(220., 220.)),
        text: Some("OpenAEC Paint Studio\nHelemaal Rust".into()),
        size: 40.,
        color: "#36363e".into(),
        ..Default::default()
    });
    let mut app = App::with_document(doc);
    app.settings.theme = ThemeId::Light;
    app
}
fn render(app: &App, size: Size) -> tiny_skia::Pixmap {
    fonts();
    let mut element = app.view();
    let mut renderer = iced_tiny_skia::Renderer::new(openaec_ui::fonts::BODY, iced::Pixels(12.));
    let mut tree = Tree::new(&element);
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, size),
    );
    let mut messages = Vec::new();
    let mut shell = iced::advanced::Shell::new(&mut messages);
    let redraw = iced::Event::Window(iced::window::Event::RedrawRequested(
        std::time::Instant::now(),
    ));
    element.as_widget_mut().update(
        &mut tree,
        &redraw,
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &renderer,
        &mut iced::advanced::clipboard::Null,
        &mut shell,
        &Rectangle::with_size(size),
    );
    let theme = app.settings.theme.iced();
    element.as_widget().draw(
        &tree,
        &mut renderer,
        &theme,
        &renderer::Style {
            text_color: theme.palette().text,
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(size),
    );
    let (width, height) = (size.width as u32, size.height as u32);
    let mut bytes = vec![0; (width * height * 4) as usize];
    let mut p = tiny_skia::PixmapMut::from_bytes(&mut bytes, width, height).unwrap();
    let mut mask = tiny_skia::Mask::new(width, height).unwrap();
    renderer.draw(
        &mut p,
        &mut mask,
        &iced_tiny_skia::graphics::Viewport::with_physical_size(Size::new(width, height), 1.),
        &[Rectangle::with_size(size)],
        Color::from_rgb8(255, 0, 255),
    );
    for p in bytes.chunks_exact_mut(4) {
        p.swap(0, 2);
    }
    tiny_skia::Pixmap::from_vec(bytes, tiny_skia::IntSize::from_wh(width, height).unwrap()).unwrap()
}
fn accessibility(app: &App) -> Collect {
    fonts();
    let mut e = app.view();
    let r = iced_tiny_skia::Renderer::new(openaec_ui::fonts::BODY, iced::Pixels(12.));
    let mut t = Tree::new(&e);
    let node = e
        .as_widget_mut()
        .layout(&mut t, &r, &layout::Limits::new(Size::ZERO, app.width));
    let mut found = Collect::default();
    e.as_widget_mut()
        .operate(&mut t, Layout::new(&node), &r, &mut found);
    found
}
#[test]
fn adaptive_ribbon_controls_stay_inside_every_supported_width() {
    for width in [560., 640., 720., 800., 900., 1024., 1280., 1440., 1920.] {
        let mut app = example();
        app.width = Size::new(width, 900.);
        app.fit();
        let found = accessibility(&app);
        let ribbon: Vec<_> = found
            .nodes
            .iter()
            .filter(|f| f.bounds.y >= 70. && f.bounds.y < 186. && f.bounds.height < 80.)
            .collect();
        assert!(!ribbon.is_empty());
        for f in ribbon {
            assert!(
                f.bounds.x >= -0.5 && f.bounds.x + f.bounds.width <= width + 0.5,
                "{} at width {width}: {:?}",
                f.node.name,
                f.bounds
            );
        }
    }
}
#[test]
fn collapsed_group_menus_expose_their_commands() {
    let mut app = example();
    app.width = Size::new(560., 800.);
    for index in 0..6 {
        app.menu = Some(index);
        let found = accessibility(&app);
        assert!(found
            .nodes
            .iter()
            .any(|n| n.bounds.y > 190. && n.node.name.len() > 2));
        for n in &found.nodes {
            assert!(
                n.bounds.x + n.bounds.width <= 564.,
                "menu {index}: {} {:?}",
                n.node.name,
                n.bounds
            );
        }
    }
}
#[test]
fn escape_cancels_uncommitted_drawing_and_switches_to_select() {
    let mut app = example();
    let before = app.editor.document.clone();
    let _ = app.update(Msg::Canvas(
        Input::Down(Point::new(400., 300.), false),
        app.canvas_size,
    ));
    let _ = app.update(Msg::Canvas(
        Input::Move(Point::new(450., 350.)),
        app.canvas_size,
    ));
    let _ = app.update(Msg::Command(Command::Cancel));
    assert_eq!(app.tool, Tool::Select);
    assert_eq!(app.editor.document, before);
    assert!(app.preview.is_none());
}
#[test]
fn wheel_zoom_and_middle_pan_do_not_edit_the_document() {
    let mut app = example();
    let before = app.editor.document.clone();
    let initial = app.zoom;
    app.canvas(Input::Wheel(40., Point::new(300., 300.), true));
    assert_eq!(app.zoom, initial + 10.);
    app.canvas(Input::Down(Point::new(300., 300.), true));
    let pan = app.pan;
    app.canvas(Input::Move(Point::new(360., 330.)));
    assert_eq!(app.pan, pan + Vector::new(60., 30.));
    app.canvas(Input::Up);
    assert_eq!(app.editor.document, before);
}
#[test]
fn render_screens() {
    let output = std::env::var_os("PAINT_SCREENSHOTS").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        std::fs::create_dir_all(dir).unwrap();
    }
    for (label, width, theme) in [
        ("light-wide", 1600., ThemeId::Light),
        ("dark-wide", 1600., ThemeId::Dark),
        ("blueprint-wide", 1600., ThemeId::Blueprint),
        ("compact-900", 900., ThemeId::Light),
        ("collapsed-560", 560., ThemeId::Light),
    ] {
        let mut app = example();
        app.width = Size::new(width, 900.);
        app.settings.theme = theme;
        app.fit();
        let image = render(&app, app.width);
        assert_eq!(image.width(), width as u32);
        if let Some(dir) = &output {
            image.save_png(dir.join(format!("{label}.png"))).unwrap();
        }
    }
}

#[test]
fn text_that_exceeds_document_limits_remains_available_for_correction() {
    let mut app = example();
    let before = app.editor.document.clone();
    let text = "x".repeat(2001);
    app.text_box = Some(TextDraft {
        content: text_editor::Content::with_text(&text),
        at: paint_core::Point::new(50., 50.),
        index: None,
    });
    let _ = app.update(Msg::Command(Command::ApplyText));
    assert_eq!(app.editor.document, before);
    assert_eq!(
        app.text_box
            .as_ref()
            .unwrap()
            .content
            .text()
            .trim_end_matches('\n'),
        text
    );
}

#[test]
fn cancelling_resized_import_preserves_document_dirty_and_draft() {
    let mut app = example();
    app.editor.dirty = true;
    let before = app.editor.document.clone();
    app.pending_import = Some(paint_core::codec::ImageImport {
        document: Document::new(100, 100, "Other"),
        original: paint_core::Size {
            width: 6000,
            height: 4000,
        },
    });
    app.dialog = Some(Dialog::ImportResize);
    let _ = app.update(Msg::Dialog(None));
    assert_eq!(app.editor.document, before);
    assert!(app.editor.dirty);
    assert!(app.pending_draft.is_none());
    assert!(app.pending_import.is_none());
}
#[test]
fn save_and_clipboard_failures_never_clear_dirty_or_replace_work() {
    let mut app = example();
    app.editor.dirty = true;
    let before = app.editor.document.clone();
    let _ = app.update(Msg::Saved(Ok(false), Some(before.clone())));
    assert!(app.editor.dirty);
    let _ = app.update(Msg::Saved(Err("disk full".into()), Some(before.clone())));
    assert!(app.editor.dirty);
    let _ = app.update(Msg::Saved(Ok(true), None));
    assert!(app.editor.dirty, "export is not project save");
    let mut older = before.clone();
    older.name = "Older".into();
    let _ = app.update(Msg::Saved(Ok(true), Some(older)));
    assert!(
        app.editor.dirty,
        "a stale save cannot mark new changes saved"
    );
    let _ = app.update(Msg::ClipboardRead(Err("denied".into())));
    let _ = app.update(Msg::ClipboardWritten(
        Err("denied".into()),
        Some(Document::default()),
    ));
    assert_eq!(app.editor.document, before);
    assert!(app.editor.dirty);
    let _ = app.update(Msg::Saved(Ok(true), Some(before)));
    assert!(!app.editor.dirty);
}
#[test]
fn paste_adds_movable_image_and_is_one_undo_step() {
    let mut app = example();
    let before = app.editor.document.clone();
    let bytes = paint_core::png(&Document::new(40, 30, "Clipboard")).unwrap();
    let _ = app.update(Msg::ClipboardRead(Ok(bytes)));
    assert_eq!(app.tool, Tool::Select);
    assert_eq!(
        app.editor.document.operations.len(),
        before.operations.len() + 1
    );
    let image = app.editor.document.operations.last().unwrap();
    assert_eq!(image.kind, Kind::Image);
    assert!(image.transform.is_some());
    let _ = app.update(Msg::Command(Command::Undo));
    assert_eq!(app.editor.document, before);
}
#[test]
fn pixel_selection_escape_and_edit_restore_vectors_with_undo() {
    let mut app = example();
    let before = app.editor.document.clone();
    let _ = app.update(Msg::Tool(Tool::Pixels));
    assert_eq!(app.dialog, Some(Dialog::PixelMode));
    let _ = app.update(Msg::PixelMode);
    assert_eq!(app.tool, Tool::Pixels);
    app.pixel_selection = Some(paint_core::selection::PixelRect {
        x: 170,
        y: 160,
        width: 100,
        height: 100,
    });
    let _ = app.update(Msg::Command(Command::Delete));
    assert_eq!(app.editor.document.operations.len(), 1);
    let _ = app.update(Msg::Command(Command::Undo));
    assert_eq!(app.editor.document, before);
    let _ = app.update(Msg::Command(Command::Cancel));
    assert_eq!(app.tool, Tool::Select);
    assert!(app.pixel_selection.is_none());
}
#[test]
fn new_dialogs_render_at_560px_and_allow_keyboard_object_access() {
    let mut app = example();
    app.width = Size::new(560., 800.);
    app.fit();
    for dialog in [Dialog::Objects, Dialog::PixelMode, Dialog::ImportResize] {
        app.dialog = Some(dialog);
        app.pending_import = Some(paint_core::codec::ImageImport {
            document: Document::new(3547, 2364, "Large"),
            original: paint_core::Size {
                width: 6000,
                height: 4000,
            },
        });
        let found = accessibility(&app);
        for n in &found.nodes {
            assert!(
                n.bounds.x >= -0.5 && n.bounds.x + n.bounds.width <= 560.5,
                "{} {:?}",
                n.node.name,
                n.bounds
            );
        }
        let image = render(&app, app.width);
        assert_eq!(image.width(), 560);
    }
    app.dialog = None;
    let _ = app.update(Msg::Key(
        iced::keyboard::Key::Named(iced::keyboard::key::Named::F6),
        iced::keyboard::Modifiers::empty(),
    ));
    assert_eq!(app.dialog, Some(Dialog::Objects));
}

#[test]
fn larger_clipboard_expands_canvas_without_replacing_existing_objects() {
    let mut app = example();
    let before = app.editor.document.clone();
    let bytes = paint_core::png(&Document::new(1600, 900, "Large clipboard")).unwrap();
    let _ = app.update(Msg::ClipboardRead(Ok(bytes)));
    assert_eq!(
        app.editor.document.size(),
        paint_core::Size {
            width: 1600,
            height: 900
        }
    );
    assert_eq!(&app.editor.document.operations[..2], &before.operations[..]);
    app.editor.undo();
    assert_eq!(app.editor.document, before);
}
#[test]
fn atomic_save_replaces_only_a_complete_file_and_cleans_failed_temporary_files() {
    let dir = std::env::temp_dir().join(format!("paint-save-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("work.paint.json");
    std::fs::write(&file, b"old project").unwrap();
    platform::atomic_write(&file, b"complete new project").unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), b"complete new project");
    let folder = dir.join("existing-directory");
    std::fs::create_dir(&folder).unwrap();
    assert!(platform::atomic_write(&folder, b"cannot replace directory").is_err());
    assert!(folder.is_dir());
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn recovered_draft_requires_portable_save_and_late_recovery_cannot_replace_an_opened_file() {
    let mut app = App::with_document(Document::default());
    let recovered = example().editor.document;
    let _ = app.update(Msg::DraftLoaded(Ok(Some(recovered.clone()))));
    assert_eq!(app.editor.document, recovered);
    assert!(app.editor.dirty);
    let mut opened = App::with_document(Document::new(80, 60, "Opened file"));
    opened.draft_generation = 1;
    let before = opened.editor.document.clone();
    let _ = opened.update(Msg::DraftLoaded(Ok(Some(recovered))));
    assert_eq!(opened.editor.document, before);
}
