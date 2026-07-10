import type { DevToolsDomMirrorMode } from '../../runtime-config';
import type { DebugTreeBounds, DebugTreeNode, DebugTreeSnapshot } from '../../debug-tree';
import { getPointerPosition } from '../events/canvas-geometry';

const MIRROR_ROOT_ID = 'effindom-devtools-dom-mirror';
const OVERLAY_ROOT_ID = 'effindom-devtools-overlay';
const DEBUG_DIALOG_ID = 'effindom-devtools-debug-dialog';

const SEMANTIC_ROLE_NAMES: Record<number, string> = {
  1: 'button',
  2: 'textbox',
  3: 'link',
  4: 'heading',
  5: 'form',
  6: 'list',
  7: 'list-item',
  8: 'image',
  9: 'dialog',
  10: 'static-text',
  11: 'checkbox',
  12: 'radio',
  13: 'radio-group',
  14: 'switch',
  15: 'slider',
  16: 'combo-box',
};

const SEMANTIC_CONTROL_TAGS: Record<number, string> = {
  1: 'fui-button',
  2: 'fui-textbox',
  3: 'fui-link',
  5: 'fui-form',
  6: 'fui-list',
  7: 'fui-list-item',
  9: 'fui-dialog',
  11: 'fui-checkbox',
  12: 'fui-radio',
  13: 'fui-radio-group',
  14: 'fui-switch',
  15: 'fui-slider',
  16: 'fui-combo-box',
};

interface DevToolsDialogPalette {
  readonly colorScheme: 'dark' | 'light';
  readonly shellBackground: string;
  readonly shellBorder: string;
  readonly divider: string;
  readonly panelBackground: string;
  readonly text: string;
  readonly mutedText: string;
  readonly buttonText: string;
  readonly buttonMutedText: string;
  readonly buttonPlainHoverBackground: string;
  readonly optionHoverBackground: string;
  readonly toggleTrackOff: string;
  readonly toggleTrackOn: string;
  readonly toggleThumb: string;
}

interface MirrorEntry {
  element: HTMLElement;
  scrollContent: HTMLElement | null;
  tagName: string;
  attributeSignature: string;
  geometryAttributeSignature: string;
  frameSignature: string;
  scrollContentSignature: string;
}

interface DevToolsDomMirrorOptions {
  readonly hitTest: (x: number, y: number) => string | null;
}

function formatBounds(bounds: DebugTreeBounds): string {
  return `${String(bounds.x)},${String(bounds.y)},${String(bounds.width)},${String(bounds.height)}`;
}

function setOptionalAttribute(element: Element, name: string, value: string | null): void {
  if (value === null || value === '') {
    element.removeAttribute(name);
    return;
  }
  element.setAttribute(name, value);
}

function setBooleanAttribute(element: Element, name: string, value: boolean): void {
  if (value) {
    element.setAttribute(name, 'true');
  } else {
    element.removeAttribute(name);
  }
}

function formatCssPx(value: number): string {
  return `${String(Math.max(0, value))}px`;
}

function formatCssTranslate(x: number, y: number): string {
  return `translate(${String(x)}px, ${String(y)}px)`;
}

function makeRootFrameSignature(canvas: HTMLCanvasElement): string | null {
  const parent = canvas.parentElement;
  if (!(parent instanceof HTMLElement)) {
    return null;
  }
  const canvasRect = canvas.getBoundingClientRect();
  const parentRect = parent.getBoundingClientRect();
  return [
    canvasRect.left - parentRect.left,
    canvasRect.top - parentRect.top,
    canvasRect.width,
    canvasRect.height,
  ].join(',');
}

function applyRootFrame(root: HTMLElement, signature: string): void {
  const [left, top, width, height] = signature.split(',');
  root.style.left = `${left ?? '0'}px`;
  root.style.top = `${top ?? '0'}px`;
  root.style.width = formatCssPx(Number(width ?? 0));
  root.style.height = formatCssPx(Number(height ?? 0));
}

function makeNodeFrameSignature(
  node: DebugTreeNode,
  parentNode: DebugTreeNode | undefined,
): string {
  const parentX = parentNode?.bounds.x ?? 0;
  const parentY = parentNode?.bounds.y ?? 0;
  const scrollOffsetX = parentNode?.behavior.scrollView === true ? parentNode.scroll.offsetX : 0;
  const scrollOffsetY = parentNode?.behavior.scrollView === true ? parentNode.scroll.offsetY : 0;
  return [
    node.bounds.x - parentX + scrollOffsetX,
    node.bounds.y - parentY + scrollOffsetY,
    Math.max(0, node.bounds.width),
    Math.max(0, node.bounds.height),
    node.flags.clipToBounds || node.behavior.scrollView ? 'hidden' : 'visible',
  ].join(',');
}

function applyNodeFrame(element: HTMLElement, signature: string): void {
  const [left, top, width, height, overflow] = signature.split(',');
  element.style.left = `${left ?? '0'}px`;
  element.style.top = `${top ?? '0'}px`;
  element.style.width = formatCssPx(Number(width ?? 0));
  element.style.height = formatCssPx(Number(height ?? 0));
  element.style.overflow = overflow ?? 'visible';
}

function makeScrollContentSignature(node: DebugTreeNode): string {
  if (!node.behavior.scrollView) {
    return 'none';
  }
  return [
    Math.max(0, node.scroll.contentWidth),
    Math.max(0, node.scroll.contentHeight),
    -node.scroll.offsetX,
    -node.scroll.offsetY,
  ].join(',');
}

function applyScrollContentFrame(element: HTMLElement, signature: string): void {
  const [width, height, translateX, translateY] = signature.split(',');
  element.style.display = 'block';
  element.style.width = formatCssPx(Number(width ?? 0));
  element.style.height = formatCssPx(Number(height ?? 0));
  element.style.transform = formatCssTranslate(Number(translateX ?? 0), Number(translateY ?? 0));
}

function transformViewportBounds(
  bounds: DebugTreeBounds,
  scale: number,
  offsetX: number,
  offsetY: number,
): DebugTreeBounds {
  return {
    x: (bounds.x * scale) + offsetX,
    y: (bounds.y * scale) + offsetY,
    width: bounds.width * scale,
    height: bounds.height * scale,
  };
}

function containsPoint(bounds: DebugTreeBounds, x: number, y: number): boolean {
  return bounds.width > 0 &&
    bounds.height > 0 &&
    x >= bounds.x &&
    x <= bounds.x + bounds.width &&
    y >= bounds.y &&
    y <= bounds.y + bounds.height;
}

function makeAttributeSignature(node: DebugTreeNode): string {
  return [
    node.handle,
    node.parentHandle ?? '',
    node.nodeId,
    node.nodeTypeName,
    node.nodeType,
    node.semanticRole,
    node.semanticLabel,
    node.scroll.nearestScrollAncestorHandle ?? '',
    node.behavior.interactive ? '1' : '0',
    node.behavior.focusable ? '1' : '0',
    node.behavior.editable ? '1' : '0',
    node.behavior.customDrawable ? '1' : '0',
    node.behavior.portal ? '1' : '0',
  ].join('|');
}

function makeGeometryAttributeSignature(node: DebugTreeNode): string {
  return [
    formatBounds(node.bounds),
    formatBounds(node.visibleBounds),
    node.flags.clippedOrEmpty ? '1' : '0',
    node.behavior.scrollView
      ? `${String(node.scroll.offsetX)},${String(node.scroll.offsetY)},${String(node.scroll.contentWidth)},${String(node.scroll.contentHeight)},${String(node.scroll.viewportWidth)},${String(node.scroll.viewportHeight)}`
      : '',
  ].join('|');
}

function getMirrorTagName(node: DebugTreeNode): string {
  const semanticControlTag = SEMANTIC_CONTROL_TAGS[node.semanticRole];
  if (semanticControlTag !== undefined) {
    return semanticControlTag;
  }
  if (node.semanticRole === 4 && node.nodeTypeName !== 'text') {
    return 'fui-heading';
  }
  if (node.semanticRole === 8 && node.nodeTypeName !== 'svg') {
    return 'fui-image';
  }
  switch (node.nodeTypeName) {
    case 'flex-box':
      return 'fui-flex-box';
    case 'text':
      return 'fui-text';
    case 'image':
      return 'fui-image';
    case 'svg':
      return 'fui-svg';
    case 'scroll-view':
      return 'fui-scroll-view';
    case 'grid':
      return 'fui-grid';
    case 'path':
      return 'fui-path';
    default:
      return 'fui-node';
  }
}

function getDebugTypeName(node: DebugTreeNode): string {
  const roleName = SEMANTIC_ROLE_NAMES[node.semanticRole];
  if (roleName !== undefined && node.semanticRole !== 10) {
    return roleName;
  }
  return node.nodeTypeName;
}

function initializeMirrorElement(element: HTMLElement): HTMLElement {
  element.style.display = 'block';
  element.style.position = 'absolute';
  element.style.boxSizing = 'border-box';
  element.style.margin = '0';
  element.style.padding = '0';
  element.style.pointerEvents = 'none';
  return element;
}

function createMirrorElement(tagName: string): HTMLElement {
  return initializeMirrorElement(document.createElement(tagName));
}

function createScrollContentElement(): HTMLElement {
  const element = document.createElement('fui-scroll-content');
  element.style.display = 'none';
  element.style.position = 'absolute';
  element.style.left = '0';
  element.style.top = '0';
  element.style.boxSizing = 'border-box';
  element.style.margin = '0';
  element.style.padding = '0';
  element.style.pointerEvents = 'none';
  element.style.transformOrigin = '0 0';
  return element;
}

function createDevToolsCloseIcon(): SVGSVGElement {
  const ns = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(ns, 'svg');
  svg.setAttribute('viewBox', '0 0 16 16');
  svg.setAttribute('fill', 'none');
  svg.setAttribute('stroke', 'currentColor');
  svg.setAttribute('stroke-width', '1.8');
  svg.setAttribute('stroke-linecap', 'round');
  svg.setAttribute('stroke-linejoin', 'round');
  svg.setAttribute('aria-hidden', 'true');
  svg.style.display = 'block';
  svg.style.width = '16px';
  svg.style.height = '16px';
  const first = document.createElementNS(ns, 'path');
  first.setAttribute('d', 'M4.75 4.75L11.25 11.25');
  const second = document.createElementNS(ns, 'path');
  second.setAttribute('d', 'M11.25 4.75L4.75 11.25');
  svg.append(first, second);
  return svg;
}

function createDevToolsIconButton(label: string): HTMLButtonElement {
  const button = document.createElement('button');
  button.type = 'button';
  button.setAttribute('aria-label', label);
  button.setAttribute('data-fui-devtools-icon-button', 'true');
  button.appendChild(createDevToolsCloseIcon());
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
  button.style.setProperty('-webkit-appearance', 'none');
  button.style.boxShadow = 'none';
  button.style.cursor = 'pointer';
  button.style.userSelect = 'none';
  button.style.setProperty('-webkit-user-select', 'none');
  button.style.flex = '0 0 auto';
  button.style.transition = 'background 120ms ease, color 120ms ease, opacity 120ms ease';
  button.dataset.edHovered = '0';
  return button;
}

export class DevToolsDomMirror {
  private readonly entriesByHandle = new Map<string, MirrorEntry>();
  private disposeShortcut: (() => void) | null = null;
  private geometryAttributeSyncTimer: number | null = null;
  private root: HTMLDivElement | null = null;
  private overlayRoot: HTMLDivElement | null = null;
  private overlayBox: HTMLDivElement | null = null;
  private debugDialogRoot: HTMLDivElement | null = null;
  private rootFrameSignature = '';
  private active = false;
  private inspectActive = false;
  private lastSnapshot: DebugTreeSnapshot | null = null;
  private selectedHandle: string | null = null;
  private hoveredHandle: string | null = null;
  private disposeInspectMode: (() => void) | null = null;
  private viewportScale = 1.0;
  private viewportOffsetX = 0.0;
  private viewportOffsetY = 0.0;
  private readonly themeMedia: MediaQueryList | null;
  private readonly handleThemeChange = (): void => {
    this.syncDebugDialog();
  };

  public constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly mode: DevToolsDomMirrorMode,
    private readonly options: DevToolsDomMirrorOptions,
  ) {
    this.themeMedia = typeof window.matchMedia === 'function'
      ? window.matchMedia('(prefers-color-scheme: dark)')
      : null;
    this.themeMedia?.addEventListener('change', this.handleThemeChange);
    if (mode === 'enabled') {
      this.activate();
    }
    if (mode !== 'disabled') {
      const onKeyDown = (event: KeyboardEvent): void => {
        if (!event.metaKey || !event.shiftKey || event.code !== 'F12') {
          return;
        }
        event.preventDefault();
        this.toggleDebugDialog();
      };
      document.addEventListener('keydown', onKeyDown, { capture: true });
      this.disposeShortcut = () => {
        document.removeEventListener('keydown', onKeyDown, { capture: true });
      };
    }
  }

  public sync(snapshot: DebugTreeSnapshot): void {
    this.lastSnapshot = snapshot;
    if (!this.active) {
      return;
    }
    const root = this.ensureRoot();
    const rootFrameSignature = makeRootFrameSignature(this.canvas);
    if (rootFrameSignature !== null && rootFrameSignature !== this.rootFrameSignature) {
      applyRootFrame(root, rootFrameSignature);
      if (this.overlayRoot !== null) {
        applyRootFrame(this.overlayRoot, rootFrameSignature);
      }
      this.rootFrameSignature = rootFrameSignature;
    }
    const seen = new Set<string>();
    for (const node of snapshot.nodes) {
      this.syncNode(node, seen);
    }
    for (const node of snapshot.nodes) {
      this.syncNodeParent(node, root);
    }
    for (const [handle, entry] of this.entriesByHandle) {
      if (seen.has(handle)) {
        continue;
      }
      if (handle === this.selectedHandle) {
        this.clearSelection();
      }
      if (handle === this.hoveredHandle) {
        this.setHoveredHandle(null);
      }
      entry.element.remove();
      this.entriesByHandle.delete(handle);
    }
    this.syncSelection();
    this.scheduleGeometryAttributeSync();
  }

  public syncViewportTransform(scale: number, offsetX: number, offsetY: number): void {
    this.viewportScale = scale;
    this.viewportOffsetX = offsetX;
    this.viewportOffsetY = offsetY;
    this.syncSelection();
  }

  public activate(): void {
    if (this.mode === 'disabled' || this.active) {
      return;
    }
    this.active = true;
    this.ensureRoot();
    if (this.lastSnapshot !== null) {
      this.sync(this.lastSnapshot);
    }
    this.syncDebugDialog();
  }

  public deactivate(): void {
    if (!this.active) {
      return;
    }
    this.active = false;
    if (this.geometryAttributeSyncTimer !== null) {
      window.clearTimeout(this.geometryAttributeSyncTimer);
      this.geometryAttributeSyncTimer = null;
    }
    this.entriesByHandle.clear();
    this.root?.remove();
    this.overlayRoot?.remove();
    this.root = null;
    this.overlayRoot = null;
    this.overlayBox = null;
    this.rootFrameSignature = '';
    this.selectedHandle = null;
    this.hoveredHandle = null;
    this.setInspectMode(false);
    this.syncDebugDialog();
  }

  public toggle(): boolean {
    if (this.active) {
      this.deactivate();
      return false;
    }
    this.activate();
    return this.active;
  }

  public isActive(): boolean {
    return this.active;
  }

  public selectHandle(handle: string): boolean {
    if (this.mode === 'disabled') {
      return false;
    }
    this.activate();
    const snapshot = this.lastSnapshot;
    if (snapshot?.nodesByHandle[handle] === undefined) {
      return false;
    }
    this.setSelectedHandle(handle);
    return true;
  }

  public clearSelection(): void {
    this.setSelectedHandle(null);
  }

  public getSelectedHandle(): string | null {
    return this.selectedHandle;
  }

  public openDebugDialog(): boolean {
    if (this.mode === 'disabled') {
      return false;
    }
    this.ensureDebugDialog();
    this.syncDebugDialog();
    return true;
  }

  public closeDebugDialog(): boolean {
    this.setInspectMode(false);
    this.debugDialogRoot?.remove();
    this.debugDialogRoot = null;
    return false;
  }

  public toggleDebugDialog(): boolean {
    if (this.isDebugDialogOpen()) {
      this.closeDebugDialog();
      return false;
    }
    return this.openDebugDialog();
  }

  public isDebugDialogOpen(): boolean {
    return this.debugDialogRoot !== null;
  }

  public destroy(): void {
    this.disposeShortcut?.();
    this.themeMedia?.removeEventListener('change', this.handleThemeChange);
    this.setInspectMode(false);
    if (this.geometryAttributeSyncTimer !== null) {
      window.clearTimeout(this.geometryAttributeSyncTimer);
      this.geometryAttributeSyncTimer = null;
    }
    this.deactivate();
    this.closeDebugDialog();
    this.lastSnapshot = null;
  }

  private ensureRoot(): HTMLDivElement {
    if (this.root !== null) {
      return this.root;
    }
    const parent = this.canvas.parentElement;
    if (!(parent instanceof HTMLElement)) {
      throw new Error('Expected canvas parent element for DevTools DOM mirror.');
    }
    const root = document.createElement('div');
    root.id = MIRROR_ROOT_ID;
    root.setAttribute('aria-hidden', 'true');
    root.setAttribute('inert', '');
    root.setAttribute('data-fui-devtools-dom-mirror', 'true');
    root.style.position = 'absolute';
    root.style.left = '0';
    root.style.top = '0';
    root.style.width = '0';
    root.style.height = '0';
    root.style.overflow = 'hidden';
    root.style.pointerEvents = 'none';
    root.style.opacity = '0';
    root.style.contain = 'strict';
    parent.appendChild(root);
    this.root = root;
    return root;
  }

  private ensureOverlayRoot(): HTMLDivElement {
    if (this.overlayRoot !== null) {
      return this.overlayRoot;
    }
    const parent = this.canvas.parentElement;
    if (!(parent instanceof HTMLElement)) {
      throw new Error('Expected canvas parent element for DevTools overlay.');
    }
    const root = document.createElement('div');
    root.id = OVERLAY_ROOT_ID;
    root.setAttribute('aria-hidden', 'true');
    root.setAttribute('inert', '');
    root.setAttribute('data-fui-devtools-overlay', 'true');
    root.style.position = 'absolute';
    root.style.left = '0';
    root.style.top = '0';
    root.style.width = '0';
    root.style.height = '0';
    root.style.overflow = 'hidden';
    root.style.pointerEvents = 'none';
    root.style.contain = 'strict';

    const box = document.createElement('div');
    box.setAttribute('data-fui-devtools-overlay-box', 'true');
    box.style.position = 'absolute';
    box.style.display = 'none';
    box.style.boxSizing = 'border-box';
    box.style.pointerEvents = 'none';
    box.style.border = '2px solid rgba(59, 130, 246, 0.95)';
    box.style.background = 'rgba(59, 130, 246, 0.08)';
    box.style.boxShadow = '0 0 0 1px rgba(255, 255, 255, 0.9), 0 0 0 4px rgba(59, 130, 246, 0.25)';
    root.appendChild(box);
    if (this.rootFrameSignature !== '') {
      applyRootFrame(root, this.rootFrameSignature);
    }
    parent.appendChild(root);
    this.overlayRoot = root;
    this.overlayBox = box;
    return root;
  }

  private ensureDebugDialog(): HTMLDivElement {
    if (this.debugDialogRoot !== null) {
      return this.debugDialogRoot;
    }
    const root = document.createElement('div');
    root.id = DEBUG_DIALOG_ID;
    root.setAttribute('role', 'dialog');
    root.setAttribute('aria-label', 'EffinDom Debug');
    root.setAttribute('data-fui-devtools-debug-dialog', 'true');
    root.style.position = 'fixed';
    root.style.top = '6px';
    root.style.right = '6px';
    root.style.display = 'flex';
    root.style.flexDirection = 'column';
    root.style.zIndex = '2147483647';
    root.style.width = 'min(356px, calc(100vw - 12px))';
    root.style.boxSizing = 'border-box';
    root.style.border = '1px solid transparent';
    root.style.borderRadius = '14px';
    root.style.boxShadow = '0 10px 24px rgba(0, 0, 0, 0.22)';
    root.style.backdropFilter = 'blur(10px)';
    root.style.font = '500 13px/1.2 system-ui, sans-serif';
    root.style.pointerEvents = 'auto';
    root.style.overflow = 'hidden';

    const header = document.createElement('div');
    header.setAttribute('data-fui-devtools-dialog-header', 'true');
    header.style.display = 'flex';
    header.style.alignItems = 'center';
    header.style.minHeight = '42px';
    header.style.padding = '0 8px 0 12px';

    const title = document.createElement('div');
    title.textContent = 'EffinDom Debug';
    title.style.flex = '1 1 auto';
    title.style.minWidth = '0';
    title.style.font = '500 13px/1.2 system-ui, sans-serif';
    title.style.paddingRight = '6px';

    const status = document.createElement('span');
    status.setAttribute('data-fui-devtools-dialog-mirror-status', 'true');
    status.style.flex = '0 0 auto';
    status.style.minWidth = '56px';
    status.style.textAlign = 'center';
    status.style.font = '500 13px/1.2 system-ui, sans-serif';
    status.style.padding = '0 4px 0 2px';

    const divider = document.createElement('div');
    divider.setAttribute('aria-hidden', 'true');
    divider.setAttribute('data-fui-devtools-dialog-divider', 'true');
    divider.style.width = '1px';
    divider.style.height = '20px';
    divider.style.margin = '0 6px 0 4px';
    divider.style.flex = '0 0 auto';

    const closeButton = createDevToolsIconButton('Close debug dialog');
    closeButton.setAttribute('data-fui-devtools-dialog-close', 'true');
    this.bindDialogButton(closeButton, () => {
      this.closeDebugDialog();
    });
    header.append(title, status, divider, closeButton);

    const body = document.createElement('div');
    body.setAttribute('data-fui-devtools-dialog-body', 'true');
    body.style.display = 'flex';
    body.style.flexDirection = 'column';
    body.style.padding = '2px 0';

    const mirrorRow = this.createDialogSwitchRow('DOM Mirror');
    mirrorRow.setAttribute('data-fui-devtools-dialog-mirror-row', 'true');
    const mirrorToggle = this.createDialogSwitch('Toggle DOM Mirror', 'data-fui-devtools-dialog-mirror-toggle');
    this.bindDialogButton(mirrorRow, () => {
      if (this.active) {
        this.deactivate();
      } else {
        this.activate();
      }
      this.syncDebugDialog();
    });
    mirrorRow.appendChild(mirrorToggle);

    const inspectRow = this.createDialogSwitchRow('Inspect Mode');
    inspectRow.setAttribute('data-fui-devtools-dialog-inspect-row', 'true');
    const inspectToggle = this.createDialogSwitch('Toggle Inspect Mode', 'data-fui-devtools-dialog-inspect-toggle');
    this.bindDialogButton(inspectRow, () => {
      this.setInspectMode(!this.inspectActive);
    });
    inspectRow.appendChild(inspectToggle);

    const selectedHandleRow = this.createDialogRow('Selected');
    const selectedHandle = document.createElement('span');
    selectedHandle.setAttribute('data-fui-devtools-dialog-selected-handle', 'true');
    selectedHandle.style.flex = '1 1 auto';
    selectedHandle.style.minWidth = '0';
    selectedHandle.style.overflow = 'hidden';
    selectedHandle.style.textOverflow = 'ellipsis';
    selectedHandle.style.whiteSpace = 'nowrap';
    selectedHandle.style.textAlign = 'right';
    selectedHandle.setAttribute('data-fui-devtools-dialog-muted-text', 'true');
    const clearButton = document.createElement('button');
    clearButton.type = 'button';
    clearButton.textContent = 'Clear';
    clearButton.setAttribute('data-fui-devtools-dialog-clear-selection', 'true');
    clearButton.style.marginLeft = '8px';
    clearButton.style.border = 'none';
    clearButton.style.borderRadius = '10px';
    clearButton.style.padding = '0 8px';
    clearButton.style.height = '28px';
    clearButton.style.background = 'transparent';
    clearButton.style.cursor = 'pointer';
    clearButton.style.transition = 'background 120ms ease, color 120ms ease, opacity 120ms ease';
    clearButton.dataset.edHovered = '0';
    this.bindDialogButton(clearButton, () => {
      this.clearSelection();
    });
    selectedHandleRow.append(selectedHandle, clearButton);

    const typeRow = this.createDialogInfoRow('Type', 'data-fui-devtools-dialog-selected-type');
    const roleRow = this.createDialogInfoRow('Role', 'data-fui-devtools-dialog-selected-role');
    const labelRow = this.createDialogInfoRow('Label', 'data-fui-devtools-dialog-selected-label');
    body.append(mirrorRow, inspectRow, selectedHandleRow, typeRow, roleRow, labelRow);
    root.append(header, body);
    root.addEventListener('keydown', (event) => {
      event.stopPropagation();
      if (event.key === 'Escape') {
        event.preventDefault();
        this.closeDebugDialog();
      }
    });
    document.body.appendChild(root);
    this.debugDialogRoot = root;
    return root;
  }

  private createDialogRow(labelText: string): HTMLDivElement {
    const row = document.createElement('div');
    row.style.display = 'flex';
    row.style.alignItems = 'center';
    row.style.minHeight = '34px';
    row.style.gap = '8px';
    row.style.padding = '0 12px';
    row.style.overflow = 'hidden';
    const label = document.createElement('span');
    label.textContent = labelText;
    label.setAttribute('data-fui-devtools-dialog-label', 'true');
    label.style.flex = '0 0 72px';
    label.style.minWidth = '0';
    label.style.overflow = 'hidden';
    label.style.textOverflow = 'ellipsis';
    label.style.whiteSpace = 'nowrap';
    label.style.font = '500 12px/1.2 system-ui, sans-serif';
    row.appendChild(label);
    return row;
  }

  private createDialogSwitchRow(labelText: string): HTMLButtonElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.setAttribute('aria-label', labelText);
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
    button.dataset.edHovered = '0';
    const label = document.createElement('span');
    label.textContent = labelText;
    label.setAttribute('data-fui-devtools-dialog-label', 'true');
    label.style.pointerEvents = 'none';
    label.style.font = 'inherit';
    button.appendChild(label);
    return button;
  }

  private createDialogInfoRow(labelText: string, valueAttribute: string): HTMLDivElement {
    const row = this.createDialogRow(labelText);
    const value = document.createElement('span');
    value.setAttribute(valueAttribute, 'true');
    value.setAttribute('data-fui-devtools-dialog-muted-text', 'true');
    value.style.flex = '1 1 auto';
    value.style.minWidth = '0';
    value.style.overflow = 'hidden';
    value.style.textOverflow = 'ellipsis';
    value.style.whiteSpace = 'nowrap';
    value.style.textAlign = 'right';
    row.appendChild(value);
    return row;
  }

  private createDialogSwitch(label: string, attribute: string): HTMLButtonElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.setAttribute('role', 'switch');
    button.setAttribute('aria-label', label);
    button.setAttribute(attribute, 'true');
    button.style.position = 'relative';
    button.style.flex = '0 0 auto';
    button.style.width = '30px';
    button.style.height = '18px';
    button.style.border = 'none';
    button.style.borderRadius = '999px';
    button.style.padding = '0';
    button.style.pointerEvents = 'none';
    button.style.transition = 'background 120ms ease';
    const thumb = document.createElement('span');
    thumb.setAttribute('data-fui-devtools-dialog-switch-thumb', 'true');
    thumb.style.position = 'absolute';
    thumb.style.top = '2px';
    thumb.style.left = '2px';
    thumb.style.width = '14px';
    thumb.style.height = '14px';
    thumb.style.borderRadius = '999px';
    thumb.style.boxShadow = '0 1px 2px rgba(0, 0, 0, 0.24)';
    thumb.style.transform = 'translateX(0)';
    thumb.style.transition = 'transform 120ms ease';
    thumb.style.pointerEvents = 'none';
    button.appendChild(thumb);
    return button;
  }

  private bindDialogButton(button: HTMLElement, callback: () => void): void {
    button.addEventListener('pointerdown', (event) => {
      event.preventDefault();
    });
    button.addEventListener('pointerenter', () => {
      button.dataset.edHovered = '1';
      this.syncDebugDialog();
    });
    button.addEventListener('pointerleave', () => {
      button.dataset.edHovered = '0';
      this.syncDebugDialog();
    });
    button.addEventListener('click', () => {
      callback();
    });
  }

  private setInspectMode(active: boolean): void {
    if (active) {
      if (this.mode === 'disabled' || this.inspectActive) {
        return;
      }
      this.activate();
      this.inspectActive = true;
      this.installInspectModeHandlers();
      this.syncDebugDialog();
      return;
    }
    if (!this.inspectActive) {
      return;
    }
    this.inspectActive = false;
    this.disposeInspectMode?.();
    this.disposeInspectMode = null;
    this.setHoveredHandle(null);
    this.clearSelection();
    this.syncDebugDialog();
  }

  private installInspectModeHandlers(): void {
    if (this.disposeInspectMode !== null) {
      return;
    }
    const consume = (event: Event): void => {
      event.preventDefault();
      event.stopPropagation();
      event.stopImmediatePropagation();
    };
    const hitTest = (event: { readonly clientX: number; readonly clientY: number }): string | null => {
      const position = getPointerPosition(this.canvas, event);
      return this.resolveInspectHit(position.x, position.y);
    };
    const onPointerMove = (event: PointerEvent): void => {
      consume(event);
      this.setHoveredHandle(hitTest(event));
    };
    const onPointerDown = (event: PointerEvent): void => {
      consume(event);
      this.setHoveredHandle(hitTest(event));
    };
    const onPointerUp = (event: PointerEvent): void => {
      consume(event);
      this.setHoveredHandle(hitTest(event));
    };
    const onClick = (event: MouseEvent): void => {
      consume(event);
      const handle = hitTest(event);
      this.setHoveredHandle(handle);
      if (handle !== null) {
        this.setSelectedHandle(handle);
      }
    };
    const onPointerLeave = (event: PointerEvent): void => {
      consume(event);
      this.setHoveredHandle(null);
    };
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key !== 'Escape') {
        return;
      }
      consume(event);
      this.setInspectMode(false);
    };
    this.canvas.addEventListener('pointermove', onPointerMove, { capture: true });
    this.canvas.addEventListener('pointerdown', onPointerDown, { capture: true });
    this.canvas.addEventListener('pointerup', onPointerUp, { capture: true });
    this.canvas.addEventListener('click', onClick, { capture: true });
    this.canvas.addEventListener('pointerleave', onPointerLeave, { capture: true });
    document.addEventListener('keydown', onKeyDown, { capture: true });
    this.disposeInspectMode = () => {
      this.canvas.removeEventListener('pointermove', onPointerMove, { capture: true });
      this.canvas.removeEventListener('pointerdown', onPointerDown, { capture: true });
      this.canvas.removeEventListener('pointerup', onPointerUp, { capture: true });
      this.canvas.removeEventListener('click', onClick, { capture: true });
      this.canvas.removeEventListener('pointerleave', onPointerLeave, { capture: true });
      document.removeEventListener('keydown', onKeyDown, { capture: true });
    };
  }

  private resolveInspectHit(x: number, y: number): string | null {
    const snapshot = this.lastSnapshot;
    if (snapshot === null) {
      return this.options.hitTest(x, y);
    }
    let bestHandle: string | null = null;
    let bestArea = Number.POSITIVE_INFINITY;
    for (const node of snapshot.nodes) {
      if (!containsPoint(node.visibleBounds, x, y)) {
        continue;
      }
      const area = node.visibleBounds.width * node.visibleBounds.height;
      if (area <= bestArea) {
        bestArea = area;
        bestHandle = node.handle;
      }
    }
    return bestHandle ?? this.options.hitTest(x, y);
  }

  private getDialogPalette(): DevToolsDialogPalette {
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
        buttonText: '#d9d9d9',
        buttonMutedText: '#7c7c7c',
        buttonPlainHoverBackground: 'rgba(255, 255, 255, 0.06)',
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
      buttonText: '#374151',
      buttonMutedText: '#9ca3af',
      buttonPlainHoverBackground: 'rgba(15, 23, 42, 0.05)',
      optionHoverBackground: 'rgba(15, 23, 42, 0.03)',
      toggleTrackOff: '#d1d5db',
      toggleTrackOn: '#818cf8',
      toggleThumb: '#ffffff',
    };
  }

  private syncNode(node: DebugTreeNode, seen: Set<string>): void {
    seen.add(node.handle);
    let entry = this.entriesByHandle.get(node.handle);
    if (entry === undefined) {
      const tagName = getMirrorTagName(node);
      const element = createMirrorElement(tagName);
      entry = {
        element,
        scrollContent: null,
        tagName,
        attributeSignature: '',
        geometryAttributeSignature: '',
        frameSignature: '',
        scrollContentSignature: '',
      };
      this.entriesByHandle.set(node.handle, entry);
    } else {
      this.syncNodeTag(entry, node);
    }

    const { element } = entry;
    const attributeSignature = makeAttributeSignature(node);
    if (attributeSignature !== entry.attributeSignature) {
      this.syncNodeAttributes(element, node);
      entry.attributeSignature = attributeSignature;
    }
    if (entry.geometryAttributeSignature === '') {
      this.syncNodeGeometryAttributes(entry, node);
    }

    const parentNode = node.parentHandle !== null ? this.lastSnapshot?.nodesByHandle[node.parentHandle] : undefined;
    const frameSignature = makeNodeFrameSignature(node, parentNode);
    if (frameSignature !== entry.frameSignature) {
      applyNodeFrame(element, frameSignature);
      entry.frameSignature = frameSignature;
    }

    const scrollContentSignature = makeScrollContentSignature(node);
    if (scrollContentSignature !== entry.scrollContentSignature) {
      if (scrollContentSignature === 'none') {
        entry.scrollContent?.remove();
        entry.scrollContent = null;
      } else {
        entry.scrollContent ??= createScrollContentElement();
        if (entry.scrollContent.parentElement !== element) {
          element.appendChild(entry.scrollContent);
        }
        applyScrollContentFrame(entry.scrollContent, scrollContentSignature);
      }
      entry.scrollContentSignature = scrollContentSignature;
    }
  }

  private syncNodeTag(entry: MirrorEntry, node: DebugTreeNode): void {
    const tagName = getMirrorTagName(node);
    if (tagName === entry.tagName) {
      return;
    }
    const replacement = createMirrorElement(tagName);
    for (const attribute of Array.from(entry.element.attributes)) {
      replacement.setAttribute(attribute.name, attribute.value);
    }
    for (const child of Array.from(entry.element.childNodes)) {
      replacement.appendChild(child);
    }
    entry.element.replaceWith(replacement);
    entry.element = replacement;
    entry.tagName = tagName;
  }

  private syncNodeAttributes(element: HTMLElement, node: DebugTreeNode): void {
    element.setAttribute('data-fui-handle', node.handle);
    setOptionalAttribute(element, 'data-fui-parent-handle', node.parentHandle);
    setOptionalAttribute(element, 'data-fui-node-id', node.nodeId);
    element.setAttribute('data-fui-type', getDebugTypeName(node));
    element.setAttribute('data-fui-render-node-type', node.nodeTypeName);
    element.setAttribute('data-fui-node-type', String(node.nodeType));
    if (node.semanticRole === 0) {
      element.removeAttribute('data-fui-semantic-role');
      element.removeAttribute('data-fui-semantic-role-name');
    } else {
      element.setAttribute('data-fui-semantic-role', String(node.semanticRole));
      element.setAttribute('data-fui-semantic-role-name', SEMANTIC_ROLE_NAMES[node.semanticRole] ?? `unknown-${String(node.semanticRole)}`);
    }
    setOptionalAttribute(element, 'data-fui-semantic-label', node.semanticLabel);
    setOptionalAttribute(element, 'data-fui-scroll-ancestor', node.scroll.nearestScrollAncestorHandle);
    setBooleanAttribute(element, 'data-fui-interactive', node.behavior.interactive);
    setBooleanAttribute(element, 'data-fui-focusable', node.behavior.focusable);
    setBooleanAttribute(element, 'data-fui-editable', node.behavior.editable);
    setBooleanAttribute(element, 'data-fui-custom-drawable', node.behavior.customDrawable);
    setBooleanAttribute(element, 'data-fui-portal', node.behavior.portal);
  }

  private syncNodeGeometryAttributes(entry: MirrorEntry, node: DebugTreeNode): void {
    const geometryAttributeSignature = makeGeometryAttributeSignature(node);
    if (geometryAttributeSignature === entry.geometryAttributeSignature) {
      return;
    }
    entry.element.setAttribute('data-fui-bounds', formatBounds(node.bounds));
    entry.element.setAttribute('data-fui-visible-bounds', formatBounds(node.visibleBounds));
    setBooleanAttribute(entry.element, 'data-fui-clipped', node.flags.clippedOrEmpty);
    if (node.behavior.scrollView) {
      entry.element.setAttribute(
        'data-fui-scroll',
        `${String(node.scroll.offsetX)},${String(node.scroll.offsetY)},${String(node.scroll.contentWidth)},${String(node.scroll.contentHeight)},${String(node.scroll.viewportWidth)},${String(node.scroll.viewportHeight)}`,
      );
    } else {
      entry.element.removeAttribute('data-fui-scroll');
    }
    entry.geometryAttributeSignature = geometryAttributeSignature;
  }

  private setSelectedHandle(handle: string | null): void {
    if (handle === this.selectedHandle) {
      this.syncSelection();
      return;
    }
    if (this.selectedHandle !== null) {
      const previousEntry = this.entriesByHandle.get(this.selectedHandle);
      previousEntry?.element.removeAttribute('data-fui-selected');
      previousEntry?.element.removeAttribute('data-fui-offscreen');
    }
    this.selectedHandle = handle;
    this.syncSelection();
    this.syncDebugDialog();
  }

  private setHoveredHandle(handle: string | null): void {
    if (handle === this.hoveredHandle) {
      this.syncSelection();
      return;
    }
    if (this.hoveredHandle !== null) {
      const previousEntry = this.entriesByHandle.get(this.hoveredHandle);
      previousEntry?.element.removeAttribute('data-fui-inspect-hovered');
    }
    this.hoveredHandle = handle;
    if (this.hoveredHandle !== null) {
      const entry = this.entriesByHandle.get(this.hoveredHandle);
      entry?.element.setAttribute('data-fui-inspect-hovered', 'true');
    }
    this.syncSelection();
  }

  private syncSelection(): void {
    const selectedEntry = this.selectedHandle !== null
      ? this.entriesByHandle.get(this.selectedHandle)
      : undefined;
    if (this.selectedHandle !== null && selectedEntry === undefined) {
      this.clearSelection();
      return;
    }
    if (selectedEntry !== undefined) {
      selectedEntry.element.setAttribute('data-fui-selected', 'true');
    }
    const overlayHandle = this.inspectActive && this.hoveredHandle !== null
      ? this.hoveredHandle
      : this.selectedHandle;
    if (overlayHandle === null) {
      this.overlayRoot?.remove();
      this.overlayRoot = null;
      this.overlayBox = null;
      this.syncDebugDialog();
      return;
    }
    const snapshot = this.lastSnapshot;
    const node = snapshot?.nodesByHandle[overlayHandle];
    const entry = this.entriesByHandle.get(overlayHandle);
    if (snapshot === null || node === undefined || entry === undefined) {
      if (overlayHandle === this.selectedHandle) {
        this.clearSelection();
      } else {
        this.setHoveredHandle(null);
      }
      return;
    }
    this.ensureOverlayRoot();
    const box = this.overlayBox;
    if (box === null || node.visibleBounds.width <= 0 || node.visibleBounds.height <= 0) {
      if (overlayHandle === this.selectedHandle) {
        entry.element.setAttribute('data-fui-offscreen', 'true');
      }
      if (box !== null) {
        box.style.display = 'none';
      }
      return;
    }
    if (overlayHandle === this.selectedHandle) {
      entry.element.removeAttribute('data-fui-offscreen');
    }
    const transformedBounds = transformViewportBounds(
      node.visibleBounds,
      this.viewportScale,
      this.viewportOffsetX,
      this.viewportOffsetY,
    );
    box.style.display = 'block';
    box.style.left = `${String(transformedBounds.x)}px`;
    box.style.top = `${String(transformedBounds.y)}px`;
    box.style.width = formatCssPx(transformedBounds.width);
    box.style.height = formatCssPx(transformedBounds.height);
    this.syncDebugDialog();
  }

  private syncDebugDialog(): void {
    const dialog = this.debugDialogRoot;
    if (dialog === null) {
      return;
    }
    const palette = this.getDialogPalette();
    dialog.style.colorScheme = palette.colorScheme;
    dialog.style.background = palette.shellBackground;
    dialog.style.borderColor = palette.shellBorder;
    dialog.style.color = palette.text;
    const body = dialog.querySelector<HTMLElement>('[data-fui-devtools-dialog-body="true"]');
    if (body !== null) {
      body.style.background = palette.panelBackground;
      body.style.borderTop = `1px solid ${palette.divider}`;
    }
    dialog.querySelectorAll<HTMLElement>('[data-fui-devtools-dialog-divider="true"]').forEach((divider) => {
      divider.style.background = palette.divider;
    });
    dialog.querySelectorAll<HTMLElement>('[data-fui-devtools-dialog-label="true"]').forEach((label) => {
      label.style.color = palette.text;
    });
    dialog.querySelectorAll<HTMLElement>('[data-fui-devtools-dialog-muted-text="true"]').forEach((mutedText) => {
      mutedText.style.color = palette.mutedText;
    });
    const status = dialog.querySelector<HTMLElement>('[data-fui-devtools-dialog-mirror-status="true"]');
    if (status !== null) {
      status.textContent = this.active ? 'Mirror on' : 'Mirror off';
      status.style.background = 'transparent';
      status.style.color = palette.mutedText;
    }
    const mirrorRow = dialog.querySelector<HTMLButtonElement>('[data-fui-devtools-dialog-mirror-row="true"]');
    if (mirrorRow !== null) {
      mirrorRow.setAttribute('aria-checked', this.active ? 'true' : 'false');
      mirrorRow.style.color = palette.text;
      mirrorRow.style.background = mirrorRow.dataset.edHovered === '1'
        ? palette.optionHoverBackground
        : 'transparent';
    }
    const mirrorToggle = dialog.querySelector<HTMLButtonElement>('[data-fui-devtools-dialog-mirror-toggle="true"]');
    if (mirrorToggle !== null) {
      mirrorToggle.setAttribute('aria-checked', this.active ? 'true' : 'false');
      mirrorToggle.style.background = this.active ? palette.toggleTrackOn : palette.toggleTrackOff;
      const thumb = mirrorToggle.querySelector<HTMLElement>('[data-fui-devtools-dialog-switch-thumb="true"]');
      if (thumb !== null) {
        thumb.style.background = palette.toggleThumb;
        thumb.style.transform = this.active ? 'translateX(12px)' : 'translateX(0)';
      }
    }
    const inspectRow = dialog.querySelector<HTMLButtonElement>('[data-fui-devtools-dialog-inspect-row="true"]');
    if (inspectRow !== null) {
      inspectRow.setAttribute('aria-checked', this.inspectActive ? 'true' : 'false');
      inspectRow.style.color = palette.text;
      inspectRow.style.background = inspectRow.dataset.edHovered === '1'
        ? palette.optionHoverBackground
        : 'transparent';
    }
    const inspectToggle = dialog.querySelector<HTMLButtonElement>('[data-fui-devtools-dialog-inspect-toggle="true"]');
    if (inspectToggle !== null) {
      inspectToggle.setAttribute('aria-checked', this.inspectActive ? 'true' : 'false');
      inspectToggle.style.background = this.inspectActive ? palette.toggleTrackOn : palette.toggleTrackOff;
      const thumb = inspectToggle.querySelector<HTMLElement>('[data-fui-devtools-dialog-switch-thumb="true"]');
      if (thumb !== null) {
        thumb.style.background = palette.toggleThumb;
        thumb.style.transform = this.inspectActive ? 'translateX(12px)' : 'translateX(0)';
      }
    }
    dialog.querySelectorAll<HTMLButtonElement>('[data-fui-devtools-icon-button="true"]').forEach((button) => {
      const enabled = !button.disabled;
      button.style.opacity = enabled ? '1' : '0.45';
      button.style.cursor = enabled ? 'pointer' : 'default';
      button.style.color = enabled ? palette.buttonText : palette.buttonMutedText;
      button.style.background = enabled && button.dataset.edHovered === '1'
        ? palette.buttonPlainHoverBackground
        : 'transparent';
    });
    const node = this.selectedHandle !== null ? this.lastSnapshot?.nodesByHandle[this.selectedHandle] : undefined;
    this.setDialogValue(dialog, 'data-fui-devtools-dialog-selected-handle', this.selectedHandle ?? 'None');
    this.setDialogValue(dialog, 'data-fui-devtools-dialog-selected-type', node !== undefined ? getDebugTypeName(node) : 'None');
    this.setDialogValue(
      dialog,
      'data-fui-devtools-dialog-selected-role',
      node !== undefined && node.semanticRole !== 0
        ? SEMANTIC_ROLE_NAMES[node.semanticRole] ?? `unknown-${String(node.semanticRole)}`
        : 'None',
    );
    this.setDialogValue(dialog, 'data-fui-devtools-dialog-selected-label', node?.semanticLabel ?? '');
    const clearButton = dialog.querySelector<HTMLButtonElement>('[data-fui-devtools-dialog-clear-selection="true"]');
    if (clearButton !== null) {
      clearButton.disabled = this.selectedHandle === null;
      const enabled = !clearButton.disabled;
      clearButton.style.opacity = enabled ? '1' : '0.45';
      clearButton.style.cursor = enabled ? 'pointer' : 'default';
      clearButton.style.color = enabled ? palette.buttonText : palette.buttonMutedText;
      clearButton.style.background = enabled && clearButton.dataset.edHovered === '1'
        ? palette.buttonPlainHoverBackground
        : 'transparent';
    }
  }

  private setDialogValue(dialog: HTMLElement, attribute: string, value: string): void {
    const target = dialog.querySelector<HTMLElement>(`[${attribute}="true"]`);
    if (target !== null) {
      target.textContent = value;
      target.title = value;
    }
  }

  private scheduleGeometryAttributeSync(): void {
    if (this.geometryAttributeSyncTimer !== null) {
      window.clearTimeout(this.geometryAttributeSyncTimer);
    }
    this.geometryAttributeSyncTimer = window.setTimeout(() => {
      this.geometryAttributeSyncTimer = null;
      const snapshot = this.lastSnapshot;
      if (snapshot === null || !this.active) {
        return;
      }
      for (const node of snapshot.nodes) {
        const entry = this.entriesByHandle.get(node.handle);
        if (entry !== undefined) {
          this.syncNodeGeometryAttributes(entry, node);
        }
      }
    }, 150);
  }

  private syncNodeParent(node: DebugTreeNode, root: HTMLElement): void {
    const entry = this.entriesByHandle.get(node.handle);
    if (entry === undefined) {
      return;
    }
    const parentNode = node.parentHandle !== null ? this.lastSnapshot?.nodesByHandle[node.parentHandle] : undefined;
    if (parentNode === undefined) {
      if (entry.element.parentElement !== root) {
        root.appendChild(entry.element);
      }
      return;
    }
    const parentEntry = this.entriesByHandle.get(parentNode.handle);
    if (parentEntry === undefined) {
      return;
    }
    const parentElement = parentNode.behavior.scrollView && parentEntry.scrollContent !== null
      ? parentEntry.scrollContent
      : parentEntry.element;
    if (entry.element.parentElement !== parentElement) {
      parentElement.appendChild(entry.element);
    }
  }
}
