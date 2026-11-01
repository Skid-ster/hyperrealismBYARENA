import { defineConfig, devices } from '@playwright/test';
import path from 'node:path';
const port = Number(process.env.E2E_WEB_PORT || 5173);
const executablePath = process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH;
export default defineConfig({
  testDir: './tests-web/e2e',
  fullyParallel: false,
  workers: 1,
  timeout: 60000,
  expect: { timeout: 10000 },
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    launchOptions: { executablePath, args: ['--no-sandbox'] },
    viewport: { width: 1440, height: 1000 },
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'], viewport: { width: 1440, height: 1000 } } },
  ],
  webServer: {
    command: 'npm run dev:web',
    url: `http://127.0.0.1:${port}`,
    reuseExistingServer: false,
    timeout: 60000,
    env: {
      E2E_WEB_PORT: String(port),
      API_PORT: '3101',
      HYPERREALISM_DATA_DIR: path.resolve('.workspace/e2e'),
      HYPERREALISM_BIN:
        process.env.HYPERREALISM_BIN ||
        path.resolve(`target/release/hyperrealism-core${process.platform === 'win32' ? '.exe' : ''}`),
    },
  },
});
