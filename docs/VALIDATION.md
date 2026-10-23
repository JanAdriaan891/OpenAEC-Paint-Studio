# Validation — 23 October 2026

- TypeScript/frontend and compiled Node service builds pass.
- Nine integration/unit tests pass: rendering, flood fill, erasing/alpha, schema limits, API authentication/CORS/Host validation/persistence/revision conflicts, image formats, MCP stdio tools, release signing/tamper detection, vector transforms/SVG and color correction.
- Seven browser integration tests pass: drawing/undo/export/rotation/draft restore, settings, import/resize/crop, Home controls/inline text/object move/SVG/focus, TIFF decoding in a worker, SVG opening, and a crop regression with a fill seed outside the crop region.
- Format round trips cover PNG, JPEG, WebP, BMP, TIFF, GIF, PSD, ICO, TGA, AVIF and JPEG 2000. A public libheif HEIC example additionally decodes successfully (1280 × 854 px). See IMAGE-FORMATS.md for limits and distinctions from advertised reader capabilities.
- The browser preview has been visually checked with the light and Forge themes. Desktop UI interaction and Windows/macOS installer builds have not been exercised locally; their build/signing setup is in the CI workflow.
- All seven browser tests also pass against the production build, including the bundled WebAssembly worker. Tests use dedicated ports and refuse to reuse unrelated servers.
- The emitted JavaScript API and MCP entrypoints were started as standalone Node processes; a document was created through the compiled MCP server against the compiled API.

- Linux release bundles built successfully: DEB (8.18 MiB) and AppImage (85.59 MiB). Both contain the production frontend and have detached Ed25519 signatures verified against `release-public.pem`. Web and compiled-service archives are signed and verified as well. These signatures are separate from OS publisher certificates.
