# Migrating to 2026.44.0

Paint Studio now uses a Rust editor, shared image engine, REST API and MCP server. Desktop installers no longer need a WebView or Node runtime. The browser preview is the same editor compiled to WebAssembly.

## Documents and drafts

- Existing IFCX/IFCXB raster exports still open through **Open IFCX**. They contain a PNG snapshot, rather than editable scene operations.
- The browser retains the existing `openaec-paint` IndexedDB draft and document schema. Existing operations can be restored in the Rust editor.
- Native Rust drafts use a new platform application-data location. Older Tauri application data is retained by upgrades; it is not automatically converted. Export important drawings before replacing an older installation, and keep the original data until you have checked the result.
- The API v1 document schema remains compatible, including affine transforms, embedded PNG operations and retained vectors. Existing API workspace documents and tokens can continue to use the same `PAINT_DATA_DIR`.

## API and MCP

Replace the previous `node .../api.js` or `node .../mcp.js` commands with the native `paint-api` and `paint-mcp` executables (`.exe` on Windows). Start the API separately and point the MCP host to the native executable. Routes, optimistic revisions and the six MCP tool names are retained. See [API](docs/API.md), [MCP](docs/MCP.md) and [native service distribution](docs/SERVICE-DISTRIBUTION.md).

## Interface and installation

The Home ribbon automatically expands, compacts and collapses individual groups according to available width. All commands remain in group menus without horizontal ribbon scrolling. Escape returns to selection, Ctrl+A selects objects, Ctrl+wheel zooms and middle-button dragging pans. Text fields keep their normal editing shortcuts.

Windows and Linux installers register Open With support for image formats and IFCX; macOS declares image editor document types. Existing default viewers are preserved. Themes now come from OpenAEC Blueprint 2026.44.1. See [format limits](docs/IMAGE-FORMATS.md) for supported profiles and camera variants.

New source is MIT, with the Blueprint LGPL-3.0 and Rawler LGPL-2.1 components retained under their own licenses. The release includes complete corresponding source, locked dependencies, notices and offline build configuration. Detached Ed25519 signatures are separate from Windows Authenticode and Apple Developer ID/notarization. See [releasing](docs/RELEASING.md) and [validation](docs/VALIDATION.md).
