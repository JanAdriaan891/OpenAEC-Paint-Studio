# Decoder fixtures

These tiny 32×24 solid-blue images were generated for Paint Studio's regression tests. They contain no user photos or third-party artwork.

TIFF, PSD, AVIF, JPEG XL and JPEG 2000 were independently encoded with the previous application's ImageMagick WASM 0.0.44. The Rust tests decode them and check dimensions and pixel colors. This verifies interoperability rather than only asking a decoder to read its own output.

HEIC was encoded from the same solid-blue PNG with libheif/heif-enc 1.17.6 and x265, in lossless mode.
