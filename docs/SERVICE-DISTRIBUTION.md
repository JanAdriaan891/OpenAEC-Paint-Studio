# Native services

Release assets ending in `_services.tar.gz` contain `paint-api`, `paint-mcp`, their API/MCP documentation and dependency notices. Linux x64, Windows x64 (`.exe`) and universal macOS binaries need no Node runtime. Desktop installers include the same executables.

Extract the archive and start `paint-api` from your chosen workspace directory. It creates `.paint-data/token` and `.paint-data/documents/`. Set `PAINT_DATA_DIR` to an absolute directory when launching the API/MCP from another working directory. Give your MCP host the absolute path to `paint-mcp` and the same data directory. Nothing starts automatically on login.

Use the signed manifest and trusted repository public key to verify the archive before extracting; see RELEASING.md. Back up the data directory to preserve documents.
