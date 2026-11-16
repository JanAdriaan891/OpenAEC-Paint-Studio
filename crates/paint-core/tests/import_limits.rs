use image::{codecs::webp::WebPDecoder, ImageDecoder, ImageError, Limits};
use paint_core::{codec, render, render::rgba};
use std::io::Cursor;

// Fail the test binary before a regression can actually reserve gigabytes.
// Native-only: the browser acceptance uses the same fixture after this suite passes.
#[cfg(not(target_arch = "wasm32"))]
mod allocation_guard {
    use std::alloc::{GlobalAlloc, Layout, System};
    struct Guard;
    fn check(size: usize) {
        if size > 320_000_000 {
            std::process::exit(86);
        }
    }
    unsafe impl GlobalAlloc for Guard {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            check(layout.size());
            System.alloc(layout)
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            check(layout.size());
            System.alloc_zeroed(layout)
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            check(size);
            System.realloc(ptr, layout, size)
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout)
        }
    }
    #[global_allocator]
    static ALLOCATOR: Guard = Guard;
}

#[test]
fn webp_huge_exif_returns_a_limit_error_even_with_a_misleading_filename() {
    let bytes = include_bytes!("fixtures/webp-exif-overflow.webp");
    assert_eq!(bytes.len(), 64);
    for name in ["received.webp", "received.png"] {
        let error = codec::import_image(bytes, name).unwrap_err();
        assert!(
            matches!(
                error.downcast_ref::<ImageError>(),
                Some(ImageError::Limits(_))
            ),
            "{error}"
        );
    }
    let mut control = bytes.to_vec();
    control[56..60].copy_from_slice(&4u32.to_le_bytes());
    let document = codec::import_image(&control, "control.webp").unwrap();
    assert_eq!((document.width, document.height), (1, 1));
}

fn webp_metadata(tag: &[u8; 4], flags: u8) -> Vec<u8> {
    let mut bytes = include_bytes!("fixtures/webp-exif-overflow.webp").to_vec();
    bytes[20] = flags;
    bytes[52..56].copy_from_slice(tag);
    bytes[56..60].copy_from_slice(&4u32.to_le_bytes());
    bytes
}

#[test]
fn webp_forwards_limits_for_all_metadata_readers_and_checks_dimensions() {
    for (tag, flag) in [(b"EXIF", 8), (b"ICCP", 32), (b"XMP ", 4)] {
        let bytes = webp_metadata(tag, flag);
        let mut decoder = WebPDecoder::new(Cursor::new(&bytes)).unwrap();
        let mut limits = Limits::default();
        limits.max_alloc = Some(3);
        decoder.set_limits(limits).unwrap();
        let error = match tag {
            b"EXIF" => decoder.exif_metadata(),
            b"ICCP" => decoder.icc_profile(),
            _ => decoder.xmp_metadata(),
        }
        .unwrap_err();
        assert!(matches!(error, ImageError::Limits(_)), "{tag:?}: {error}");
        let mut limits = Limits::default();
        limits.max_alloc = Some(4);
        decoder.set_limits(limits).unwrap();
        let data = match tag {
            b"EXIF" => decoder.exif_metadata(),
            b"ICCP" => decoder.icc_profile(),
            _ => decoder.xmp_metadata(),
        }
        .unwrap()
        .unwrap();
        assert_eq!(data, [0x49, 0x49, 0x2a, 0]);
    }
    let mut decoder = WebPDecoder::new(Cursor::new(webp_metadata(b"EXIF", 8))).unwrap();
    let mut limits = Limits::default();
    limits.max_image_width = Some(0);
    assert!(matches!(
        decoder.set_limits(limits),
        Err(ImageError::Limits(_))
    ));
}

#[test]
fn valid_webp_preserves_alpha_and_exif_orientation() {
    let pixels = vec![255, 0, 0, 128, 0, 0, 255, 255];
    let image = image::RgbaImage::from_raw(2, 1, pixels.clone()).unwrap();
    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut encoded, image::ImageFormat::WebP)
        .unwrap();
    // Extended container, alpha + EXIF flags, width 2 and height 1.
    let mut bytes = b"RIFF\0\0\0\0WEBPVP8X\x0a\0\0\0\x18\0\0\0\x01\0\0\0\0\0".to_vec();
    bytes.extend_from_slice(&encoded.get_ref()[12..]);
    let exif = [
        0x49, 0x49, 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 1, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
    ];
    bytes.extend_from_slice(b"EXIF");
    bytes.extend_from_slice(&(exif.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&exif);
    let size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    let document = codec::import_image(&bytes, "oriented.webp").unwrap();
    assert_eq!((document.width, document.height), (1, 2));
    assert_eq!(rgba(&render(&document).unwrap()), pixels);
}

#[test]
fn bmp_rejects_offsets_in_headers_palettes_and_bitmasks() {
    for (name, start) in [
        ("bmp-rgb24.bmp", 54),
        ("bmp-pal8.bmp", 62),
        ("bmp-bitfields16.bmp", 66),
        ("bmp-v4-bitfields.bmp", 122),
        ("bmp-v5-bitfields.bmp", 138),
        ("bmp-core-pal8.bmp", 794),
    ] {
        let fixture = std::fs::read(format!(
            "{}/tests/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        for offset in [0u32, 1, 13, 14, start - 1] {
            let mut bytes = fixture.clone();
            bytes[10..14].copy_from_slice(&offset.to_le_bytes());
            let error = codec::import_image(&bytes, "disguised.png").unwrap_err();
            assert!(
                error.to_string().contains("BMP pixel offset"),
                "{name} offset {offset}: {error}"
            );
        }
    }
}

#[test]
fn valid_bmp_headers_palettes_masks_and_pixel_gaps_keep_exact_colors() {
    for (name, start) in [
        ("bmp-rgb24.bmp", 54),
        ("bmp-pal8.bmp", 62),
        ("bmp-bitfields16.bmp", 66),
        ("bmp-v4-bitfields.bmp", 122),
        ("bmp-v5-bitfields.bmp", 138),
        ("bmp-core-pal8.bmp", 794),
    ] {
        let fixture = std::fs::read(format!(
            "{}/tests/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        for gap in [0, 7] {
            let mut bytes = fixture.clone();
            bytes.splice(start..start, vec![0; gap]);
            bytes[10..14].copy_from_slice(&((start + gap) as u32).to_le_bytes());
            let size = bytes.len() as u32;
            bytes[2..6].copy_from_slice(&size.to_le_bytes());
            let document =
                codec::import_image(&bytes, name).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(
                rgba(&render(&document).unwrap())[..8],
                [255, 0, 0, 255, 0, 0, 255, 255],
                "{name} gap {gap}"
            );
        }
    }
}

#[test]
fn ico_without_a_bmp_file_header_still_imports() {
    let document =
        codec::import_image(include_bytes!("fixtures/bmp-icon.ico"), "icon.ico").unwrap();
    assert_eq!((document.width, document.height), (2, 1));
    assert_eq!(
        rgba(&render(&document).unwrap()),
        [255, 0, 0, 255, 0, 0, 255, 255]
    );
}
