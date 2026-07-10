const textEncoder = new TextEncoder();

function syncOrderedChildren(parent: HTMLElement, orderedChildren: readonly HTMLElement[]): void {
  let changed = parent.childElementCount !== orderedChildren.length;
  if (!changed) {
    for (let index = 0; index < orderedChildren.length; index += 1) {
      if (parent.children.item(index) !== orderedChildren[index]) {
        changed = true;
        break;
      }
    }
  }
  if (!changed) {
    return;
  }
  parent.replaceChildren(...orderedChildren);
}

export interface FindOnPageDocument {
  readonly handle: string;
  readonly text: string;
}

export interface ResolvedFindSelection {
  readonly handle: string;
  readonly start: number;
  readonly end: number;
}

function utf8ByteLength(text: string): number {
  return textEncoder.encode(text).byteLength;
}

function closestElement(node: Node | null): Element | null {
  if (node instanceof Element) {
    return node;
  }
  return node?.parentElement ?? null;
}

function utf8ByteOffsetFromCodeUnitIndex(text: string, codeUnitIndex: number): number {
  const clampedIndex = Math.max(0, Math.min(codeUnitIndex, text.length));
  return utf8ByteLength(text.slice(0, clampedIndex));
}

function codeUnitIndexFromUtf8ByteOffset(text: string, byteOffset: number): number {
  const target = Math.max(0, Math.min(byteOffset, utf8ByteLength(text)));
  let currentIndex = 0;
  while (currentIndex < text.length) {
    const codePoint = text.codePointAt(currentIndex) ?? 0;
    const nextIndex = currentIndex + (codePoint > 0xFFFF ? 2 : 1);
    const nextByteOffset = utf8ByteOffsetFromCodeUnitIndex(text, nextIndex);
    if (nextByteOffset > target) {
      break;
    }
    currentIndex = nextIndex;
    if (nextByteOffset === target) {
      break;
    }
  }
  return currentIndex;
}

function readBoundaryCodeUnitOffset(fragment: HTMLElement, container: Node, offset: number): number | null {
  const range = document.createRange();
  range.selectNodeContents(fragment);
  try {
    range.setEnd(container, offset);
  } catch {
    return null;
  }
  return range.toString().length;
}

function ensureProjectedElement(
  layer: HTMLElement,
  byHandle: Map<string, HTMLDivElement>,
  canvasId: string,
  entry: FindOnPageDocument,
): HTMLDivElement {
  const existing = byHandle.get(entry.handle);
  if (existing !== undefined) {
    return existing;
  }

  const created = document.createElement('div');
  created.setAttribute('data-ed-find-fragment', '1');
  created.setAttribute('data-ed-canvas-id', canvasId);
  created.style.display = 'block';
  created.style.position = 'relative';
  created.style.whiteSpace = 'pre-wrap';
  created.style.wordBreak = 'break-word';
  created.style.pointerEvents = 'none';
  created.style.userSelect = 'text';
  layer.appendChild(created);
  byHandle.set(entry.handle, created);
  return created;
}

export class FindOnPageProjector {
  private readonly layer: HTMLDivElement;
  private readonly content: HTMLDivElement;
  private readonly canvasId: string;
  private readonly elementsByHandle = new Map<string, HTMLDivElement>();

  public constructor(canvas: HTMLCanvasElement, canvasId: string) {
    const parent = canvas.parentElement;
    if (!(parent instanceof HTMLElement)) {
      throw new Error('Expected scene shell for Find-on-Page projection.');
    }

    const layer = document.createElement('div');
    layer.id = `find-on-page-layer-${canvasId}`;
    layer.style.position = 'absolute';
    layer.style.left = '0';
    layer.style.top = '0';
    layer.style.pointerEvents = 'none';
    layer.style.opacity = '0';
    layer.style.overflow = 'hidden';
    layer.style.userSelect = 'text';
    layer.setAttribute('aria-hidden', 'true');
    layer.setAttribute('data-ed-find-root', '1');
    layer.setAttribute('data-ed-canvas-id', canvasId);

    const content = document.createElement('div');
    content.id = `find-on-page-content-${canvasId}`;
    content.style.position = 'absolute';
    content.style.left = '0';
    content.style.top = '0';
    content.style.width = '100%';
    content.style.minHeight = '100%';
    content.style.pointerEvents = 'none';
    content.style.userSelect = 'text';
    content.style.whiteSpace = 'pre-wrap';
    content.style.color = 'transparent';
    content.style.background = 'transparent';
    content.style.transformOrigin = '0 0';
    layer.appendChild(content);
    parent.appendChild(layer);

    this.layer = layer;
    this.content = content;
    this.canvasId = canvasId;
  }

  public syncSize(logicalWidth: number, logicalHeight: number): void {
    const width = `${String(logicalWidth)}px`;
    const height = `${String(logicalHeight)}px`;
    this.layer.style.width = width;
    this.layer.style.height = height;
  }

  public syncViewportTransform(scale: number, offsetX: number, offsetY: number): void {
    this.content.style.transform = scale === 1.0 && offsetX === 0.0 && offsetY === 0.0
      ? ''
      : `translate(${String(offsetX)}px, ${String(offsetY)}px) scale(${String(scale)})`;
  }

  public update(documents: readonly FindOnPageDocument[]): void {
    const seenHandles = new Set<string>();
    const orderedElements: HTMLDivElement[] = [];

    for (const entry of documents) {
      seenHandles.add(entry.handle);
      const element = ensureProjectedElement(this.content, this.elementsByHandle, this.canvasId, entry);
      element.setAttribute('data-ed-handle', entry.handle);
      element.setAttribute('data-ed-start', '0');
      element.setAttribute('data-ed-end', String(utf8ByteLength(entry.text)));
      if (element.textContent !== entry.text) {
        element.textContent = entry.text;
      }
      orderedElements.push(element);
    }

    syncOrderedChildren(this.content, orderedElements);
    for (const [handle] of this.elementsByHandle.entries()) {
      if (seenHandles.has(handle)) {
        continue;
      }
      this.elementsByHandle.delete(handle);
    }
  }

  public resolveSelection(selection: Selection | null): ResolvedFindSelection | null {
    if (
      selection === null ||
      selection.rangeCount === 0 ||
      selection.isCollapsed ||
      document.activeElement?.getAttribute('data-effindom-hidden-editor') === 'true'
    ) {
      return null;
    }

    const range = selection.getRangeAt(0);
    const startElement = closestElement(range.startContainer);
    const endElement = closestElement(range.endContainer);
    if (startElement === null || endElement === null) {
      return null;
    }
    if (
      startElement.closest('[data-effindom-hidden-editor="true"]') !== null ||
      endElement.closest('[data-effindom-hidden-editor="true"]') !== null
    ) {
      return null;
    }

    const startRoot = startElement.closest('[data-ed-find-root="1"]');
    const endRoot = endElement.closest('[data-ed-find-root="1"]');
    const startFragment = startElement.closest('[data-ed-find-fragment="1"]');
    const endFragment = endElement.closest('[data-ed-find-fragment="1"]');
    if (
      !(startRoot instanceof HTMLElement) ||
      !(endRoot instanceof HTMLElement) ||
      !(startFragment instanceof HTMLElement) ||
      !(endFragment instanceof HTMLElement) ||
      startRoot !== endRoot ||
      startFragment !== endFragment ||
      startRoot.dataset.edCanvasId !== this.canvasId ||
      startFragment.dataset.edCanvasId !== this.canvasId
    ) {
      return null;
    }

    const handle = startFragment.dataset.edHandle;
    const fragmentStart = Number(startFragment.dataset.edStart);
    const fragmentEnd = Number(startFragment.dataset.edEnd);
    if (
      handle === undefined ||
      !Number.isFinite(fragmentStart) ||
      !Number.isFinite(fragmentEnd) ||
      fragmentStart < 0 ||
      fragmentEnd < fragmentStart
    ) {
      return null;
    }

    const text = startFragment.textContent;
    const localStart = readBoundaryCodeUnitOffset(startFragment, range.startContainer, range.startOffset);
    const localEnd = readBoundaryCodeUnitOffset(startFragment, range.endContainer, range.endOffset);
    if (localStart === null || localEnd === null) {
      return null;
    }

    const start = fragmentStart + utf8ByteOffsetFromCodeUnitIndex(text, localStart);
    const end = fragmentStart + utf8ByteOffsetFromCodeUnitIndex(text, localEnd);
    if (start >= end || end > fragmentEnd) {
      return null;
    }

    return {
      handle,
      start,
      end,
    };
  }

  public selectMatch(match: ResolvedFindSelection | null): boolean {
    const selection = document.getSelection();
    if (selection === null) {
      return false;
    }
    if (match === null) {
      selection.removeAllRanges();
      return true;
    }

    const fragment = this.elementsByHandle.get(match.handle);
    if (!(fragment instanceof HTMLElement)) {
      return false;
    }

    const fragmentStart = Number(fragment.dataset.edStart);
    const fragmentEnd = Number(fragment.dataset.edEnd);
    if (
      !Number.isFinite(fragmentStart) ||
      !Number.isFinite(fragmentEnd) ||
      match.start < fragmentStart ||
      match.end > fragmentEnd ||
      match.start >= match.end
    ) {
      return false;
    }

    const text = fragment.textContent;
    const localStart = match.start - fragmentStart;
    const localEnd = match.end - fragmentStart;
    const startIndex = codeUnitIndexFromUtf8ByteOffset(text, localStart);
    const endIndex = codeUnitIndexFromUtf8ByteOffset(text, localEnd);
    const textNode = fragment.firstChild instanceof Text
      ? fragment.firstChild
      : document.createTextNode(text);
    if (fragment.firstChild !== textNode) {
      fragment.replaceChildren(textNode);
    }

    const range = document.createRange();
    range.setStart(textNode, startIndex);
    range.setEnd(textNode, endIndex);
    selection.removeAllRanges();
    selection.addRange(range);
    return true;
  }

  public destroy(): void {
    this.elementsByHandle.clear();
    this.layer.remove();
  }
}
