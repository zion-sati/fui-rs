import type { BridgeRuntime } from '@effindomv2/runtime';

export const EXPECTED_EFFINDOM_CORE_ABI_VERSION = 2;
export const EXPECTED_EFFINDOM_UI_ABI_VERSION = 1;

function readAbiVersion(getter: (() => number) | undefined): number {
  return typeof getter === 'function' ? getter() : 0;
}

export function assertCompatibleAbi(runtime: BridgeRuntime): void {
  const actualCoreVersion = readAbiVersion(runtime.core._ed_get_abi_version?.bind(runtime.core));
  const actualUiVersion = readAbiVersion(runtime.ui._ui_get_abi_version?.bind(runtime.ui));

  if (
    actualCoreVersion === EXPECTED_EFFINDOM_CORE_ABI_VERSION &&
    actualUiVersion === EXPECTED_EFFINDOM_UI_ABI_VERSION
  ) {
    return;
  }

  throw new Error(
    `EffinDom ABI mismatch: fui-as expects core ABI ${String(EXPECTED_EFFINDOM_CORE_ABI_VERSION)} ` +
    `and ui ABI ${String(EXPECTED_EFFINDOM_UI_ABI_VERSION)}, but loaded core ABI ` +
    `${String(actualCoreVersion)} and ui ABI ${String(actualUiVersion)}. ` +
    'Rebuild/publish @effindomv2/runtime and @effindomv2/fui-as together.',
  );
}
