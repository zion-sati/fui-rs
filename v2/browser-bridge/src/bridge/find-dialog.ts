import type {
BridgeRuntime,
OpenCanvasFindMatch,
OpenCanvasFindState,
OpenCanvasResolvedFindOptions,
} from '../core-types';
import { DEFAULT_OPEN_CANVAS_FIND_OPTIONS } from './find-session';
import { isMobileBrowser,isPlatformShortcutKey } from './platform';

type FindOptionKey = keyof OpenCanvasResolvedFindOptions;
type FindIconKind = 'chevronUp' | 'chevronDown' | 'filter' | 'close';

interface FindDialogPalette {
  readonly colorScheme: 'dark' | 'light';
  readonly shellBackground: string;
  readonly shellBorder: string;
  readonly divider: string;
  readonly panelBackground: string;
  readonly text: string;
  readonly mutedText: string;
  readonly errorText: string;
  readonly buttonText: string;
  readonly buttonMutedText: string;
  readonly buttonActiveBackground: string;
  readonly buttonPlainHoverBackground: string;
  readonly filterButtonBackground: string;
  readonly filterButtonActiveBackground: string;
  readonly optionHoverBackground: string;
  readonly toggleTrackOff: string;
  readonly toggleTrackOn: string;
  readonly toggleThumb: string;
}

function isFindShortcut(event: Pick<KeyboardEvent, 'key' | 'ctrlKey' | 'metaKey' | 'altKey'>): boolean {
  return isPlatformShortcutKey(event) &&
    (event.key === 'f' || event.key === 'F');
}

function isNextShortcut(event: Pick<KeyboardEvent, 'key' | 'ctrlKey' | 'metaKey' | 'altKey' | 'shiftKey'>): boolean {
  return isPlatformShortcutKey(event) &&
    (event.key === 'g' || event.key === 'G');
}

function createSvgIcon(kind: FindIconKind): SVGSVGElement {
  const ns = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(ns, 'svg');
  svg.setAttribute('viewBox', '0 0 16 16');
  svg.setAttribute('fill', 'none');
  svg.setAttribute('stroke', 'currentColor');
  svg.setAttribute('stroke-width', kind === 'filter' ? '1.6' : '1.8');
  svg.setAttribute('stroke-linecap', 'round');
  svg.setAttribute('stroke-linejoin', 'round');
  svg.setAttribute('aria-hidden', 'true');
  svg.style.display = 'block';
  svg.style.width = '16px';
  svg.style.height = '16px';

  const appendPath = (d: string): void => {
   const path = document.createElementNS(ns, 'path');
   path.setAttribute('d', d);
   svg.appendChild(path);
  };

  switch (kind) {
   case 'chevronUp':
     appendPath('M4.5 10.25L8 6.75L11.5 10.25');
     break;
   case 'chevronDown':
     appendPath('M4.5 5.75L8 9.25L11.5 5.75');
     break;
   case 'filter':
     appendPath('M3 4.5H13');
     appendPath('M5 8H11');
     appendPath('M6.75 11.5H9.25');
     break;
   case 'close':
     appendPath('M4.75 4.75L11.25 11.25');
     appendPath('M11.25 4.75L4.75 11.25');
     break;
  }

  return svg;
}

function createIconButton(
  label: string,
  icon: FindIconKind,
  variant: 'plain' | 'filter' = 'plain',
): HTMLButtonElement {
  const button = document.createElement('button');
  button.type = 'button';
  button.setAttribute('aria-label', label);
  button.setAttribute('data-ed-find-icon-variant', variant);
  button.appendChild(createSvgIcon(icon));
  button.style.width = '32px';
  button.style.height = '32px';
  button.style.display = 'inline-flex';
  button.style.alignItems = 'center';
  button.style.justifyContent = 'center';
  button.style.padding = '0';
  button.style.border = 'none';
  button.style.borderRadius = '10px';
  button.style.background = 'transparent';
  button.style.outline = 'none';
  button.style.appearance = 'none';
  button.style.boxShadow = 'none';
  button.style.cursor = 'pointer';
  button.style.userSelect = 'none';
  button.style.flex = '0 0 auto';
  button.style.transition = 'background 120ms ease, color 120ms ease, opacity 120ms ease';
  button.dataset.edHovered = '0';
  return button;
}

function sameMatch(left: OpenCanvasFindMatch | null, right: OpenCanvasFindMatch | null): boolean {
  return left?.handle === right?.handle &&
    left?.start === right?.start &&
    left?.end === right?.end;
}

export class DesktopFindDialogController {
  private readonly runtime: BridgeRuntime;
  private readonly root: HTMLDivElement;
  private readonly input: HTMLInputElement;
  private readonly status: HTMLSpanElement;
  private readonly advancedPanel: HTMLDivElement;
  private readonly previousButton: HTMLButtonElement;
  private readonly nextButton: HTMLButtonElement;
  private readonly disclosureButton: HTMLButtonElement;
  private readonly closeButton: HTMLButtonElement;
  private readonly optionButtons: Record<FindOptionKey, HTMLButtonElement>;
  private readonly optionTracks: Record<FindOptionKey, HTMLSpanElement>;
  private readonly optionThumbs: Record<FindOptionKey, HTMLSpanElement>;
  private readonly themeMedia: MediaQueryList | null;
  private matches: OpenCanvasFindMatch[] = [];
  private activeMatchIndex = -1;
  private options: OpenCanvasResolvedFindOptions = { ...DEFAULT_OPEN_CANVAS_FIND_OPTIONS };
  private optionsExpanded = false;

  private readonly handleThemeChange = (): void => {
    this.syncThemeUi();
  };

  public constructor(runtime: BridgeRuntime) {
    this.runtime = runtime;
    const parent = document.body;
    if (!(parent instanceof HTMLElement)) {
      throw new Error('Expected document root for desktop Find dialog.');
    }

    const root = document.createElement('div');
    root.setAttribute('data-ed-find-dialog', '1');
    root.setAttribute('data-ed-open', '0');
    root.setAttribute('role', 'dialog');
    root.setAttribute('aria-label', 'Find on page');
    root.style.position = 'fixed';
    root.style.top = '6px';
    root.style.right = '6px';
    root.style.display = 'none';
    root.style.flexDirection = 'column';
    root.style.width = 'min(356px, calc(100vw - 12px))';
    root.style.boxSizing = 'border-box';
    root.style.borderRadius = '14px';
    root.style.border = '1px solid transparent';
    root.style.overflow = 'hidden';
    root.style.pointerEvents = 'auto';
    root.style.zIndex = '2147483647';
    root.style.boxShadow = '0 10px 24px rgba(0, 0, 0, 0.22)';
    root.style.backdropFilter = 'blur(10px)';
    root.style.font = '500 13px/1.2 system-ui, sans-serif';

    const topRow = document.createElement('div');
    topRow.style.display = 'flex';
    topRow.style.alignItems = 'center';
    topRow.style.minHeight = '42px';
    topRow.style.padding = '0 8px 0 12px';

    const searchShell = document.createElement('div');
    searchShell.style.flex = '1 1 auto';
    searchShell.style.minWidth = '0';
    searchShell.style.display = 'flex';
    searchShell.style.alignItems = 'center';
    searchShell.style.paddingRight = '6px';

    const input = document.createElement('input');
    input.type = 'text';
    input.autocomplete = 'off';
    input.spellcheck = false;
    input.placeholder = 'Find text';
    input.setAttribute('aria-label', 'Find query');
    input.style.width = '100%';
    input.style.minWidth = '0';
    input.style.height = '24px';
    input.style.padding = '0';
    input.style.margin = '0';
    input.style.border = 'none';
    input.style.outline = 'none';
    input.style.background = 'transparent';
    input.style.font = '500 13px/1.2 system-ui, sans-serif';
    searchShell.appendChild(input);

    const status = document.createElement('span');
    status.setAttribute('aria-live', 'polite');
    status.style.minWidth = '36px';
    status.style.textAlign = 'center';
    status.style.font = '500 13px/1.2 system-ui, sans-serif';
    status.style.padding = '0 4px 0 2px';

    const divider = document.createElement('div');
    divider.setAttribute('aria-hidden', 'true');
    divider.style.width = '1px';
    divider.style.height = '20px';
    divider.style.margin = '0 6px 0 4px';
    divider.style.flex = '0 0 auto';

    const previousButton = createIconButton('Previous result', 'chevronUp');
    const nextButton = createIconButton('Next result', 'chevronDown');
    const disclosureButton = createIconButton('Show advanced find options', 'filter', 'filter');
    const closeButton = createIconButton('Close find dialog', 'close');

    const navigationGroup = document.createElement('div');
    navigationGroup.style.display = 'flex';
    navigationGroup.style.alignItems = 'center';
    navigationGroup.style.gap = '2px';
    navigationGroup.append(previousButton, nextButton, disclosureButton, closeButton);

    topRow.append(searchShell, status, divider, navigationGroup);

    const advancedPanel = document.createElement('div');
    advancedPanel.style.display = 'none';
    advancedPanel.style.flexDirection = 'column';
    advancedPanel.style.padding = '2px 0';

    const createToggleRow = (
      key: FindOptionKey,
      label: string,
    ): { readonly button: HTMLButtonElement; readonly track: HTMLSpanElement; readonly thumb: HTMLSpanElement } => {
      const button = document.createElement('button');
      button.type = 'button';
      button.setAttribute('data-ed-find-option', key);
      button.setAttribute('aria-label', label);
      button.setAttribute('role', 'switch');
      button.style.width = '100%';
      button.style.height = '38px';
      button.style.display = 'flex';
      button.style.alignItems = 'center';
      button.style.justifyContent = 'space-between';
      button.style.padding = '0 12px';
      button.style.border = 'none';
      button.style.background = 'transparent';
      button.style.cursor = 'pointer';
      button.style.textAlign = 'left';
      button.style.font = '500 12px/1.2 system-ui, sans-serif';

      const labelSpan = document.createElement('span');
      labelSpan.textContent = label;
      labelSpan.style.pointerEvents = 'none';
      labelSpan.style.font = 'inherit';

      const track = document.createElement('span');
      track.setAttribute('aria-hidden', 'true');
      track.style.position = 'relative';
      track.style.display = 'inline-flex';
      track.style.alignItems = 'center';
      track.style.width = '30px';
      track.style.height = '18px';
      track.style.borderRadius = '999px';
      track.style.transition = 'background 120ms ease';
      track.style.pointerEvents = 'none';

      const thumb = document.createElement('span');
      thumb.style.position = 'absolute';
      thumb.style.top = '2px';
      thumb.style.left = '2px';
      thumb.style.width = '14px';
      thumb.style.height = '14px';
      thumb.style.borderRadius = '999px';
      thumb.style.boxShadow = '0 1px 2px rgba(0, 0, 0, 0.24)';
      thumb.style.transition = 'transform 120ms ease';
      thumb.style.pointerEvents = 'none';
      track.appendChild(thumb);

      button.append(labelSpan, track);
      return { button, track, thumb };
    };

    const highlightAllRow = createToggleRow('highlightAll', 'Highlight all');
    const matchCaseRow = createToggleRow('matchCase', 'Match case');
    const matchDiacriticsRow = createToggleRow('matchDiacritics', 'Match diacritics');
    const wholeWordsRow = createToggleRow('wholeWords', 'Match whole word');
    const optionButtons = {
      highlightAll: highlightAllRow.button,
      matchCase: matchCaseRow.button,
      matchDiacritics: matchDiacriticsRow.button,
      wholeWords: wholeWordsRow.button,
    } satisfies Record<FindOptionKey, HTMLButtonElement>;
    const optionTracks = {
      highlightAll: highlightAllRow.track,
      matchCase: matchCaseRow.track,
      matchDiacritics: matchDiacriticsRow.track,
      wholeWords: wholeWordsRow.track,
    } satisfies Record<FindOptionKey, HTMLSpanElement>;
    const optionThumbs = {
      highlightAll: highlightAllRow.thumb,
      matchCase: matchCaseRow.thumb,
      matchDiacritics: matchDiacriticsRow.thumb,
      wholeWords: wholeWordsRow.thumb,
    } satisfies Record<FindOptionKey, HTMLSpanElement>;
    advancedPanel.append(
      optionButtons.highlightAll,
      optionButtons.matchCase,
      optionButtons.matchDiacritics,
      optionButtons.wholeWords,
    );

    root.append(topRow, advancedPanel);
    parent.appendChild(root);

    const preserveInputFocus = (): void => {
      this.restoreInputFocus(false);
    };
    const setButtonHovered = (
      button: HTMLButtonElement,
      hovered: boolean,
    ): void => {
      button.dataset.edHovered = hovered ? '1' : '0';
      this.syncStatusUi();
    };
    const bindNonBlurringButton = (
      button: HTMLButtonElement,
      callback: () => void,
    ): void => {
      button.addEventListener('pointerdown', (event) => {
        event.preventDefault();
      });
      button.addEventListener('pointerenter', () => {
        setButtonHovered(button, true);
      });
      button.addEventListener('pointerleave', () => {
        setButtonHovered(button, false);
      });
      button.addEventListener('click', () => {
        callback();
        preserveInputFocus();
      });
    };

    root.addEventListener('keydown', (event) => {
      event.stopPropagation();
    });
    input.addEventListener('input', () => {
      this.refreshMatches(false);
    });
    input.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        this.step(event.shiftKey ? -1 : 1);
        return;
      }
      if (event.key === 'ArrowDown' && !event.altKey && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        this.step(1);
        return;
      }
      if (event.key === 'ArrowUp' && !event.altKey && !event.ctrlKey && !event.metaKey) {
        event.preventDefault();
        this.step(-1);
        return;
      }
      if (event.key === 'Escape') {
        event.preventDefault();
        this.hide();
      }
    });
    bindNonBlurringButton(previousButton, () => {
      this.step(-1);
    });
    bindNonBlurringButton(nextButton, () => {
      this.step(1);
    });
    bindNonBlurringButton(disclosureButton, () => {
      this.optionsExpanded = !this.optionsExpanded;
      this.syncOptionUi();
    });
    bindNonBlurringButton(closeButton, () => {
      this.hide();
    });
    for (const [key, button] of Object.entries(optionButtons) as [FindOptionKey, HTMLButtonElement][]) {
      bindNonBlurringButton(button, () => {
        this.options = {
          ...this.options,
          [key]: !this.options[key],
        };
        this.syncOptionUi();
        this.refreshMatches(true);
      });
    }

    this.root = root;
    this.input = input;
    this.status = status;
    this.advancedPanel = advancedPanel;
    this.previousButton = previousButton;
    this.nextButton = nextButton;
    this.disclosureButton = disclosureButton;
    this.closeButton = closeButton;
    this.optionButtons = optionButtons;
    this.optionTracks = optionTracks;
    this.optionThumbs = optionThumbs;
    this.themeMedia = typeof window.matchMedia === 'function'
      ? window.matchMedia('(prefers-color-scheme: dark)')
      : null;
    this.themeMedia?.addEventListener('change', this.handleThemeChange);
    this.syncOptionUi();
    this.syncThemeUi();
  }

  public consumeGlobalKeyEvent(event: KeyboardEvent, type: 'down' | 'up'): boolean {
    if (isMobileBrowser()) {
      return false;
    }

    if (isFindShortcut(event)) {
      event.preventDefault();
      event.stopPropagation();
      if (type === 'down') {
        this.show();
      }
      return true;
    }

    if (type === 'down' && this.isOpen() && !event.altKey && !event.ctrlKey && !event.metaKey && event.key === 'F3') {
      event.preventDefault();
      this.step(event.shiftKey ? -1 : 1);
      return true;
    }

    if (type === 'down' && this.isOpen() && isNextShortcut(event)) {
      event.preventDefault();
      this.step(event.shiftKey ? -1 : 1);
      return true;
    }

    if (this.containsTarget(event.target)) {
      return true;
    }

    return false;
  }

  public containsTarget(target: EventTarget | null): boolean {
    return target instanceof Node && this.root.contains(target);
  }

  public destroy(): void {
    this.themeMedia?.removeEventListener('change', this.handleThemeChange);
    this.root.remove();
  }

  private getPalette(): FindDialogPalette {
    const darkMode = this.themeMedia?.matches ?? true;
    if (darkMode) {
      return {
        colorScheme: 'dark',
        shellBackground: 'rgba(36, 36, 36, 0.98)',
        shellBorder: 'rgba(255, 255, 255, 0.08)',
        divider: 'rgba(255, 255, 255, 0.12)',
        panelBackground: 'rgba(0, 0, 0, 0.10)',
        text: '#f5f5f5',
        mutedText: '#cfcfcf',
        errorText: '#fca5a5',
        buttonText: '#d9d9d9',
        buttonMutedText: '#7c7c7c',
        buttonActiveBackground: 'rgba(255, 255, 255, 0.08)',
        buttonPlainHoverBackground: 'rgba(255, 255, 255, 0.06)',
        filterButtonBackground: 'transparent',
        filterButtonActiveBackground: 'rgba(255, 255, 255, 0.20)',
        optionHoverBackground: 'rgba(255, 255, 255, 0.03)',
        toggleTrackOff: '#5a5a5a',
        toggleTrackOn: '#818cf8',
        toggleThumb: '#ffffff',
      };
    }
    return {
      colorScheme: 'light',
      shellBackground: 'rgba(251, 251, 251, 0.98)',
      shellBorder: 'rgba(15, 23, 42, 0.12)',
      divider: 'rgba(15, 23, 42, 0.12)',
      panelBackground: 'rgba(15, 23, 42, 0.03)',
      text: '#111827',
      mutedText: '#4b5563',
      errorText: '#dc2626',
      buttonText: '#374151',
      buttonMutedText: '#9ca3af',
      buttonActiveBackground: 'rgba(15, 23, 42, 0.08)',
      buttonPlainHoverBackground: 'rgba(15, 23, 42, 0.05)',
      filterButtonBackground: 'transparent',
      filterButtonActiveBackground: 'rgba(15, 23, 42, 0.16)',
      optionHoverBackground: 'rgba(15, 23, 42, 0.03)',
      toggleTrackOff: '#d1d5db',
      toggleTrackOn: '#818cf8',
      toggleThumb: '#ffffff',
    };
  }

  private isOpen(): boolean {
    return this.root.style.display !== 'none';
  }

  private restoreInputFocus(selectAll: boolean): void {
    const selectionStart = this.input.selectionStart ?? this.input.value.length;
    const selectionEnd = this.input.selectionEnd ?? selectionStart;
    queueMicrotask(() => {
      if (!this.isOpen()) {
        return;
      }
      this.input.focus({ preventScroll: true });
      if (selectAll) {
        this.input.select();
        return;
      }
      this.input.setSelectionRange(selectionStart, selectionEnd);
    });
  }

  private syncThemeUi(): void {
    const palette = this.getPalette();
    this.root.style.colorScheme = palette.colorScheme;
    this.root.style.background = palette.shellBackground;
    this.root.style.borderColor = palette.shellBorder;
    this.root.style.color = palette.text;
    this.advancedPanel.style.background = palette.panelBackground;
    this.advancedPanel.style.borderTop = this.optionsExpanded ? `1px solid ${palette.divider}` : 'none';
    this.input.style.color = palette.text;
    this.input.style.caretColor = palette.text;
    this.status.style.color = palette.mutedText;
    this.previousButton.style.color = palette.buttonText;
    this.nextButton.style.color = palette.buttonText;
    this.closeButton.style.color = palette.buttonText;
    this.disclosureButton.style.color = palette.buttonText;
    this.previousButton.style.background = 'transparent';
    this.nextButton.style.background = 'transparent';
    this.closeButton.style.background = 'transparent';
    this.disclosureButton.style.background = this.optionsExpanded
      ? palette.filterButtonActiveBackground
      : palette.filterButtonBackground;
    this.syncOptionUi();
    this.syncStatusUi();
  }

  private syncOptionUi(): void {
    const palette = this.getPalette();
    this.root.setAttribute('data-ed-open', this.isOpen() ? '1' : '0');
    this.advancedPanel.style.display = this.optionsExpanded ? 'flex' : 'none';
    this.advancedPanel.style.borderTop = this.optionsExpanded ? `1px solid ${palette.divider}` : 'none';
    this.disclosureButton.setAttribute(
      'aria-label',
      this.optionsExpanded ? 'Hide advanced find options' : 'Show advanced find options',
    );
    for (const [key, button] of Object.entries(this.optionButtons) as [FindOptionKey, HTMLButtonElement][]) {
      const pressed = this.options[key];
      button.setAttribute('aria-checked', pressed ? 'true' : 'false');
      button.style.color = palette.text;
      button.style.background = 'transparent';
      const track = this.optionTracks[key];
      const thumb = this.optionThumbs[key];
      track.style.background = pressed ? palette.toggleTrackOn : palette.toggleTrackOff;
      thumb.style.background = palette.toggleThumb;
      thumb.style.transform = pressed ? 'translateX(12px)' : 'translateX(0)';
    }
  }

  private syncStatusUi(): void {
    const palette = this.getPalette();
    const query = this.input.value;
    if (query.length === 0) {
      this.status.textContent = '';
      this.status.style.color = palette.mutedText;
    } else if (this.matches.length === 0 || this.activeMatchIndex < 0) {
      this.status.textContent = '0/0';
      this.status.style.color = palette.errorText;
    } else {
      this.status.textContent = `${String(this.activeMatchIndex + 1)}/${String(this.matches.length)}`;
      this.status.style.color = palette.mutedText;
    }
    const hasMatches = this.matches.length > 0;
    const syncButtonState = (button: HTMLButtonElement, variant: 'plain' | 'filter'): void => {
      button.disabled = !hasMatches && button !== this.disclosureButton && button !== this.closeButton;
      const enabled = !button.disabled;
      const hovered = button.dataset.edHovered === '1';
      button.style.opacity = enabled ? '1' : '0.45';
      button.style.cursor = enabled ? 'pointer' : 'default';
      if (variant === 'filter') {
        if (this.optionsExpanded) {
          button.style.background = palette.filterButtonActiveBackground;
        } else if (enabled && hovered) {
          button.style.background = palette.buttonPlainHoverBackground;
        } else {
          button.style.background = palette.filterButtonBackground;
        }
      } else {
        button.style.background = enabled && hovered
          ? palette.buttonPlainHoverBackground
          : 'transparent';
      }
      button.style.color = enabled ? palette.buttonText : palette.buttonMutedText;
    };
    syncButtonState(this.previousButton, 'plain');
    syncButtonState(this.nextButton, 'plain');
    syncButtonState(this.disclosureButton, 'filter');
    syncButtonState(this.closeButton, 'plain');
  }

  private show(): void {
    this.root.style.display = 'flex';
    this.root.setAttribute('data-ed-open', '1');
    this.syncThemeUi();
    this.refreshMatches(true);
    this.restoreInputFocus(true);
  }

  private hide(): void {
    this.root.style.display = 'none';
    this.root.setAttribute('data-ed-open', '0');
    this.previousButton.dataset.edHovered = '0';
    this.nextButton.dataset.edHovered = '0';
    this.disclosureButton.dataset.edHovered = '0';
    this.closeButton.dataset.edHovered = '0';
    this.runtime.openCanvasApi.setFindState(null);
  }

  private refreshMatches(preserveCurrentMatch: boolean): void {
    const query = this.input.value;
    const previousMatch =
      preserveCurrentMatch && this.activeMatchIndex >= 0
        ? this.matches[this.activeMatchIndex] ?? null
        : null;
    const results = this.runtime.openCanvasApi.findText(query, this.options);
    this.options = { ...results.options };
    this.matches = results.matches.map((match) => ({ ...match }));
    if (this.matches.length === 0) {
      this.activeMatchIndex = -1;
      this.runtime.openCanvasApi.setFindState(null);
      this.syncOptionUi();
      this.syncStatusUi();
      this.restoreInputFocus(false);
      return;
    }

    if (previousMatch !== null) {
      const preservedIndex = this.matches.findIndex((entry) => sameMatch(entry, previousMatch));
      this.activeMatchIndex = preservedIndex >= 0 ? preservedIndex : 0;
    } else if (this.activeMatchIndex < 0 || this.activeMatchIndex >= this.matches.length) {
      this.activeMatchIndex = 0;
    }

    this.applyCurrentState(true);
  }

  private step(delta: 1 | -1): void {
    if (this.matches.length === 0) {
      this.refreshMatches(false);
      return;
    }
    this.activeMatchIndex = (this.activeMatchIndex + delta + this.matches.length) % this.matches.length;
    this.applyCurrentState(true);
  }

  private applyCurrentState(revealActive: boolean): void {
    const state: OpenCanvasFindState | null =
      this.matches.length === 0 || this.activeMatchIndex < 0 || this.activeMatchIndex >= this.matches.length
        ? null
        : {
          query: this.input.value,
          options: { ...this.options },
          matches: this.matches.map((match) => ({ ...match })),
          activeMatchIndex: this.activeMatchIndex,
        };
    if (state === null || !this.runtime.openCanvasApi.setFindState(state, revealActive)) {
      this.matches = [];
      this.activeMatchIndex = -1;
      this.runtime.openCanvasApi.setFindState(null);
    }
    this.syncOptionUi();
    this.syncStatusUi();
    this.restoreInputFocus(false);
  }
}
