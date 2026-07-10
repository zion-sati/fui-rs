const DEFAULT_ACCENT_COLOR = 0x2563ebff;
const URL_PREVIEW_BAR_ID = 'fui-url-bar';
const PLATFORM_FAMILY_UNKNOWN = 0;
const PLATFORM_FAMILY_APPLE = 1;
const PLATFORM_FAMILY_WINDOWS = 2;
const PLATFORM_FAMILY_LINUX = 3;
const LOADING_OVERLAY_ID = 'effindom-loading-overlay';
const LOADING_TITLE_ID = 'effindom-loading-title';
const LOADING_DETAIL_ID = 'effindom-loading-detail';

function packColor(red: number, green: number, blue: number, alpha = 255): number {
  return ((((red & 0xff) << 24) | ((green & 0xff) << 16) | ((blue & 0xff) << 8) | (alpha & 0xff)) >>> 0);
}

function parseCssColorToRgba(colorValue: string): number | null {
  const probe = document.createElement('span');
  probe.style.color = colorValue;
  if (probe.style.color.length === 0) {
    return null;
  }
  probe.style.position = 'absolute';
  probe.style.pointerEvents = 'none';
  probe.style.opacity = '0';
  document.body.appendChild(probe);
  const computed = getComputedStyle(probe).color.trim();
  probe.remove();

  const match = /^rgba?\(([^)]+)\)$/.exec(computed);
  if (match === null) {
    return null;
  }
  const channels = match[1];
  if (channels === undefined) {
    return null;
  }
  const parts = channels.split(',').map((part) => part.trim());
  if (parts.length < 3) {
    return null;
  }
  const [redPart, greenPart, bluePart, alphaPart] = parts;
  if (redPart === undefined || greenPart === undefined || bluePart === undefined) {
    return null;
  }
  const red = Number.parseInt(redPart, 10);
  const green = Number.parseInt(greenPart, 10);
  const blue = Number.parseInt(bluePart, 10);
  if ([red, green, blue].some((channel) => Number.isNaN(channel))) {
    return null;
  }

  const alpha = parts.length < 4
    ? 255
    : Math.max(0, Math.min(255, Math.round(Number.parseFloat(alphaPart ?? '1') * 255)));
  return packColor(red, green, blue, alpha);
}

function hexByte(value: number): string {
  return Math.max(0, Math.min(255, value | 0)).toString(16).padStart(2, '0');
}

export function formatPackedColorAsHex(color: number): string {
  const normalized = color >>> 0;
  return `#${hexByte((normalized >>> 24) & 0xff)}${hexByte((normalized >>> 16) & 0xff)}${hexByte((normalized >>> 8) & 0xff)}`;
}

function readCssSystemAccentColor(): number | null {
  if (typeof CSS === 'undefined' || typeof CSS.supports !== 'function' || !CSS.supports('color', 'AccentColor')) {
    return null;
  }
  return parseCssColorToRgba('AccentColor');
}

function readCssRootAccentColor(): number | null {
  const accent = getComputedStyle(document.documentElement).getPropertyValue('accent-color').trim();
  if (accent.length === 0 || accent === 'auto') {
    return null;
  }
  return parseCssColorToRgba(accent);
}

function readWebkitFocusRingAccentColor(): number | null {
  return parseCssColorToRgba('-webkit-focus-ring-color');
}

export function readHostAccentColor(): number {
  return readCssSystemAccentColor() ?? readWebkitFocusRingAccentColor() ?? readCssRootAccentColor() ?? DEFAULT_ACCENT_COLOR;
}

function ensureUrlPreviewBar(): HTMLDivElement {
  const existing = document.getElementById(URL_PREVIEW_BAR_ID);
  if (existing instanceof HTMLDivElement) {
    return existing;
  }

  const bar = document.createElement('div');
  bar.id = URL_PREVIEW_BAR_ID;
  bar.hidden = true;
  bar.dataset.visible = 'false';
  bar.setAttribute('aria-hidden', 'true');
  bar.style.position = 'fixed';
  bar.style.left = '12px';
  bar.style.bottom = '12px';
  bar.style.maxWidth = 'min(60vw, 720px)';
  bar.style.padding = '6px 10px';
  bar.style.borderRadius = '10px';
  bar.style.background = 'rgba(15, 23, 42, 0.84)';
  bar.style.color = '#f8fafc';
  bar.style.font = '12px/1.4 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';
  bar.style.letterSpacing = '0.01em';
  bar.style.whiteSpace = 'nowrap';
  bar.style.overflow = 'hidden';
  bar.style.textOverflow = 'ellipsis';
  bar.style.pointerEvents = 'none';
  bar.style.opacity = '0';
  bar.style.transform = 'translateY(6px)';
  bar.style.transition = 'opacity 120ms ease, transform 120ms ease';
  bar.style.backdropFilter = 'blur(12px)';
  bar.style.boxShadow = '0 10px 28px rgba(2, 6, 23, 0.24)';
  bar.style.zIndex = '2147483647';
  document.body.appendChild(bar);
  return bar;
}

export function waitForFrame(): Promise<void> {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        resolve();
      });
    });
  });
}

export class HarnessUiChrome {
  getLoadingOverlayText(): { title: string; detail: string } {
    const overlay = document.getElementById(LOADING_OVERLAY_ID);
    const titleNode = document.getElementById(LOADING_TITLE_ID);
    const detailNode = document.getElementById(LOADING_DETAIL_ID);
    const title = titleNode instanceof HTMLElement ? titleNode.textContent : '';
    const detail = detailNode instanceof HTMLElement ? detailNode.textContent : '';
    if (overlay instanceof HTMLElement && title.length > 0 && detail.length > 0) {
      return { title, detail };
    }
    return {
      title: 'Loading...',
      detail: 'The runtime is starting up.',
    };
  }

  setLoadingOverlay(state: 'loading' | 'error', title: string, detail: string): void {
    const overlay = document.getElementById(LOADING_OVERLAY_ID);
    const titleNode = document.getElementById(LOADING_TITLE_ID);
    const detailNode = document.getElementById(LOADING_DETAIL_ID);
    if (!(overlay instanceof HTMLElement) || !(titleNode instanceof HTMLElement) || !(detailNode instanceof HTMLElement)) {
      return;
    }
    overlay.dataset.state = state;
    overlay.hidden = false;
    overlay.setAttribute('aria-hidden', 'false');
    titleNode.textContent = title;
    detailNode.textContent = detail;
  }

  hideLoadingOverlay(): void {
    const overlay = document.getElementById(LOADING_OVERLAY_ID);
    if (!(overlay instanceof HTMLElement)) {
      return;
    }
    overlay.hidden = true;
    overlay.dataset.state = 'ready';
    overlay.setAttribute('aria-hidden', 'true');
  }

  setUrlPreviewText(text: string): void {
    const bar = ensureUrlPreviewBar();
    if (text.length === 0) {
      bar.textContent = '';
      bar.hidden = true;
      bar.dataset.visible = 'false';
      bar.style.opacity = '0';
      bar.style.transform = 'translateY(6px)';
      window.__fuiUrlPreviewText = '';
      return;
    }

    bar.textContent = text;
    bar.hidden = false;
    bar.dataset.visible = 'true';
    bar.style.opacity = '1';
    bar.style.transform = 'translateY(0)';
    window.__fuiUrlPreviewText = text;
  }

  readHostAccentColor(): number {
    return readHostAccentColor();
  }

  detectPlatformFamily(): number {
    const navigatorWithUserAgentData = navigator as Navigator & {
      userAgentData?: {
        platform?: string;
      };
    };
    const platform = (navigatorWithUserAgentData.userAgentData?.platform ?? navigator.userAgent).toLowerCase();
    if (
      platform.includes('mac') ||
      platform.includes('iphone') ||
      platform.includes('ipad') ||
      platform.includes('ipod') ||
      platform.includes('ios')
    ) {
      return PLATFORM_FAMILY_APPLE;
    }
    if (platform.includes('win')) {
      return PLATFORM_FAMILY_WINDOWS;
    }
    if (
      platform.includes('linux') ||
      platform.includes('android') ||
      platform.includes('x11') ||
      platform.includes('cros')
    ) {
      return PLATFORM_FAMILY_LINUX;
    }
    return PLATFORM_FAMILY_UNKNOWN;
  }

  detectCoarsePointer(): boolean {
    return window.matchMedia('(pointer: coarse)').matches || navigator.maxTouchPoints > 0;
  }

  getCanvasSizeSource(canvas: HTMLCanvasElement): HTMLElement | HTMLCanvasElement {
    const source = canvas.closest('[data-effindom-canvas-size-source]');
    return source instanceof HTMLElement ? source : canvas;
  }
}
