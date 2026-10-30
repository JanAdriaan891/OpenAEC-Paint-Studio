# MCP server

The Rust `paint-mcp` executable implements newline-delimited JSON-RPC over stdio. It calls the same authenticated localhost API as the editor. Start `paint-api` first; the MCP host launches `paint-mcp`. No Node or SDK installation is required.

```json
{
  "mcpServers": {
    "openaec-paint-studio": {
      "command": "/absolute/path/to/paint-mcp",
      "env": {
        "PAINT_DATA_DIR": "/absolute/path/to/paint-workspace",
        "PAINT_API_URL": "http://127.0.0.1:47831"
      }
    }
  }
}
```

On Windows use the installed `paint-mcp.exe` path. API and MCP must use the same data directory/token. Alternatively set `PAINT_API_TOKEN` in both process environments. The token is never written to stdout; stdout is exclusively protocol messages. Supported negotiation versions: 2024-11-05, 2025-03-26, 2025-06-18 and 2025-11-25.

| Tool | Action |
| --- | --- |
| `paint_list_documents` | List saved API documents |
| `paint_create_document` | Create an image and receive ID/revision |
| `paint_get_document` | Read a document and revision |
| `paint_draw` | Append operations with `expectedRevision` |
| `paint_undo` | Remove the last operation with `expectedRevision` |
| `paint_export_png` | Return the rendered PNG as an MCP image block |

The tool names and document schema remain compatible with 2026.43.1. Ask a host to create an image, draw operations using the returned revision, then export the PNG. Retrieve its document ID in the editor's API dialog to continue editing. Transfers are explicit; there is no background synchronization.
