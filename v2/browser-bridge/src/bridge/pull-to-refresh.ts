const PULL_TO_REFRESH_ID = 'effindom-pull-to-refresh';
const PULL_TO_REFRESH_TRIGGER_DISTANCE = 88;
const PULL_TO_REFRESH_MAX_TRAVEL = 78;

export const PULL_TO_REFRESH_THRESHOLD = PULL_TO_REFRESH_TRIGGER_DISTANCE;

interface PullToRefreshElements {
  readonly root: HTMLDivElement;
  readonly icon: HTMLSpanElement;
}

function ensurePullToRefreshElements(): PullToRefreshElements {
  const existing = document.getElementById(PULL_TO_REFRESH_ID);
  if (existing instanceof HTMLDivElement) {
    const icon = existing.firstElementChild;
    if (icon instanceof HTMLSpanElement) {
      return { root: existing, icon };
    }
    existing.remove();
  }

  const root = document.createElement('div');
  const icon = document.createElement('span');
  root.id = PULL_TO_REFRESH_ID;
  root.hidden = true;
  root.dataset.visible = 'false';
  root.dataset.armed = 'false';
  root.setAttribute('aria-hidden', 'true');
  root.style.position = 'fixed';
  root.style.left = '50%';
  root.style.top = '12px';
  root.style.width = '48px';
  root.style.height = '48px';
  root.style.display = 'flex';
  root.style.alignItems = 'center';
  root.style.justifyContent = 'center';
  root.style.borderRadius = '999px';
  root.style.background = 'rgba(248, 250, 252, 0.96)';
  root.style.color = '#0f172a';
  root.style.boxShadow = '0 10px 28px rgba(15, 23, 42, 0.22)';
  root.style.backdropFilter = 'blur(12px)';
  root.style.pointerEvents = 'none';
  root.style.opacity = '0';
  root.style.transform = 'translate(-50%, -18px)';
  root.style.transition = 'opacity 120ms ease, transform 120ms ease';
  root.style.zIndex = '2147483647';

  icon.textContent = '↻';
  icon.style.display = 'block';
  icon.style.font = '600 24px/1 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';
  icon.style.transform = 'rotate(0deg)';
  icon.style.transition = 'transform 90ms linear';
  root.appendChild(icon);
  document.body.appendChild(root);
  return { root, icon };
}

export interface PullToRefreshOverlay {
  show(pullDistance: number): void;
  hide(immediate?: boolean): void;
  destroy(): void;
}

export function createPullToRefreshOverlay(): PullToRefreshOverlay {
  const { root, icon } = ensurePullToRefreshElements();
  let visible = false;
  let hideTimer = 0;

  return {
    show(pullDistance: number): void {
      if (hideTimer !== 0) {
        window.clearTimeout(hideTimer);
        hideTimer = 0;
      }
      visible = true;
      const distance = Math.max(0, pullDistance);
      const normalized = Math.max(0, Math.min(distance / PULL_TO_REFRESH_TRIGGER_DISTANCE, 1));
      const travel = Math.min(distance * 0.75, PULL_TO_REFRESH_MAX_TRAVEL);
      root.hidden = false;
      root.dataset.visible = 'true';
      root.dataset.armed = normalized >= 1 ? 'true' : 'false';
      root.style.transition = 'none';
      root.style.opacity = distance <= 0 ? '0' : String(0.15 + (normalized * 0.85));
      root.style.transform = `translate(-50%, ${String(travel - 18)}px)`;
      root.style.background = normalized >= 1 ? 'rgba(219, 234, 254, 0.98)' : 'rgba(248, 250, 252, 0.96)';
      icon.style.transform = `rotate(${String(Math.round(normalized * 360))}deg)`;
    },
    hide(immediate = false): void {
      if (!visible && root.hidden) {
        return;
      }
      visible = false;
      root.dataset.visible = 'false';
      root.dataset.armed = 'false';
      root.style.transition = immediate
        ? 'none'
        : 'opacity 90ms ease-out, transform 90ms ease-out, background 90ms ease-out';
      root.style.opacity = '0';
      root.style.transform = 'translate(-50%, -18px)';
      root.style.background = 'rgba(248, 250, 252, 0.96)';
      icon.style.transform = 'rotate(0deg)';
      if (immediate) {
        if (hideTimer !== 0) {
          window.clearTimeout(hideTimer);
          hideTimer = 0;
        }
        root.hidden = true;
        return;
      }
      hideTimer = window.setTimeout(() => {
        if (root.dataset.visible === 'false') {
          root.hidden = true;
        }
        hideTimer = 0;
      }, 100);
    },
    destroy(): void {
      if (hideTimer !== 0) {
        window.clearTimeout(hideTimer);
      }
      root.remove();
    },
  };
}
