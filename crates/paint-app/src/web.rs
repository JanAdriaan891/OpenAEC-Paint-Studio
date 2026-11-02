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
    let keyboard =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(|event: web_sys::KeyboardEvent| {
            let canvas = event
                .target()
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .is_some();
            if canvas
                && (event.key() == "Tab"
                    || ((event.ctrl_key() || event.meta_key())
                        && matches!(
                            event.key().to_ascii_lowercase().as_str(),
                            "a" | "s" | "o" | "c" | "v" | "x" | "z" | "y"
                        )))
            {
                event.prevent_default();
            }
        });
    let _ = window.add_event_listener_with_callback_and_bool(
        "keydown",
        keyboard.as_ref().unchecked_ref(),
        true,
    );
    keyboard.forget();
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
    let canvas = window
        .document()
        .ok_or("Document ontbreekt")?
        .create_element("canvas")
        .map_err(|e| fail(e.into()))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|e| fail(e.into()))?;
    canvas.set_width(w);
    canvas.set_height(h);
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

/// A browser download has no disk-completion callback. Mark a project saved only
/// after explicit handoff, never after cancelling this dialog (rfd's Wasm writer
/// discards its cancel result). Native saving uses the actual file write result.
pub async fn save_download(bytes: &[u8], name: &str) -> Result<bool, String> {
    let fail = |e| format!("Download: {e:?}");
    let window = web_sys::window().ok_or("Browser ontbreekt")?;
    let document = window.document().ok_or("Document ontbreekt")?;
    let overlay = document.create_element("div").map_err(fail)?;
    overlay.set_id("paint-download");
    overlay.set_attribute("role", "dialog").map_err(fail)?;
    overlay.set_attribute("aria-modal", "true").map_err(fail)?;
    overlay
        .set_attribute("aria-label", "Bestand downloaden")
        .map_err(fail)?;
    let card = document.create_element("section").map_err(fail)?;
    let title = document.create_element("h3").map_err(fail)?;
    title.set_text_content(Some("Bestand downloaden"));
    card.append_child(&title).map_err(fail)?;
    let label = document.create_element("p").map_err(fail)?;
    label.set_text_content(Some(name));
    card.append_child(&label).map_err(fail)?;
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(if name.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    });
    let blob =
        web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(fail)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(fail)?;
    let link = document
        .create_element("a")
        .map_err(fail)?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|e| fail(e.into()))?;
    link.set_id("paint-download-link");
    link.set_href(&url);
    link.set_download(name);
    link.set_text_content(Some("Downloaden"));
    card.append_child(&link).map_err(fail)?;
    let cancel = document.create_element("button").map_err(fail)?;
    cancel.set_id("paint-download-cancel");
    cancel.set_text_content(Some("Annuleren"));
    card.append_child(&cancel).map_err(fail)?;
    overlay.append_child(&card).map_err(fail)?;
    document
        .body()
        .ok_or("Pagina ontbreekt")?
        .append_child(&overlay)
        .map_err(fail)?;
    let mut callbacks = None;
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let yes = resolve.clone();
        let no = resolve.clone();
        let escape = resolve.clone();
        let accept = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let _ = yes.call1(&JsValue::UNDEFINED, &JsValue::TRUE);
        });
        let reject = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let _ = no.call1(&JsValue::UNDEFINED, &JsValue::FALSE);
        });
        let key =
            Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                if e.key() == "Escape" {
                    e.prevent_default();
                    e.stop_propagation();
                    let _ = escape.call1(&JsValue::UNDEFINED, &JsValue::FALSE);
                }
            });
        let _ = link.add_event_listener_with_callback("click", accept.as_ref().unchecked_ref());
        let _ = cancel.add_event_listener_with_callback("click", reject.as_ref().unchecked_ref());
        let _ = overlay.add_event_listener_with_callback("keydown", key.as_ref().unchecked_ref());
        callbacks = Some((accept, reject, key));
    });
    let _ = link.focus();
    let result = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(fail)?;
    if let Some((accept, reject, key)) = callbacks {
        let _ = link.remove_event_listener_with_callback("click", accept.as_ref().unchecked_ref());
        let _ =
            cancel.remove_event_listener_with_callback("click", reject.as_ref().unchecked_ref());
        let _ =
            overlay.remove_event_listener_with_callback("keydown", key.as_ref().unchecked_ref());
    }
    overlay.remove();
    let cleanup = Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    });
    let _ =
        window.set_timeout_with_callback_and_timeout_and_arguments_0(cleanup.unchecked_ref(), 1000);
    focus_canvas();
    Ok(result.as_bool().unwrap_or(false))
}
