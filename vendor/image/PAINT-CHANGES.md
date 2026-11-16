# Paint changes to image 0.25.10

Base: the unmodified crates.io `image 0.25.10` source, MIT OR Apache-2.0.
License files and the published manifest/source are retained. The upstream repository is
https://github.com/image-rs/image. Cargo's root patch applies to every workspace consumer,
including image readers used by SVG import, native desktop and browser Wasm.

16 November 2026:

- WebP `ImageDecoder::set_limits` checks dimension/support limits and forwards `max_alloc`
  to `image-webp::WebPDecoder::set_memory_limit` before metadata is read. Backend
  `MemoryLimitExceeded` maps to `ImageError::Limits`. Adapted to the released API from
  the proposal https://github.com/image-rs/image/pull/3135; no upstream-main API upgrade.
- BMP readers with a file header reject pixel offsets inside the file/DIB headers,
  external Info bitmasks or the declared palette. The choice is strict rejection,
  rather than guessing where malformed files intended their pixels to start.
  Files with shortened/overstated palettes are rejected as well. Embedded V2–V5
  bitmasks are not counted twice. DIB/ICO paths without a file header retain their
  existing metadata-derived pixel offset. Tracks
  https://github.com/image-rs/image/issues/3131 and
  https://github.com/image-rs/image/pull/3142.

Owned regression tests are in `crates/paint-core/tests/import_limits.rs`, UI failure
state tests in `crates/paint-app/src/tests.rs`, and browser acceptance in
`tests/web-workflows.mjs`. This does not claim a bound on every internal decoder
allocation: lossless WebP Huffman allocations in image-webp 0.2.4 remain a separate
upstream investigation (#197/#211/#213).
