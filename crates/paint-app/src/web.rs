//! Browser integration. All editor behaviour remains in Rust.
use wasm_bindgen::{prelude::*, JsCast};
pub fn focus_canvas() {
    if let Some(canvas) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector("canvas").ok().flatten())
        .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
    {
        let _ = canvas.focus();
    }
}
pub fn install() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let listener = Closure::<dyn FnMut(web_sys::WheelEvent)>::new(|event: web_sys::WheelEvent| {
        if event.ctrl_key()
            && event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .is_some()
        {
            event.prevent_default();
        }
    });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(false);
    options.set_capture(true);
    let _ = window.add_event_listener_with_callback_and_add_event_listener_options(
        "wheel",
        listener.as_ref().unchecked_ref(),
        &options,
    );
    listener.forget();
    if let Some(doc) = window.document() {
        doc.set_title("OpenAEC Paint Studio");
    }
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .use_current_thread()
        .build_global();
}
/// AVIF uses the browser's bounded image bitmap decoder; editor logic stays Rust.
pub async fn avif_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use base64::Engine;
    let fail = |v: JsValue| format!("AVIF: {v:?}");
    let data = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&data);
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts).map_err(|e| fail(e.into()))?;
    let window = web_sys::window().ok_or("Browser ontbreekt")?;
    let bitmap = wasm_bindgen_futures::JsFuture::from(
        window
            .create_image_bitmap_with_blob(&blob)
            .map_err(|e| fail(e.into()))?,
    )
    .await
    .map_err(|e| fail(e.into()))?
    .dyn_into::<web_sys::ImageBitmap>()
    .map_err(|e| fail(e.into()))?;
    let (w, h) = (bitmap.width(), bitmap.height());
    if w == 0 || h == 0 || w as u64 * h as u64 > 64_000_000 {
        bitmap.close();
        return Err("Afbeelding is groter dan 64 megapixels".into());
    }
    let scale = (4096. / w as f64)
        .min(4096. / h as f64)
        .min((paint_core::MAX_PIXELS as f64 / (w as f64 * h as f64)).sqrt())
        .min(1.);
    let canvas = window
        .document()
        .ok_or("Document ontbreekt")?
        .create_element("canvas")
        .map_err(|e| fail(e.into()))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|e| fail(e.into()))?;
    canvas.set_width((w as f64 * scale).floor().max(1.) as u32);
    canvas.set_height((h as f64 * scale).floor().max(1.) as u32);
    let context = canvas
        .get_context("2d")
        .map_err(|e| fail(e.into()))?
        .ok_or("Canvas ontbreekt")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|e| fail(e.into()))?;
    context
        .draw_image_with_image_bitmap_and_dw_and_dh(
            &bitmap,
            0.,
            0.,
            canvas.width() as f64,
            canvas.height() as f64,
        )
        .map_err(|e| fail(e.into()))?;
    bitmap.close();
    let uri = canvas
        .to_data_url_with_type("image/png")
        .map_err(|e| fail(e.into()))?;
    base64::engine::general_purpose::STANDARD
        .decode(uri.split_once(',').ok_or("Ongeldige PNG")?.1)
        .map_err(|e| e.to_string())
}
