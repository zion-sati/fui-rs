import type { FullConfig } from '@playwright/test';
import type { startStaticServer } from '../../ui/tests/integration/helpers/static_server';

declare global {
  var __serverHandle: Awaited<ReturnType<typeof startStaticServer>> | null;
}

export default async function globalTeardown(_config: FullConfig) {
  if (globalThis.__serverHandle) {
    await globalThis.__serverHandle.close();
  }
}
