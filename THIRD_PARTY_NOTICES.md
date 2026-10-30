# Third-party notices

New Paint application, shared image/document engine, API, MCP and release tooling are MIT licensed. The repository MIT license does not relicense the components below.

## OpenAEC Blueprint

`vendor/openaec-ui` is copied unchanged from [OpenAEC-Foundation/OpenAEC-Blueprint](https://github.com/OpenAEC-Foundation/OpenAEC-Blueprint), revision **7a2f7cdc095e3854767af07e4e4cf07192498b8f**, package `openaec-ui 2026.44.1`, shared library 0.4. The library is LGPL-3.0. Its full license and asset/font notices remain in `vendor/openaec-ui/LICENSE.md` and `vendor/openaec-ui/licenses/`. Paint uses its widgets, colors, themes, fonts and layout helpers. Paint's small tool icons in `crates/paint-app/src/icons.rs` are original MIT source; the B2icons collection is not used.

## OpenAEC branding

The symbol in `public/brand` and application icons in `packaging/icons` derive from [OpenAEC-Foundation/OpenAEC-style-book](https://github.com/OpenAEC-Foundation/OpenAEC-style-book), revision **dfdcd415b26604081b1699560621d3e9ae7ea6bf**. Copyright OpenAEC Foundation and contributors, Creative Commons Attribution-ShareAlike 4.0. The full license is `public/brand/LICENSE`. Changes resize and package the symbol as application icons. Original React/CSS ribbon code is no longer used in the Rust application.

## Fonts

Inter and JetBrains Mono are provided by Blueprint under SIL Open Font License 1.1. Their licenses are retained under `vendor/openaec-ui/licenses/`. DejaVu Serif regular/bold are distributed under the Bitstream Vera and DejaVu terms in `assets/fonts/DejaVu-LICENSE.txt`. Font data is embedded for offline UI and image text rendering.

## Image and runtime dependencies

Cargo.lock records exact versions. This includes iced, tiny-skia, resvg, fontdue, image, heic-rs, psd, jxl-oxide, hayro-jpeg2000, Rawler, rav1d/avif-decode, axum, tokio, reqwest and Ed25519-dalek. **Rawler 0.8.0 is LGPL-2.1**. `vendor/avif-decode` retains its BSD-3-Clause license and documents the bounded-decoder/build changes in `PAINT-CHANGES.md`. The application is statically linked; it imposes no signature or installation lock on modified binaries.

`paint-release legal` collects license/notice files and exact package license identifiers into each installer/service package. The release's **source.tar.gz** contains application source, all locked Cargo dependencies and offline Cargo configuration. It is the complete corresponding source for the linked LGPL components and the application code needed to rebuild/relink them; see `docs/RELEASING.md`.

Historical demonstration media in docs/demo shows the previous 2026.43.1 release. The current editor screenshots are generated from the Rust UI. Playwright is used only for development browser checks; its notices remain in its package, and it is not distributed as an application dependency.
