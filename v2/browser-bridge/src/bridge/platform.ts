export enum PlatformFamily {
  Unknown = 0,
  Apple = 1,
  Windows = 2,
  Linux = 3,
}

export function isMobileBrowser(): boolean {
  const navigatorWithUserAgentData = navigator as Navigator & {
    userAgentData?: {
      mobile?: boolean;
    };
  };
  if (navigatorWithUserAgentData.userAgentData?.mobile === true) {
    return true;
  }
  if (/Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent)) {
    return true;
  }
  return /Macintosh/i.test(navigator.userAgent) && navigator.maxTouchPoints > 1;
}

export function detectPlatformFamily(): PlatformFamily {
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
    return PlatformFamily.Apple;
  }
  if (platform.includes('win')) {
    return PlatformFamily.Windows;
  }
  if (
    platform.includes('linux') ||
    platform.includes('android') ||
    platform.includes('x11') ||
    platform.includes('cros')
  ) {
    return PlatformFamily.Linux;
  }
  return PlatformFamily.Unknown;
}

export function isPlatformShortcutKey(
  event: Pick<KeyboardEvent, 'ctrlKey' | 'metaKey' | 'altKey'>,
  platformFamily = detectPlatformFamily(),
): boolean {
  if (event.altKey) {
    return false;
  }
  if (platformFamily === PlatformFamily.Apple) {
    return event.metaKey && !event.ctrlKey;
  }
  return event.ctrlKey && !event.metaKey;
}
