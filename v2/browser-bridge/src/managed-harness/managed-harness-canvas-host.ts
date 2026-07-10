import type { BridgeRuntime,WasmHandleLike } from '@effindomv2/runtime';
import type { CoreModule } from '@effindomv2/runtime/core-types';
import { copyBytesFromHeap, normalizePointerForWasm, pointerToHeapOffset, withHeapAllocation, withHeapBytes } from '@effindomv2/runtime';

interface ManagedHarnessCanvasHostDependencies {
  getRuntime(): BridgeRuntime;
  readAppBytes(ptr: number, len: number): Uint8Array;
  writeAppBytes(ptr: number, capacity: number, bytes: Uint8Array, context: string): number;
}

export function createManagedHarnessCanvasHost(deps: ManagedHarnessCanvasHostDependencies) {
  const surfaces = new Map<number, number>();
  const canvasPointersByToken = new Map<number, WasmHandleLike>();
  const canvasTokensByPointer = new Map<string, number>();
  const offscreenCanvasTokens = new Map<number, number>();
  let nextCanvasToken = 0x30000000;
  const core = (): CoreModule => deps.getRuntime().core;
  const ptr = (p: WasmHandleLike): WasmHandleLike => {
    const token = pointerToHeapOffset(p);
    const canvasPtr = canvasPointersByToken.get(token);
    if (canvasPtr !== undefined) {
      return canvasPtr;
    }
    return normalizePointerForWasm(core(), p);
  };
  const pointerKey = (p: WasmHandleLike): string => normalizePointerForWasm(core(), p).toString();
  const tokenForCanvasPointer = (canvasPtr: WasmHandleLike): number => {
    const normalizedCanvasPtr = normalizePointerForWasm(core(), canvasPtr);
    const key = pointerKey(normalizedCanvasPtr);
    const existingToken = canvasTokensByPointer.get(key);
    if (existingToken !== undefined) {
      return existingToken;
    }
    const token = nextCanvasToken++;
    canvasTokensByPointer.set(key, token);
    canvasPointersByToken.set(token, normalizedCanvasPtr);
    return token;
  };

  const I: Record<string, unknown> = {};

  I.fui_canvas_save = (p: WasmHandleLike): void => { core()._ed_canvas_save(ptr(p)); };
  I.fui_canvas_restore = (p: WasmHandleLike): void => { core()._ed_canvas_restore(ptr(p)); };
  I.fui_canvas_translate = (p: WasmHandleLike, x: number, y: number): void => { core()._ed_canvas_translate(ptr(p), x, y); };
  I.fui_canvas_scale = (p: WasmHandleLike, sx: number, sy: number): void => { core()._ed_canvas_scale(ptr(p), sx, sy); };
  I.fui_canvas_rotate = (p: WasmHandleLike, d: number): void => { core()._ed_canvas_rotate(ptr(p), d); };
  I.fui_canvas_clip_rect = (p: WasmHandleLike, x: number, y: number, w: number, h: number): void => { core()._ed_canvas_clip_rect(ptr(p), x, y, w, h); };
  I.fui_canvas_clip_round_rect = (
    p: WasmHandleLike,
    x: number,
    y: number,
    w: number,
    h: number,
    tl: number,
    tr: number,
    br: number,
    bl: number,
  ): void => { core()._ed_canvas_clip_round_rect(ptr(p), x, y, w, h, tl, tr, br, bl); };
  I.fui_canvas_draw_rect = (p: WasmHandleLike, x: number, y: number, w: number, h: number, fc: number, sc: number, sw: number) =>
  { core()._ed_canvas_draw_rect(ptr(p), x, y, w, h, fc, sc, sw); };
  I.fui_canvas_draw_circle = (p: WasmHandleLike, cx: number, cy: number, r: number, fc: number, sc: number, sw: number) =>
  { core()._ed_canvas_draw_circle(ptr(p), cx, cy, r, fc, sc, sw); };
  I.fui_canvas_draw_line = (p: WasmHandleLike, x1: number, y1: number, x2: number, y2: number, c: number, sw: number) =>
  { core()._ed_canvas_draw_line(ptr(p), x1, y1, x2, y2, c, sw); };
  I.fui_canvas_draw_round_rect = (p: WasmHandleLike, x: number, y: number, w: number, h: number, rx: number, ry: number, fc: number, sc: number, sw: number) =>
  { core()._ed_canvas_draw_round_rect(ptr(p), x, y, w, h, rx, ry, fc, sc, sw); };
  I.fui_canvas_draw_path = (p: WasmHandleLike, pid: number, fc: number, sc: number, sw: number) =>
  { core()._ed_canvas_draw_path(ptr(p), pid, fc, sc, sw); };
  I.fui_canvas_draw_text_node = (p: WasmHandleLike, lo: number, hi: number, x: number, y: number) =>
  { core()._ed_canvas_draw_text_node(ptr(p), lo >>> 0, hi >>> 0, x, y); };
  I.fui_canvas_draw_image = (p: WasmHandleLike, tid: number, x: number, y: number, w: number, h: number, sk: number, ma: number) =>
  { core()._ed_canvas_draw_image(ptr(p), tid, x, y, w, h, sk, ma); };
  I.fui_canvas_draw_svg = (p: WasmHandleLike, sid: number, x: number, y: number, w: number, h: number) =>
  { core()._ed_canvas_draw_svg(ptr(p), sid, x, y, w, h); };
  I.fui_canvas_draw_batch = (p: WasmHandleLike, wordsPtr: WasmHandleLike, wordCount: number) => {
    if (wordCount <= 0) {
      return;
    }
    const wordByteLength = wordCount * 4;
    const words = deps.readAppBytes(pointerToHeapOffset(wordsPtr), wordByteLength);
    const c = core();
    withHeapBytes(c, words, (heap) => {
      c._ed_canvas_draw_batch(ptr(p), heap.ptr, wordCount);
    });
  };

  I.fui_path_create = () => core()._ed_path_create();
  I.fui_path_destroy = (id: number): void => { core()._ed_path_destroy(id); };
  I.fui_path_move_to = (id: number, x: number, y: number): void => { core()._ed_path_move_to(id, x, y); };
  I.fui_path_line_to = (id: number, x: number, y: number): void => { core()._ed_path_line_to(id, x, y); };
  I.fui_path_quad_to = (id: number, cx: number, cy: number, x: number, y: number): void => { core()._ed_path_quad_to(id, cx, cy, x, y); };
  I.fui_path_cubic_to = (id: number, cx1: number, cy1: number, cx2: number, cy2: number, x: number, y: number) =>
  { core()._ed_path_cubic_to(id, cx1, cy1, cx2, cy2, x, y); };
  I.fui_path_close = (id: number): void => { core()._ed_path_close(id); };
  I.fui_path_add_rect = (id: number, x: number, y: number, w: number, h: number): void => { core()._ed_path_add_rect(id, x, y, w, h); };
  I.fui_path_add_circle = (id: number, cx: number, cy: number, r: number): void => { core()._ed_path_add_circle(id, cx, cy, r); };

  I.fui_canvas_create_offscreen = (w: number, h: number) => {
    const id = core()._ed_canvas_create_offscreen(w, h);
    if (id !== 0) surfaces.set(id, id);
    return id;
  };
  I.fui_canvas_get_offscreen_ptr = (id: number) => {
    const existingToken = offscreenCanvasTokens.get(id);
    if (existingToken !== undefined) {
      return existingToken;
    }
    const token = tokenForCanvasPointer(core()._ed_canvas_get_offscreen_canvas(id));
    offscreenCanvasTokens.set(id, token);
    return token;
  };
  I.fui_canvas_read_offscreen_pixels = (id: number, outPtr: WasmHandleLike, w: number, h: number) => {
    const bytesLen = w * h * 4;
    const c = core();
    withHeapAllocation(c, bytesLen, (heap) => {
      c._ed_canvas_read_offscreen_pixels(id, heap.ptr);
      deps.writeAppBytes(pointerToHeapOffset(outPtr), bytesLen, copyBytesFromHeap(c, heap.ptr, bytesLen), 'canvas-read');
    });
  };
  I.fui_canvas_destroy_offscreen = (id: number): void => {
    surfaces.delete(id);
    const token = offscreenCanvasTokens.get(id);
    if (token !== undefined) {
      offscreenCanvasTokens.delete(id);
      canvasPointersByToken.delete(token);
      for (const [key, value] of canvasTokensByPointer.entries()) {
        if (value === token) {
          canvasTokensByPointer.delete(key);
          break;
        }
      }
    }
    core()._ed_canvas_destroy_offscreen(id);
  };

  return { imports: I, tokenForCanvasPointer };
}
