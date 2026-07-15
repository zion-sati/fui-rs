import type { ClipboardWritePayload, EffinDomCallbacks } from '../core-types';

export interface ManagedPlatformHost {
  nowMilliseconds(): number;
  getDevicePixelRatio(): number;
  isDarkMode(): boolean;
  reload(): void;
  resolveUrl(target: string): URL;
  publishClipboard(payload: ClipboardWritePayload): void;
  defer(callback: () => void): number;
  setTimer(callback: () => void, delayMs: number): number;
  clearTimer(timerId: number): void;
}

export class BrowserManagedPlatformHost implements ManagedPlatformHost {
  public nowMilliseconds(): number {
    return performance.now();
  }

  public getDevicePixelRatio(): number {
    return window.devicePixelRatio > 0 ? window.devicePixelRatio : 1;
  }

  public isDarkMode(): boolean {
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  }

  public reload(): void {
    window.location.reload();
  }

  public resolveUrl(target: string): URL {
    return new URL(target, window.location.href);
  }

  public publishClipboard(payload: ClipboardWritePayload): void {
    const callbacks: EffinDomCallbacks | undefined = window.__effindomCallbacks;
    callbacks?.onClipboardWrite?.(payload);
  }

  public defer(callback: () => void): number {
    return window.setTimeout(callback, 0);
  }

  public setTimer(callback: () => void, delayMs: number): number {
    return window.setTimeout(callback, delayMs);
  }

  public clearTimer(timerId: number): void {
    window.clearTimeout(timerId);
  }
}

export const browserManagedPlatformHost: ManagedPlatformHost = new BrowserManagedPlatformHost();
