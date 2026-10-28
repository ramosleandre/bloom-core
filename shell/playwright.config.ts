import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  testMatch: '**/*.spec.ts',
  fullyParallel: false,
  workers: 1,
  timeout: 30_000,
  retries: 0,
  use: {
    viewport: { width: 1440, height: 1000 },
    launchOptions: {
      executablePath: process.env.BLOOM_CHROMIUM_EXECUTABLE || undefined,
    },
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',
  },
});
