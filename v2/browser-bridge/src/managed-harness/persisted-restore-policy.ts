export type BrowserNavigationType = 'navigate' | 'reload' | 'back_forward' | 'prerender' | 'unknown';

export function readBrowserNavigationType(
  performanceLike: Performance = globalThis.performance,
): BrowserNavigationType {
  const navigationEntry = performanceLike.getEntriesByType('navigation')[0] as PerformanceNavigationTiming | undefined;
  if (navigationEntry !== undefined) {
    switch (navigationEntry.type) {
      case 'navigate':
      case 'reload':
      case 'back_forward':
      case 'prerender':
        return navigationEntry.type;
      default:
        return 'unknown';
    }
  }

  return 'unknown';
}

export function shouldRestoreInitialHistorySnapshot(
  navigationType: BrowserNavigationType,
  hasHistorySnapshotId: boolean,
): boolean {
  if (navigationType === 'back_forward') {
    return true;
  }
  if (navigationType === 'navigate') {
    return hasHistorySnapshotId;
  }
  return false;
}
