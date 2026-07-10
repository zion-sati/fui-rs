import type { UiModule } from '../../core-types';
import type { DebugTreeSnapshot } from '../../debug-tree';
import { parseDebugTreeBufferSafe } from '../../debug-tree';
import { extractDebugTreeBuffer } from '../utils/heap';

export class DebugTreeController {
  private snapshot: DebugTreeSnapshot = parseDebugTreeBufferSafe(new Uint32Array(), () => undefined);

  public constructor(private readonly ui: UiModule) {}

  public syncDebugTreeState(): void {
    this.snapshot = parseDebugTreeBufferSafe(
      extractDebugTreeBuffer(this.ui),
      (message) => { console.error(message); },
    );
  }

  public getDebugTree(): DebugTreeSnapshot {
    return this.snapshot;
  }

  public destroy(): void {
    this.snapshot = parseDebugTreeBufferSafe(new Uint32Array(), () => undefined);
  }
}
