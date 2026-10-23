# Paint API and MCP service build

This archive contains compiled JavaScript. Requires Node.js 22+ and pnpm 11.25.0.

```sh
pnpm install --prod --frozen-lockfile
node dist-services/services/api.js
```

Keep the API running. Start your MCP host with `node dist-services/services/mcp.js` as its stdio command (use absolute paths in the host configuration). See `docs/MCP.md` and `docs/API.md`.

Source development scripts such as `pnpm dev`, `pnpm api` and `pnpm test` require the full source checkout. In this compiled archive, use the Node commands above.

This package does not include a Node runtime or native desktop installer. `pnpm install` fetches the platform's native renderer dependency. Do not copy `node_modules` between operating systems.
