# MCP server

The server uses the official MCP TypeScript SDK v2 and stdio transport. It calls the same authenticated local API as the editor. Start the API first. The MCP host launches the stdio process; do not run it as an HTTP endpoint.

Example host configuration (replace absolute paths):

```json
{
  "mcpServers": {
    "openaec-paint-studio": {
      "command": "/absolute/path/to/node",
      "args": ["/absolute/path/to/OpenAEC Paint/dist-services/services/mcp.js"],
      "env": {
        "PAINT_DATA_DIR": "/absolute/path/to/OpenAEC Paint/.paint-data",
        "PAINT_API_URL": "http://127.0.0.1:47831"
      }
    }
  }
}
```

`pnpm build` generates that entry point. The server reads the API token from the configured data directory; alternatively supply the same `PAINT_API_TOKEN` as the API. Tokens are never embedded in project files. MCP logs use stderr, leaving stdout for the protocol.

Tools:

- `paint_list_documents` — list saved API drawings.
- `paint_create_document` — create a blank image and receive its ID/revision.
- `paint_get_document` — read the image and revision.
- `paint_draw` — append validated drawing operations with `expectedRevision`.
- `paint_undo` — undo the last drawing operation with `expectedRevision`.
- `paint_export_png` — return the rendered PNG as an MCP image block.

Example request: “Create a 1200 × 800 image, draw an amber rectangle with a dark outline, and show me the PNG.” The host first creates a document, draws operations using the returned revision, then calls export. Load its ID in **Automation → Load from API** to continue manually. There is no background UI synchronization.
