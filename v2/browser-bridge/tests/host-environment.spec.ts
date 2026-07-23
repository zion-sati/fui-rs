import { expect, test } from '@playwright/test';

import { createHostImportModule, type HostImportDeps } from '../src/managed-harness/host-imports';
import {
  browserHostCapabilities,
  HostCapability,
  HostEnvironment,
} from '../src/managed-harness/host-environment';

test('browser FUI host imports report browser environment and operations', () => {
  const imports = createHostImportModule({} as HostImportDeps);
  expect(imports.fui_get_host_environment()).toBe(HostEnvironment.Browser);
  expect(imports.fui_get_host_capabilities()).toBe(browserHostCapabilities);
  expect(browserHostCapabilities & HostCapability.BrowserHistory).not.toBe(0);
  expect(browserHostCapabilities & HostCapability.Reload).not.toBe(0);
  expect(browserHostCapabilities & HostCapability.NewBrowsingContext).not.toBe(0);
  expect(browserHostCapabilities & HostCapability.OpenExternalUri).not.toBe(0);
  expect(browserHostCapabilities & HostCapability.ClipboardRead).not.toBe(0);
  expect(browserHostCapabilities & HostCapability.ClipboardWrite).not.toBe(0);
  expect(browserHostCapabilities & HostCapability.FileDialogs).not.toBe(0);

  let caption = '';
  const captionImports = createHostImportModule({
    platformHost: {
      setApplicationCaption(value: string): void {
        caption = value;
      },
    },
    readAppUtf8: () => 'EffinDOM • browser',
  } as unknown as HostImportDeps);
  captionImports.fui_set_application_caption(1, 21);
  expect(caption).toBe('EffinDOM • browser');
});
