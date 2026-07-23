const DEFAULT_ACCENT_COLOR = 0x2563ebff;
const URL_PREVIEW_BAR_ID = 'fui-url-bar';
const PLATFORM_FAMILY_UNKNOWN = 0;
const PLATFORM_FAMILY_APPLE = 1;
const PLATFORM_FAMILY_WINDOWS = 2;
const PLATFORM_FAMILY_LINUX = 3;
const LOADING_OVERLAY_ID = 'effindom-loading-overlay';
const LOADING_TITLE_ID = 'effindom-loading-title';
const LOADING_DETAIL_ID = 'effindom-loading-detail';

const DEFAULT_LOADING_OVERLAY_STYLES = `
.effindom-loading-overlay{--effindom-loader-color:#38bdf8;--effindom-loader-background:rgba(6,12,21,.88);position:absolute;inset:0;display:grid;place-items:center;padding:24px;box-sizing:border-box;background:linear-gradient(145deg,rgba(2,6,23,.76),rgba(15,23,42,.9));backdrop-filter:blur(10px);z-index:2;color:#e2e8f0;opacity:1;transition:opacity 140ms ease;font-family:ui-sans-serif,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;cursor:default;user-select:none;-webkit-user-select:none}
.effindom-loading-overlay[hidden]{display:none}.effindom-loading-card{max-width:420px;padding:24px 26px;border:1px solid rgba(148,163,184,.28);border-radius:20px;background:var(--effindom-loader-background);text-align:center;box-shadow:0 22px 60px rgba(2,6,23,.34)}
.effindom-loading-visual{width:88px;height:64px;margin:0 auto 18px;color:var(--effindom-loader-color)}.effindom-loading-frame{fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;stroke-dasharray:164;animation:effindom-frame 1.8s cubic-bezier(.65,0,.35,1) infinite}.effindom-loading-node{fill:currentColor;transform-box:fill-box;transform-origin:center;animation:effindom-node 1.8s ease-in-out infinite}.effindom-loading-node-b{animation-delay:-.9s}
.effindom-loading-kicker{margin:0 0 8px;font:700 11px/1.2 system-ui,sans-serif;letter-spacing:.12em;text-transform:uppercase;color:#7dd3fc}.effindom-loading-title{margin:0;font:600 24px/1.2 system-ui,sans-serif}.effindom-loading-detail{margin:10px 0 0;font:14px/1.5 system-ui,sans-serif;color:#cbd5e1}.effindom-loading-overlay[data-state=error] .effindom-loading-card{border-color:rgba(248,113,113,.5);background:rgba(69,10,10,.86)}.effindom-loading-overlay[data-state=error] .effindom-loading-visual{display:none}
@keyframes effindom-frame{0%{stroke-dashoffset:164;opacity:.35}45%,65%{stroke-dashoffset:0;opacity:1}100%{stroke-dashoffset:-164;opacity:.35}}@keyframes effindom-node{0%,100%{transform:scale(.65);opacity:.35}50%{transform:scale(1.15);opacity:1}}@media(prefers-reduced-motion:reduce){.effindom-loading-frame,.effindom-loading-node{animation:none}.effindom-loading-frame{stroke-dashoffset:0}}
`;

const DEFAULT_LOADING_OVERLAY_BODY = `
<div class="effindom-loading-card">
  <svg class="effindom-loading-visual" data-effindom-loading-visual viewBox="0 0 88 64" aria-hidden="true">
    <rect class="effindom-loading-frame" x="8" y="8" width="72" height="48" rx="12" />
    <circle class="effindom-loading-node" cx="24" cy="32" r="4" />
    <circle class="effindom-loading-node effindom-loading-node-b" cx="64" cy="32" r="4" />
  </svg>
  <p class="effindom-loading-kicker">EffinDOM</p>
  <h2 class="effindom-loading-title" id="${LOADING_TITLE_ID}" data-effindom-loading-title>Loading application</h2>
  <p class="effindom-loading-detail" id="${LOADING_DETAIL_ID}" data-effindom-loading-detail>Preparing the runtime...</p>
</div>`;

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
  ensureLoadingOverlay(): HTMLElement {
    const existing = document.getElementById(LOADING_OVERLAY_ID);
    if (existing instanceof HTMLElement) {
      return existing;
    }
    if (document.getElementById('effindom-loading-overlay-default-styles') === null) {
      const styles = document.createElement('style');
      styles.id = 'effindom-loading-overlay-default-styles';
      styles.textContent = DEFAULT_LOADING_OVERLAY_STYLES;
      document.head.appendChild(styles);
    }
    const overlay = document.createElement('div');
    overlay.id = LOADING_OVERLAY_ID;
    overlay.className = 'effindom-loading-overlay';
    overlay.dataset.state = 'loading';
    overlay.setAttribute('aria-live', 'polite');
    overlay.setAttribute('aria-hidden', 'true');
    overlay.hidden = true;
    overlay.innerHTML = DEFAULT_LOADING_OVERLAY_BODY;
    const canvas = document.getElementById('fui-canvas');
    (canvas?.parentElement ?? document.body).appendChild(overlay);
    return overlay;
  }

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
    const overlay = this.ensureLoadingOverlay();
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

  showLoading(state: 'loading' | 'error', title: string, detail: string): void {
    this.setLoadingOverlay(state, title, detail);
  }

  hideLoading(): void {
    this.hideLoadingOverlay();
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
