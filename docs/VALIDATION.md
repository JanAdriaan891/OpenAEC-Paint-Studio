# Validation — 2026.44.0 Rust migration

The Rust implementation replaces the React/Tauri editor, TypeScript image logic and Node API/MCP. Owned crates are paint-core, paint-services, openaec-paint-studio and paint-tools. UI provenance is pinned in vendor/BLUEPRINT-REVISION.

## Automated checks

`cargo test --workspace --locked`: **28 tests passed** in the migration workspace:

- 15 image/document integration tests: existing JSON compatibility, PNG alpha and image opacity, brush dots, fill/eraser, selection/history, vector geometry/SVG, crop/fill interaction, corrections, PNG/JPEG/WebP imports/exports, IFCX/IFCXB pixel/alpha roundtrips and invalid metadata/dimension rejection. Independent fixtures cover TIFF, PSD, AVIF, HEIC, JPEG XL and JPEG 2000; a generated Bayer DNG exercises real RAW development.
- 2 adaptive ribbon policy tests.
- 6 native UI tests: real tiny-skia screen rendering in three themes and multiple widths, measured control bounds from 560–1920 px, collapsed group command access, Escape behavior, zoom/pan without document edits and retaining invalid text drafts for correction.
- 4 API/MCP/store tests: actual local HTTP authentication/origin/Host checks, document revisions, MCP-driven drawing and PNG export, persistence across restarts, remote URL rejection and a spawned MCP stdio process with protocol-only stdout.
- 1 Ed25519 signing test rejects changed artifact bytes and a wrong trusted public key. The Rust verifier also successfully verifies a previous Node-signed installer against the existing repository key.

Formatting is checked for all owned Rust crates. Vendored Blueprint sources are unchanged. Current README screenshots come from the real Rust renderer, rather than an HTML mockup.

## Distribution checks

The build workflow compiles Linux x64, Windows x64, universal Intel/Apple Silicon macOS and browser Wasm. Native process smoke checks render a PNG through each platform's built API. Linux also launches the desktop executable under Xvfb. Windows checks a silent NSIS installation and its Open With registration. macOS checks both architectures, ad-hoc codesigning and DMG integrity. Complete corresponding source includes locked Cargo dependencies and offline configuration.

The Playwright smoke test exercises real Rust/Wasm drawing, undo/redo, Escape, Ctrl+A without DOM text selection, Ctrl+wheel zoom, middle-button pan, editable text, IndexedDB draft recovery, narrow ribbon layout and image-only open filtering, and actual TIFF/PSD/HEIC/AVIF/JXL/JP2/DNG imports. Browser tests run in isolated profiles and do not change the user's existing draft.

A source archive and successful compilation cannot verify every operating-system interaction. Finder's native openFiles bridge and graphical behavior on physical Windows/macOS machines still benefit from normal platform acceptance testing. No Authenticode or Apple Developer ID/notarization certificate is configured; detached release signatures are independent Ed25519 signatures.

## Compatibility and limits

The API v1 document schema and six MCP tool names remain compatible with 2026.43.1. IFCX/IFCXB save a PNG raster, not editable scene operations. Browser IndexedDB retains the previous draft schema. Native Rust drafts use their own platform data location; old Tauri application data is retained by upgrades.

Imports have bounded size/dimensions and depend on the decoder's supported profiles/camera models. First frame/page only; PSD layers are flattened; no dedicated color-management or RAW adjustment workflow. See IMAGE-FORMATS.md and IFCX.md.

## 2026.45.0 workflow changes — 2 November 2026

The core document and API schema stay at v1. Save now writes a portable editable `*.paint.json` envelope; IFCX/IFCXB remain raster exchange. The new tests cover project roundtrips and invalid versions; original/result import dimensions; cancellation and dirty-state integrity; atomic native file replacement; browser download cancellation; screenshot paste, expansion and undo; transparent PNG clipboard; rectangular pixel editing and cached previews; transformed subset alignment in six directions; and late/recovered draft handling.

The workspace suite has 41 tests: 14 UI/state, 2 ribbon, 15 original core, 5 new core workflows, 4 services and 1 signing. Browser checks exercise the actual Rust/Wasm editor in isolated profiles. The new workflow suite also checks keyboard access to pixel mode, moves pixels, restores vectors with Undo, reopens a downloaded project and checks draft recovery at 560 px. Browser offline cold start and warm reload both fail without a server; no PWA offline support is claimed.

The release-mode benchmark executes five isolated samples per input size and publishes its [raw measurements and report](benchmarks/2026-11-02.md). It measures our core, not human usability or competitor speed. Physical Windows/macOS and cross-application native clipboard acceptance remain separate from CI compilation/packaging.
