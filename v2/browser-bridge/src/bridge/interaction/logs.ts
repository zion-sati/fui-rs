import type {
  BridgeLogs,
  FocusEventLog,
  PointerEventLog,
  ScrollEventLog,
  SelectionChangeLog,
  TextChangeLog,
} from '../../core-types';

export function createBridgeLogs(): BridgeLogs {
  return {
    pointerEvents: [] as PointerEventLog[],
    focusEvents: [] as FocusEventLog[],
    textChanges: [] as TextChangeLog[],
    selectionChanges: [] as SelectionChangeLog[],
    crossSelectionChanges: [] as { areaHandle: string; text: string }[],
    clipboardWrites: [] as string[],
    clipboardReadRequests: [] as string[],
    scrollEvents: [] as ScrollEventLog[],
    missingFontCoverageRequests: [] as { fontId: number; coverageKind: number; sampleText: string }[],
    incrementalFontPackageRequests: [] as { primaryFontId: number; coverageKind: number; packageId: string; segmentIds: readonly string[]; sampleText: string }[],
  } satisfies BridgeLogs;
}
