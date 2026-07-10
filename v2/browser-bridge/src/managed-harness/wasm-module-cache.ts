import { getHostServiceImportNames, type HostServicesDefinition } from './host-services';

export class WasmModuleCache {
  private readonly wasmByteCache = new Map<string, Promise<ArrayBuffer>>();
  private readonly wasmModuleCache = new Map<string, Promise<WebAssembly.Module>>();

  async fetchWasmBytes(wasmPath: string): Promise<ArrayBuffer> {
    const cached = this.wasmByteCache.get(wasmPath);
    if (cached !== undefined) {
      return cached;
    }
    const fetchPromise = fetch(wasmPath, { cache: 'no-store' }).then(async (response) => {
      if (!response.ok) {
        throw new Error(`Failed to load wasm app: ${wasmPath}`);
      }
      return response.arrayBuffer();
    });
    this.wasmByteCache.set(wasmPath, fetchPromise);
    return fetchPromise;
  }

  async loadWasmModule(wasmPath: string): Promise<WebAssembly.Module> {
    const cached = this.wasmModuleCache.get(wasmPath);
    if (cached !== undefined) {
      return cached;
    }
    const compilePromise = this.fetchWasmBytes(wasmPath).then((bytes) => WebAssembly.compile(bytes));
    this.wasmModuleCache.set(wasmPath, compilePromise);
    return compilePromise;
  }
}

export function validateAppImports(wasmModule: WebAssembly.Module, hostServices: HostServicesDefinition | undefined): void {
  const allowedHostServiceImports = getHostServiceImportNames(hostServices);
  for (const imported of WebAssembly.Module.imports(wasmModule)) {
    if (imported.kind !== 'function') {
      throw new Error(`App import ${imported.module}.${imported.name} is not allowed.`);
    }
    if (imported.module === 'effindom_v2_ui' || imported.module === 'fui_host' || imported.module === 'env') {
      continue;
    }
    if (imported.module === 'fui_host_service' && allowedHostServiceImports.has(imported.name)) {
      continue;
    }
    throw new Error(`App import ${imported.module}.${imported.name} is not allowed.`);
  }
}
