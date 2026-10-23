# Raster images in IFCX

Export **IFCX (raster)** or **IFCXB (raster)** from the ribbon's Export menu. The complete visible canvas is rendered to a PNG with its background/alpha, then embedded in the file. The drawing in Paint Studio remains editable; the exported payload is a raster snapshot.

The JSON document uses the IFCX alpha envelope (`header.ifcxVersion: ifcx_alpha`, `imports`, `schemas`, `data`). One node at `/paint/image` carries the inline, object-valued `openaec::paint::rasterImage` attribute:

```json
{
  "version": 1,
  "name": "Image name",
  "width": 1200,
  "height": 800,
  "mimeType": "image/png",
  "uri": "data:image/png;base64,..."
}
```

Width and height are pixels. All image bytes are embedded; there are no external file references. Consumers must understand this OpenAEC raster attribute to display the image. It is an image resource, with no model placement or physical scale assigned.

IFCXB encodes the same JSON values as ordinary CBOR, prefixed with self-described-CBOR tag 55799 and compressed with gzip. This follows the generic IFCX CBOR/gzip transport used by [Open BIM Studio's reference writer](https://github.com/OpenAEC-Foundation/open-bim-studio/blob/main/spikes/dynlex-bim-slice/overlay/examples/bim-slice/bim_ifcx_binary.dl). It uses no string-reference or decimal tags. IFCXB encodings vary between applications; this transport is separate from IFCCAD's experimental typed-CBOR/Zstd format.

Use **File → Open IFCX** to reopen these IFCX/IFCXB raster files in Paint Studio. The reader requires one raster, checks PNG dimensions against metadata, and bounds compressed and decompressed input to 64 MB. Canvas/image limits also apply. Save/Ctrl+S writes IFCX with an embedded raster image. Editable scene operations remain available in the local draft; SVG exports vector geometry.
