import * as path from 'node:path';

import type { FullConfig } from '@playwright/test';

import { startStaticServer } from '../../ui/tests/integration/helpers/static_server';

const PUBLIC_DIR = path.join(__dirname, '..', '..', '..', 'public');

declare global {
  var __serverHandle: Awaited<ReturnType<typeof startStaticServer>> | null;
  var __serverPort: number | null;
}

export default async function globalSetup(_config: FullConfig) {
  globalThis.__serverHandle = await startStaticServer(PUBLIC_DIR, 11_150, 12_000);
  globalThis.__serverPort = globalThis.__serverHandle.port;
  process.env.BRIDGE_TEST_SERVER_PORT = String(globalThis.__serverPort);
}
