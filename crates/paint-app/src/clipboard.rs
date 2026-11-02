//! Image clipboard adapters. Keep the native clipboard owner alive on Linux.
#[cfg(not(target_arch = "wasm32"))]
fn native() -> Result<std::sync::MutexGuard<'static, Option<arboard::Clipboard>>, String> {
    static CLIPBOARD: std::sync::OnceLock<std::sync::Mutex<Option<arboard::Clipboard>>> =
        std::sync::OnceLock::new();
    let mut guard = CLIPBOARD
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
    }
    Ok(guard)
}
pub async fn read() -> Result<Vec<u8>, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let image = native()?
            .as_mut()
            .unwrap()
            .get_image()
            .map_err(|e| format!("Geen ondersteunde afbeelding op het klembord: {e}"))?;
        let width = u32::try_from(image.width).map_err(|e| e.to_string())?;
        let height = u32::try_from(image.height).map_err(|e| e.to_string())?;
        paint_core::validate_dimensions(width, height).map_err(|e| format!("Klembord: {e}"))?;
        paint_core::render::pixmap_from_rgba(width, height, &image.bytes)
            .and_then(|p| Ok(p.encode_png()?))
            .map_err(|e| e.to_string())
    }
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        let fail = |v| {
            format!("Klembord niet beschikbaar: {v:?}. Geef toestemming of open de afbeelding als bestand.")
        };
        let clipboard = browser_clipboard()?;
        let items = js_sys::Array::from(&JsFuture::from(clipboard.read()).await.map_err(fail)?);
        for value in items {
            let item = value.dyn_into::<web_sys::ClipboardItem>().map_err(fail)?;
            if item
                .types()
                .iter()
                .any(|t| t.as_string().as_deref() == Some("image/png"))
            {
                let blob = JsFuture::from(item.get_type("image/png"))
                    .await
                    .map_err(fail)?
                    .dyn_into::<web_sys::Blob>()
                    .map_err(fail)?;
                if blob.size() > paint_core::MAX_BYTES as f64 {
                    return Err("Klembordafbeelding is groter dan 64 MB".into());
                }
                return Ok(js_sys::Uint8Array::new(
                    &JsFuture::from(blob.array_buffer()).await.map_err(fail)?,
                )
                .to_vec());
            }
        }
        Err(
            "Geen PNG-afbeelding op het klembord. Kopieer een afbeelding of open een bestand."
                .into(),
        )
    }
}
pub async fn write(png: Vec<u8>) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let pixels = paint_core::codec::decode_png(&png).map_err(|e| e.to_string())?;
        native()?
            .as_mut()
            .unwrap()
            .set_image(arboard::ImageData {
                width: pixels.width() as usize,
                height: pixels.height() as usize,
                bytes: std::borrow::Cow::Owned(paint_core::render::rgba(&pixels)),
            })
            .map_err(|e| e.to_string())
    }
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsValue;
        let fail =
            |v| format!("Kopiëren geweigerd: {v:?}. Geef klembordtoestemming of exporteer PNG.");
        let parts = js_sys::Array::new();
        parts.push(&js_sys::Uint8Array::from(png.as_slice()));
        let blob = {
            let options = web_sys::BlobPropertyBag::new();
            options.set_type("image/png");
            web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(fail)?
        };
        let record = js_sys::Object::new();
        js_sys::Reflect::set(
            &record,
            &JsValue::from_str("image/png"),
            &js_sys::Promise::resolve(&blob),
        )
        .map_err(fail)?;
        let item = web_sys::ClipboardItem::new_with_record_from_str_to_blob_promise(&record)
            .map_err(fail)?;
        let items = js_sys::Array::new();
        items.push(&item);
        wasm_bindgen_futures::JsFuture::from(browser_clipboard()?.write(items.as_ref()))
            .await
            .map_err(fail)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
fn browser_clipboard() -> Result<web_sys::Clipboard, String> {
    use wasm_bindgen::{JsCast, JsValue};
    let navigator = web_sys::window().ok_or("Browser ontbreekt")?.navigator();
    let value = js_sys::Reflect::get(navigator.as_ref(), &JsValue::from_str("clipboard"))
        .map_err(|e| format!("Klembord: {e:?}"))?;
    if value.is_null() || value.is_undefined() {
        return Err(
            "Klembord ontbreekt. Gebruik HTTPS/localhost of open/exporteer een bestand.".into(),
        );
    }
    value.dyn_into().map_err(|_| "Klembord ontbreekt".into())
}
