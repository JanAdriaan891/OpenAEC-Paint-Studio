# Paint API v1

Local, authenticated HTTP API. Start with `pnpm api` or `node dist-services/services/api.js`.

Defaults: `127.0.0.1:47831`, workspace `.paint-data/` relative to the repository, with documents persisted atomically as JSON. Override `PAINT_DATA_DIR`, `PAINT_API_PORT` or `PAINT_API_TOKEN` (at least 32 characters). The editor currently connects to the default port; MCP can use `PAINT_API_URL`. Do not expose the service through a public proxy.

Every request, including health, needs `Authorization: Bearer <token>`. JSON request bodies require `Content-Type: application/json`. CORS accepts only the development/preview and Tauri app origins. Requests with other origins or non-loopback Host headers are rejected.

| Method | Route | Body / response |
|---|---|---|
| GET | `/health` | Version and status |
| GET | `/v1/documents` | Document summaries |
| POST | `/v1/documents` | Full document → stored document (201) |
| GET | `/v1/documents/:id` | `{id, revision, updatedAt, document}` |
| PUT | `/v1/documents/:id` | `{document, expectedRevision}` |
| POST | `/v1/documents/:id/operations` | `{operations, expectedRevision}` |
| POST | `/v1/documents/:id/undo` | `{expectedRevision}` |
| GET | `/v1/documents/:id/export.png` | Rendered PNG |

Document example:

```json
{
  "format": "openaec-paint",
  "version": 1,
  "name": "Sketch",
  "width": 1200,
  "height": 800,
  "background": "#ffffff",
  "retainVectors": true,
  "operations": []
}
```

Append a shape with the current revision:

```json
{
  "expectedRevision": 1,
  "operations": [{
    "type": "rectangle",
    "from": {"x": 100, "y": 100},
    "to": {"x": 500, "y": 350},
    "color": "#d97706",
    "size": 4,
    "opacity": 1,
    "filled": false
  }]
}
```

Operations are defined and validated in `shared/document.ts`: brush/eraser use `points`; line/rectangle/ellipse use `from`/`to`; text uses `at`/`text`; fill uses `at`/`color`; image uses embedded PNG `data` and optional `sourceSize` (original dimensions). All operations accept an optional affine `transform` array `[a,b,c,d,e,f]`, matching Canvas 2D matrices. Text additionally supports `fontFamily` (`sans-serif`, `serif`, `monospace`) and `bold`. Image operations use `sourceSize`, falling back to document dimensions for older files. Shapes and text support `size` (1–200) and `opacity` (0–1). Colors use six-digit hex. Erasing clears ink to reveal the document background. Fill uses exact RGBA matching, not a tolerance slider.

Limits: each side 1–4096 px, total 8 megapixels; 2,000 operations/document; 20,000 points/stroke; 200 operations/request; 64 MB request bodies; 48 million characters/embedded PNG; 100 API documents/workspace. UI undo/redo is separate from the API's single-operation undo. API mutations are serialized synchronously and use optimistic revisions (409 conflict). Do not run multiple API processes against the same data directory.

Errors: 400 validation/JSON, 401 authentication, 403 origin/Host, 404 missing document/route, 409 conflict/capacity, 413 oversized request, 415 wrong media type, 500 rendering/storage failure. Back up the `.paint-data/documents` directory to retain API drawings.
