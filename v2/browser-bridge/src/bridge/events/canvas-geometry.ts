const DEFAULT_LOGICAL_WIDTH = 320;
const DEFAULT_LOGICAL_HEIGHT = 220;

export function ensureCanvasLogicalSize(canvas: HTMLCanvasElement): void {
  const rect = canvas.getBoundingClientRect();
  if (rect.width <= 0 || rect.height <= 0) {
    canvas.style.width = `${String(DEFAULT_LOGICAL_WIDTH)}px`;
    canvas.style.height = `${String(DEFAULT_LOGICAL_HEIGHT)}px`;
  }
}

export function getCanvasSizeSource(canvas: HTMLCanvasElement): HTMLElement | HTMLCanvasElement {
  const source = canvas.closest('[data-effindom-canvas-size-source]');
  return source instanceof HTMLElement ? source : canvas;
}

export function readCanvasLogicalSize(canvas: HTMLCanvasElement): { readonly width: number; readonly height: number } {
  const sizeSource = getCanvasSizeSource(canvas);
  if (sizeSource.clientWidth > 0 && sizeSource.clientHeight > 0) {
    return {
      width: sizeSource.clientWidth,
      height: sizeSource.clientHeight,
    };
  }
  const styleWidth = Number.parseFloat(canvas.style.width);
  const styleHeight = Number.parseFloat(canvas.style.height);
  if (Number.isFinite(styleWidth) && styleWidth > 0 && Number.isFinite(styleHeight) && styleHeight > 0) {
    return { width: styleWidth, height: styleHeight };
  }
  return {
    width: canvas.clientWidth || DEFAULT_LOGICAL_WIDTH,
    height: canvas.clientHeight || DEFAULT_LOGICAL_HEIGHT,
  };
}

export function getPointerPosition(
  canvas: HTMLCanvasElement,
  event: { readonly clientX: number; readonly clientY: number },
): { readonly x: number; readonly y: number } {
  const sizeSource = getCanvasSizeSource(canvas);
  const rect = sizeSource.getBoundingClientRect();
  const logicalSize = readCanvasLogicalSize(canvas);
  const contentLeft = rect.left + sizeSource.clientLeft;
  const contentTop = rect.top + sizeSource.clientTop;
  const displayWidth = sizeSource.clientWidth || (rect.width - (sizeSource.clientLeft + sizeSource.clientLeft)) || DEFAULT_LOGICAL_WIDTH;
  const displayHeight = sizeSource.clientHeight || (rect.height - (sizeSource.clientTop + sizeSource.clientTop)) || DEFAULT_LOGICAL_HEIGHT;
  const x = displayWidth > 0 ? ((event.clientX - contentLeft) / displayWidth) * logicalSize.width : 0;
  const y = displayHeight > 0 ? ((event.clientY - contentTop) / displayHeight) * logicalSize.height : 0;
  return { x, y };
}

export function isPointerInsideCanvas(
  canvas: HTMLCanvasElement,
  event: { readonly clientX: number; readonly clientY: number },
): boolean {
  const rect = getCanvasSizeSource(canvas).getBoundingClientRect();
  return event.clientX >= rect.left &&
    event.clientX <= rect.right &&
    event.clientY >= rect.top &&
    event.clientY <= rect.bottom;
}

export function normalizeWheelDelta(event: WheelEvent, canvas: HTMLCanvasElement): { readonly x: number; readonly y: number } {
  let deltaX = event.deltaX;
  let deltaY = event.deltaY;
  if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) {
    deltaX *= 16.0;
    deltaY *= 16.0;
  } else if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) {
    const logicalSize = readCanvasLogicalSize(canvas);
    deltaX *= logicalSize.width;
    deltaY *= logicalSize.height;
  }
  if (event.shiftKey && Math.abs(deltaX) < 0.001 && Math.abs(deltaY) > 0.0) {
    return { x: deltaY, y: 0.0 };
  }
  return { x: deltaX, y: deltaY };
}
