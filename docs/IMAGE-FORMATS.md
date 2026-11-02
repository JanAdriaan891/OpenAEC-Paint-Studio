# Image formats

The default **Open** dialog filters image extensions. **Open IFCX** is a separate command for `.ifcx`/`.ifcxb`. Drag/drop and desktop filename arguments use the same importer. There is no `.oapaint` extension.

| Input | Decoder / behavior |
| --- | --- |
| PNG, JPEG, WebP, GIF, BMP, TIFF, ICO, TGA, PNM, QOI, HDR, EXR, DDS, farbfeld | Rust `image`; first frame/page; JPEG orientation applied |
| SVG, SVGZ | Rust resvg, rendered with embedded fonts; external file/URL images disabled |
| HEIC/HEIF | Rust heic-rs, supported HEVC still-image profiles |
| AVIF | Rust avif-decode/rav1d on desktop; browser ImageBitmap through a Rust adapter in Wasm |
| PSD | Rust psd, flattened composite; layers are not imported as editable objects |
| JPEG XL | Rust jxl-oxide, still image |
| JPEG 2000 (JP2/J2K) | Rust hayro-jpeg2000 |
| DNG and supported CR2/CR3, NEF/NRW, ARW, ORF, RW2, PEF, RAF, SRW, SR2, KDC, DCR | Rust Rawler, developed from sensor data; support depends on camera/variant |
| IFCX/IFCXB | OpenAEC embedded raster extension; see IFCX.md |

Format containers have many variants; unsupported profiles return an error. PSD v1 is supported; PSB is not guaranteed. Animated and multipage input becomes a single image. Color-managed editing and RAW development controls are not provided. No runtime image conversion service or ImageMagick download is needed.

Input is bounded to 64 MB. Sources are bounded to 64 megapixels (some decoder-specific allocation limits are tighter), then scaled with Lanczos to the canvas limits: at most 4096 pixels per side and 8,388,608 pixels total. Embedded PNG decoding checks dimensions before allocating the pixel buffer.

Output: PNG with alpha, JPEG on white, lossless WebP, SVG with retained geometry, and IFCX/IFCXB with one embedded PNG raster. SVG embeds raster operations; flood fill/eraser scenes export a raster composite to preserve appearance. Pixel corrections flatten the document and remain undoable. Geometry transforms preserve vector objects where possible.

Editable work is stored separately as `*.paint.json` using Save / Ctrl+S and Open Project / Ctrl+Shift+O. It preserves the v1 scene document; see [project storage](design/PROJECTS-AND-OBJECTS.md). Large image imports show original and resulting dimensions and require confirmation before replacing the current document.
