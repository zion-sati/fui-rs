import type { BridgeRuntime } from '../../core-types';
import { handleToBigInt } from '../utils/encoding';

function pointInSemanticBounds(
  bounds: { readonly x: number; readonly y: number; readonly width: number; readonly height: number },
  x: number,
  y: number,
): boolean {
  return x >= bounds.x &&
    x <= (bounds.x + bounds.width) &&
    y >= bounds.y &&
    y <= (bounds.y + bounds.height);
}

export function findEditorTextHandleAtPoint(runtime: BridgeRuntime, x: number, y: number): bigint {
  const tree = runtime.getDebugTree().nodes;
  for (let i = tree.length - 1; i >= 0; i -= 1) {
    const node = tree[i];
    if (node?.behavior.textEditor !== true) {
      continue;
    }
    if (pointInSemanticBounds(node.visibleBounds, x, y)) {
      return handleToBigInt(node.handle);
    }
  }
  return 0n;
}
