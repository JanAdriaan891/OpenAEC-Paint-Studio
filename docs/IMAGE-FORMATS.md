# Image formats, vectors and limits

## Opening files

The Open dialog accepts any extension so it does not hide less common image formats. Recognizing a file depends on its actual encoding and the bundled decoder, not just its extension.

Common browser formats (PNG, JPEG/JFIF, WebP, GIF, BMP, SVG, AVIF, ICO) try the native browser decoder first. Other formats use a bundled ImageMagick WebAssembly worker. Everything stays on the device; the API is not needed for image import. The first extended-format import loads and compiles the local decoder, which can take a few seconds.

The extended decoder includes TIFF, PSD, HEIC/HEIF, JPEG 2000, JPEG XL, TGA, EXR, PNM variants and camera RAW readers such as DNG, CR2 and NEF. The **File → Supported formats** list reports the installed decoder's reading capabilities. Individual format variants, compression methods, corrupt files or missing delegates can still fail; the app reports that error. This is broad image support, not a guarantee for every file format ever made. PDF/PostScript/document import is not advertised.

Verified with generated fixtures: PNG, JPEG, WebP, BMP, TIFF, GIF, PSD, ICO, TGA, AVIF and JPEG 2000. Browser integration tests also open TIFF and SVG. HEIC was additionally checked against the public [libheif example](https://github.com/strukturag/libheif/blob/master/examples/example.heic). Camera RAW variants are decoder capabilities, not independently tested camera models.

- Animated or multipage files open as a single image (first frame/page); animation and layer structures are not imported.
- Imported SVG and PSD images are rasterized. The vector-retention option applies to objects drawn/edited in Paint Studio.
- Up to 64 MB input, with a 64-megapixel source limit. The working canvas is scaled proportionally to at most 4096 px per side and 8 megapixels total. Very large files may also hit the decoder's memory/time limits.
- The working image uses 8-bit RGBA; HDR/deep-color/channel structures are not retained.
- Embedded project images use PNG and retain alpha. JPEG export composites transparent regions onto white; PNG/WebP retain alpha.

## Vector retention

**Keep vector objects** is on by default. Shapes, brush paths and text remain separate operations while editing and in the local draft. Choose **Select objects (V)** to move an object, drag the lower-right selection handle to resize it, or apply color/size/style from Properties. Delete removes the selected object. Choose Text and click existing text to edit its content directly on the canvas. Ctrl+Enter applies text.

Rotate, mirror and resize compose affine transforms with the objects, preserving geometry and text. Crop clips to the new document bounds. Turning vector retention off flattens the current image; subsequent drawing stays rasterized. Undo restores the previous document state.

Brightness/contrast/saturation and other color filters flatten the complete image. Flood fill is a pixel operation: when cropping or exporting SVG, content through the last flood fill is rasterized to preserve its appearance, while later vectors remain vectors. Erasing is represented with SVG masks. Save/Ctrl+S writes a raster snapshot in IFCX. Use SVG export for vector geometry. Local drafts retain editable scene operations; session undo/redo snapshots are not serialized.

## Workspace

Image and view shortcuts are on Home. Focus mode (button above the canvas, or F11) hides the ribbon and properties to enlarge the drawing area. The Fit button follows the available workspace; zoom controls remain in the status bar. The properties panel can also be toggled independently from Home.
