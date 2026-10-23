import { defineConfig } from '@playwright/test';
const production=process.env.PAINT_TEST_PRODUCTION==='1';
const port=production?49322:49321;
export default defineConfig({
  testDir: './tests/ui', use: { baseURL: `http://127.0.0.1:${port}`, viewport: { width: 1440, height: 1000 } },
  webServer: { command: `pnpm exec vite ${production?'preview':''} --host 127.0.0.1 --port ${port} --strictPort`, url: `http://127.0.0.1:${port}`, reuseExistingServer: false },
});
