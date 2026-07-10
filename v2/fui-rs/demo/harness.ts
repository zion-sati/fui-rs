import type { HarnessExports, HarnessState } from '../../browser-bridge/src/managed-harness.js';
import {
  startRoutedHarness,
  type RoutedHarnessManagerState,
  type RoutedHarnessRoute,
} from '../../browser-bridge/src/routed-harness.js';
import { demoHostEvents } from './src/host-events.js';
import { demoHostServices } from './src/host-services.js';
import {
  setDemoShellAccentColor,
  setDemoShellDarkMode,
  setDemoShellTick,
} from './src/host-service-state.js';
import { readHostAccentColor } from '../../browser-bridge/src/managed-harness/ui-chrome.js';

declare global {
  interface Window {
    __fuiReady?: boolean;
    __fuiError?: string;
    __fuiState?: HarnessState;
    __fuiManagerState?: {
      readonly routePath: string;
      readonly activeWasmPath: string;
      readonly routeLoads: Readonly<Record<string, number>>;
    };
    __getFuiHostTick?(): number;
    __getFuiHostDarkMode?(): boolean;
    __startFuiWorker?(): void;
    __startFuiFailingWorker?(): void;
    __getFuiWorkerStatusCode?(): number;
    __getFuiWorkerDetailHasPrimeAndClock?(): boolean;
    __getFuiWorkerDetailHasErrorClock?(): boolean;
  }
}

interface DemoExports extends HarnessExports {
  __runApp(): void;
  __disposeApp?(): void;
  __getDemoHostTick?(): number;
  __getDemoHostDarkMode?(): boolean;
  __startWorkerDemo?(): void;
  __startFailingWorkerDemo?(): void;
  __getWorkerDemoStatusCode?(): number;
  __workerDemoDetailHasPrimeAndClock?(): boolean;
  __workerDemoDetailHasErrorClock?(): boolean;
}

interface DemoRoute extends RoutedHarnessRoute {
  readonly key: 'home' | 'workbench' | 'stage4' | 'stage5';
}

interface DemoRouteManifest {
  readonly routes: readonly DemoRouteManifestEntry[];
}

interface DemoRouteManifestEntry {
  readonly key: 'home' | 'workbench' | 'stage4' | 'stage5';
  readonly routePath: string;
  readonly matchPath?: string;
  readonly wasmPath: string;
  readonly title: string;
}

const wasmVersion = Date.now().toString(36);

function withWasmVersion(path: string): string {
  return `${path}?v=${wasmVersion}`;
}

let currentExports: DemoExports | null = null;
let tick = 0;
const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
let darkMode = darkModeQuery.matches;
let demoShellTickTimer: number | null = null;

function syncDemoShellState(): void {
  setDemoShellTick(tick);
  setDemoShellAccentColor(readHostAccentColor());
  setDemoShellDarkMode(darkMode);
}

function stopDemoShellTick(): void {
  if (demoShellTickTimer === null) {
    return;
  }
  clearTimeout(demoShellTickTimer);
  demoShellTickTimer = null;
}

function scheduleDemoShellTick(): void {
  if (demoShellTickTimer !== null || document.hidden) {
    return;
  }
  demoShellTickTimer = window.setTimeout(() => {
    demoShellTickTimer = null;
    if (document.hidden) {
      return;
    }
    tick += 1;
    syncDemoShellState();
    scheduleDemoShellTick();
  }, 1000);
}

function syncDemoShellTickVisibility(): void {
  if (document.hidden) {
    stopDemoShellTick();
    return;
  }
  syncDemoShellState();
  scheduleDemoShellTick();
}

const workerHostServices = {
  scriptUrl: new URL('./worker-host-services.js', import.meta.url).toString(),
  exportName: 'demoWorkerHostServices',
};

function isManifestEntry(value: unknown): value is DemoRouteManifestEntry {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const record = value as Partial<Record<keyof DemoRouteManifestEntry, unknown>>;
  return (
    (record.key === 'home' ||
      record.key === 'workbench' ||
      record.key === 'stage4' ||
      record.key === 'stage5') &&
    typeof record.routePath === 'string' &&
    (record.matchPath === undefined || typeof record.matchPath === 'string') &&
    typeof record.wasmPath === 'string' &&
    typeof record.title === 'string'
  );
}

function readRouteManifest(raw: unknown): readonly DemoRoute[] {
  if (typeof raw !== 'object' || raw === null || !('routes' in raw)) {
    throw new Error('FUI-RS demo route manifest must contain a routes array.');
  }
  const routes = (raw as DemoRouteManifest).routes;
  if (!Array.isArray(routes) || routes.length === 0) {
    throw new Error('FUI-RS demo route manifest must contain at least one route.');
  }
  return routes.map((route) => {
    if (!isManifestEntry(route)) {
      throw new Error('FUI-RS demo route manifest contains an invalid route entry.');
    }
    return {
      key: route.key,
      routePath: route.routePath,
      ...(route.matchPath === undefined ? {} : { matchPath: route.matchPath }),
      wasmPath: withWasmVersion(route.wasmPath),
      title: route.title,
    };
  });
}

async function loadRoutes(): Promise<readonly DemoRoute[]> {
  const response = await fetch('./routes.json', { credentials: 'same-origin' });
  if (!response.ok) {
    throw new Error(`Failed to load FUI-RS demo route manifest: ${String(response.status)} ${response.statusText}`);
  }
  return readRouteManifest(await response.json());
}

function publishManagerState(state: RoutedHarnessManagerState, route: DemoRoute): void {
  window.__fuiManagerState = {
    routePath: route.routePath,
    activeWasmPath: route.wasmPath,
    routeLoads: { ...state.routeLoads },
  };
}

function installDemoHooks(): void {
  window.__getFuiHostTick = () => {
    return currentExports?.__getDemoHostTick?.() ?? -1;
  };
  window.__getFuiHostDarkMode = () => {
    return Boolean(currentExports?.__getDemoHostDarkMode?.());
  };
  window.__startFuiWorker = () => {
    currentExports?.__startWorkerDemo?.();
  };
  window.__startFuiFailingWorker = () => {
    currentExports?.__startFailingWorkerDemo?.();
  };
  window.__getFuiWorkerStatusCode = () => {
    return currentExports?.__getWorkerDemoStatusCode?.() ?? 0;
  };
  window.__getFuiWorkerDetailHasPrimeAndClock = () => {
    return Boolean(currentExports?.__workerDemoDetailHasPrimeAndClock?.());
  };
  window.__getFuiWorkerDetailHasErrorClock = () => {
    return Boolean(currentExports?.__workerDemoDetailHasErrorClock?.());
  };
}

window.__fuiReady = false;
delete window.__fuiError;
installDemoHooks();
syncDemoShellState();
darkModeQuery.addEventListener('change', (event) => {
  darkMode = event.matches;
  syncDemoShellState();
});
document.addEventListener('visibilitychange', syncDemoShellTickVisibility);
scheduleDemoShellTick();

void loadRoutes()
  .then((routes) => {
    startRoutedHarness<DemoExports, DemoRoute>({
      shellId: 'fui-rs-demo-shell',
      routeBase: '/v2/fui-rs/demo/',
      routes,
      hostEvents: demoHostEvents,
      hostServices: demoHostServices,
      workerHostServices,
      devToolsDomMirror: 'on-requested',
      recreateRuntimeOnWarmRouteSwap: true,
      onRouteReady(state, route): void {
        publishManagerState(state, route);
        window.__fuiReady = true;
      },
      onHarnessStateUpdated(state): void {
        window.__fuiState = state;
      },
      onHarnessError(error): void {
        window.__fuiError = error instanceof Error ? error.message : String(error);
      },
      run(exports): void {
        currentExports = exports;
        exports.__runApp();
      },
      onDispose(exports): void {
        if (currentExports === exports) {
          currentExports = null;
        }
        exports.__disposeApp?.();
      },
    });
  })
  .catch((error: unknown) => {
    window.__fuiError = error instanceof Error ? error.message : String(error);
  });
