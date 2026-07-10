import * as path from 'node:path';

import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: path.join(__dirname, 'tests'),
  testMatch: '**/*.spec.ts',
  testIgnore: ['old/**'],
  timeout: 60_000,
  retries: 1,
  globalSetup: path.join(__dirname, 'tests', 'global-setup.ts'),
  globalTeardown: path.join(__dirname, 'tests', 'global-teardown.ts'),
  use: {
    headless: true,
    permissions: ['clipboard-read', 'clipboard-write'],  
  },
  projects: [
    {
      name: 'chromium',
      use: {
        browserName: 'chromium',
      },
    },
  ],
  reporter: [
    ['list'],
    ['html', { outputFolder: path.join(__dirname, 'tests', 'report'), open: 'never' }],
  ],
});
