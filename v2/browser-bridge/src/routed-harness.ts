import type { BuildMode, DevToolsDomMirrorMode, PageZoomMode } from '@effindomv2/runtime';

import {
  pushManagedHistoryEntry,
  replaceManagedHistoryEntry,
  startManagedHarness,
  type HarnessAppOptions,
  type HarnessController,
  type HarnessExports,
  type HarnessState,
  type HostEventsDefinition,
  type HostServicesDefinition,
  type WorkerHostServicesBundleConfig,
} from './managed-harness';
import type { RoutedHarnessRouteSpec } from './routed-app-conventions';

type NavigationMode = 'push' | 'replace' | 'pop';

export type RoutedHarnessRoute = RoutedHarnessRouteSpec & {
  readonly matchPath?: string;
};

export interface RoutedHarnessManagerState {
  readonly shellId: string;
  readonly routePath: string;
  readonly activeWasmPath: string;
  readonly routeLoads: Readonly<Record<string, number>>;
}

export interface RoutedHarnessConfig<
  TExports extends HarnessExports,
  TRoute extends RoutedHarnessRoute = RoutedHarnessRoute,
> {
  readonly shellId: string;
  readonly routeBase: string;
  readonly routes: readonly TRoute[];
  readonly buildMode?: BuildMode;
  readonly devToolsDomMirror?: DevToolsDomMirrorMode;
  readonly pageZoom?: PageZoomMode;
  readonly hostEvents?: HostEventsDefinition;
  readonly hostServices?: HostServicesDefinition;
  readonly workerHostServices?: WorkerHostServicesBundleConfig;
  readonly recreateRuntimeOnWarmRouteSwap?: boolean;
  readonly showLoadingOverlay?: (isWarmRouteSwap: boolean, route: TRoute) => boolean | undefined;
  readonly onBooting?: () => void;
  readonly onRouteLoading?: (route: TRoute) => void;
  readonly onRouteReady?: (state: RoutedHarnessManagerState, route: TRoute) => void;
  readonly onHarnessStateUpdated?: (state: HarnessState) => void;
  readonly onHarnessError?: (error: unknown) => void;
  readonly run: (exports: TExports, route: TRoute) => void;
  readonly onDispose?: (exports: TExports, route: TRoute) => void;
}

function normalizeRoutePath(
  pathname: string,
  routeBase: string,
  routeByPath: ReadonlyMap<string, RoutedHarnessRoute>,
  fallbackRoutePath: string,
): string {
  let normalized = pathname;
  if (normalized.endsWith('/index.html')) {
    normalized = normalized.slice(0, -'index.html'.length);
  }
  if (!normalized.endsWith('/')) {
    normalized += '/';
  }
  if (normalized === routeBase) {
    return fallbackRoutePath;
  }
  return routeByPath.has(normalized) ? normalized : fallbackRoutePath;
}

function resolveRouteMatchPath(route: RoutedHarnessRoute): string {
  return route.matchPath ?? route.routePath;
}

function currentBrowserPath(): string {
  let pathname = window.location.pathname;
  if (pathname.endsWith('/index.html')) {
    pathname = pathname.slice(0, -'index.html'.length);
  }
  return pathname.endsWith('/') ? pathname : `${pathname}/`;
}

export function startRoutedHarness<
  TExports extends HarnessExports,
  TRoute extends RoutedHarnessRoute = RoutedHarnessRoute,
>(config: RoutedHarnessConfig<TExports, TRoute>): void {
  if (config.routes.length === 0) {
    throw new Error('startRoutedHarness requires at least one route.');
  }
  const routeByPath = new Map<string, TRoute>(config.routes.map((route) => [resolveRouteMatchPath(route), route]));
  const fallbackRoute = config.routes[0];
  if (fallbackRoute === undefined) {
    throw new Error('startRoutedHarness requires at least one route.');
  }
  const defaultRoute: TRoute = fallbackRoute;
  const routeLoads: Record<string, number> = {};
  let activeRoute: TRoute | null = null;
  let navigationQueue: Promise<void> = Promise.resolve();

  function resolveRoute(pathname: string): TRoute {
    const normalizedPath = normalizeRoutePath(
      pathname,
      config.routeBase,
      routeByPath,
      resolveRouteMatchPath(defaultRoute),
    );
    const resolved = routeByPath.get(normalizedPath);
    if (resolved !== undefined) {
      return resolved;
    }
    return defaultRoute;
  }

  function sameBrowserLocation(route: TRoute): boolean {
    return currentBrowserPath() === resolveRouteMatchPath(route);
  }

  function buildManagerState(route: TRoute): RoutedHarnessManagerState {
    return {
      shellId: config.shellId,
      routePath: route.routePath,
      activeWasmPath: route.wasmPath,
      routeLoads: { ...routeLoads },
    };
  }

  async function navigateToRoute(
    controller: HarnessController,
    targetUrl: URL,
    mode: NavigationMode,
  ): Promise<void> {
    const route = resolveRoute(targetUrl.pathname);
    if (mode !== 'pop' && sameBrowserLocation(route) && activeRoute?.routePath === route.routePath) {
      config.onRouteReady?.(buildManagerState(route), route);
      return;
    }

    const destination = new URL(route.routePath, window.location.origin);
    destination.search = targetUrl.search;
    destination.hash = targetUrl.hash;
    if (mode === 'push') {
      pushManagedHistoryEntry(destination);
    } else if (mode === 'replace' && !sameBrowserLocation(route)) {
      replaceManagedHistoryEntry(destination);
    }

    config.onRouteLoading?.(route);
    const isWarmRouteSwap = activeRoute !== null;
    if (isWarmRouteSwap && config.recreateRuntimeOnWarmRouteSwap === true) {
      await controller.recreateRuntime();
    }

    const persistedRestoreMode: 'initial' | 'pop' | 'none' = activeRoute === null ? 'initial' : (mode === 'pop' ? 'pop' : 'none');
    const appOptionsBase: HarnessAppOptions<TExports> = {
      wasmPath: route.wasmPath,
      persistedRestoreMode,
      run(exports: TExports): void {
        config.run(exports, route);
      },
      onDispose(exports: TExports): void {
        config.onDispose?.(exports, route);
      },
      onStateUpdated(state: HarnessState): void {
        config.onHarnessStateUpdated?.(state);
      },
    };
    const appOptions: HarnessAppOptions<TExports> = {
      ...appOptionsBase,
      ...(config.hostEvents === undefined ? {} : { hostEvents: config.hostEvents }),
      ...(config.hostServices === undefined ? {} : { hostServices: config.hostServices }),
      ...(config.workerHostServices === undefined ? {} : { workerHostServices: config.workerHostServices }),
    };
    const showLoadingOverlay = config.showLoadingOverlay?.(isWarmRouteSwap, route);
    appOptions.showLoadingOverlay = showLoadingOverlay ?? !isWarmRouteSwap;

    await controller.loadApp(appOptions);
    routeLoads[route.routePath] = (routeLoads[route.routePath] ?? 0) + 1;
    activeRoute = route;
    config.onRouteReady?.(buildManagerState(route), route);
  }

  startManagedHarness({
    ...(config.buildMode === undefined ? {} : { buildMode: config.buildMode }),
    ...(config.devToolsDomMirror === undefined ? {} : { devToolsDomMirror: config.devToolsDomMirror }),
    ...(config.pageZoom === undefined ? {} : { pageZoom: config.pageZoom }),
    onReady: async (controller): Promise<void> => {
      controller.setSameOriginNavigationHandler((target, mode) => {
        navigationQueue = navigationQueue.then(() => navigateToRoute(controller, target, mode));
        return navigationQueue;
      });

      config.onBooting?.();
      await navigateToRoute(controller, new URL(window.location.href), 'replace');
    },
    onError(error): void {
      config.onHarnessError?.(error);
    },
  });
}
