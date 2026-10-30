# OpenAEC Paint changes to Rawler 0.8.0

Upstream: crates.io rawler 0.8.0 / https://github.com/dnglab/dnglab (LGPL-2.1).

2026-10-30: use web-time::Instant for monotonic decoder/development timing. std::time::Instant panics on wasm32-unknown-unknown, preventing browser RAW imports. web-time delegates to std on native targets and Performance.now in the browser. No image-development algorithm is changed.

The upstream data/testdata photograph corpus is omitted; it is not used to compile or run the library in Paint. Camera/lens metadata, library/build source, license and upstream test/benchmark source are preserved. Cargo.lock pins all dependencies. New browser DNG regression test uses an original synthetic sensor fixture, with no third-party photographs.
