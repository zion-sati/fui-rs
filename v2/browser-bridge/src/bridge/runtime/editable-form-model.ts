import type {
  OpenCanvasForm,
  OpenCanvasEditableTextKind,
} from '../../core-types';
import type { DebugTreeSnapshot } from '../../debug-tree';

export interface MinimalSemanticNode {
  readonly handle: string;
  readonly roleName: string;
  readonly label: string;
  readonly bounds: {
    readonly x: number;
    readonly y: number;
    readonly width: number;
    readonly height: number;
  };
  readonly state: {
    readonly multiline?: boolean;
    readonly readonly?: boolean;
    readonly disabled?: boolean;
  };
}

export interface TextInputMetadataRecord {
  readonly kind: 'text' | 'password' | 'email';
  readonly hostAutofillHint: string | null;
}

interface DebugTreeNodeLike {
  readonly nodeId: string;
  readonly parentHandle: string | null;
}

export interface SemanticLightDomField {
  readonly handle: string;
  readonly formHandle: string;
  readonly bounds: MinimalSemanticNode['bounds'];
  readonly multiline: boolean;
  readonly readOnly: boolean;
  readonly disabled: boolean;
  readonly kind: OpenCanvasEditableTextKind;
  readonly autofillHint: string;
  readonly semanticLabel: string;
  readonly stableFieldName: string | null;
  readonly text: string;
}

export function resolveStableFieldName(snapshot: DebugTreeSnapshot, handle: string): string | null {
  let currentHandle: string | null = handle;
  while (currentHandle !== null) {
    const node = snapshot.nodesByHandle[currentHandle] as DebugTreeNodeLike | undefined;
    if (node === undefined) {
      return null;
    }
    if (node.nodeId.length > 0) {
      return node.nodeId;
    }
    currentHandle = node.parentHandle;
  }
  return null;
}

export function resolveOwnNodeId(snapshot: DebugTreeSnapshot, handle: string): string | null {
  const node = snapshot.nodesByHandle[handle] as DebugTreeNodeLike | undefined;
  if (node === undefined || node.nodeId.length === 0) {
    return null;
  }
  return node.nodeId;
}

export function resolveTextInputMetadata(
  snapshot: DebugTreeSnapshot,
  handle: string,
  getMetadata: (candidateHandle: string) => TextInputMetadataRecord | null,
): TextInputMetadataRecord | null {
  let currentHandle: string | null = handle;
  while (currentHandle !== null) {
    const metadata = getMetadata(currentHandle);
    if (metadata !== null) {
      return metadata;
    }
    const node = snapshot.nodesByHandle[currentHandle] as DebugTreeNodeLike | undefined;
    if (node === undefined) {
      return null;
    }
    currentHandle = node.parentHandle;
  }
  return null;
}

export function resolveFormHandle(
  snapshot: DebugTreeSnapshot,
  semanticByHandle: ReadonlyMap<string, MinimalSemanticNode>,
  handle: string,
): string | null {
  let currentHandle: string | null = handle;
  while (currentHandle !== null) {
    const semantic = semanticByHandle.get(currentHandle);
    if (semantic?.roleName === 'form') {
      return currentHandle;
    }
    const node = snapshot.nodesByHandle[currentHandle] as DebugTreeNodeLike | undefined;
    if (node === undefined) {
      return null;
    }
    currentHandle = node.parentHandle;
  }
  const target = semanticByHandle.get(handle);
  if (target === undefined) {
    return null;
  }
  let bestHandle: string | null = null;
  let bestArea = Number.POSITIVE_INFINITY;
  for (const candidate of semanticByHandle.values()) {
    if (candidate.roleName !== 'form') {
      continue;
    }
    const contains =
      candidate.bounds.x <= target.bounds.x &&
      candidate.bounds.y <= target.bounds.y &&
      (candidate.bounds.x + candidate.bounds.width) >= (target.bounds.x + target.bounds.width) &&
      (candidate.bounds.y + candidate.bounds.height) >= (target.bounds.y + target.bounds.height);
    if (!contains) {
      continue;
    }
    const area = candidate.bounds.width * candidate.bounds.height;
    if (area < bestArea) {
      bestArea = area;
      bestHandle = candidate.handle;
    }
  }
  return bestHandle;
}

export function buildForms(
  snapshot: DebugTreeSnapshot,
  semanticTree: readonly MinimalSemanticNode[],
): OpenCanvasForm[] {
  const semanticByHandle = new Map(semanticTree.map((node) => [node.handle, node]));
  const fieldsByFormHandle = new Map<string, string[]>();
  for (const node of semanticTree) {
    if (node.roleName !== 'textbox') {
      continue;
    }
    const formHandle = resolveFormHandle(snapshot, semanticByHandle, node.handle);
    if (formHandle === null) {
      continue;
    }
    const fields = fieldsByFormHandle.get(formHandle);
    if (fields === undefined) {
      fieldsByFormHandle.set(formHandle, [node.handle]);
    } else {
      fields.push(node.handle);
    }
  }
  const forms: OpenCanvasForm[] = [];
  for (const node of semanticTree) {
    if (node.roleName !== 'form') {
      continue;
    }
    forms.push({
      handle: node.handle,
      stableName: resolveOwnNodeId(snapshot, node.handle),
      purpose: 'generic',
      fieldHandles: fieldsByFormHandle.get(node.handle) ?? [],
      submitHandle: null,
    });
  }
  return forms;
}

export function buildSemanticLightDomFields(
  snapshot: DebugTreeSnapshot,
  semanticTree: readonly MinimalSemanticNode[],
  textByHandle: Readonly<Record<string, string>>,
  getMetadata: (candidateHandle: string) => TextInputMetadataRecord | null,
): SemanticLightDomField[] {
  const semanticByHandle = new Map(semanticTree.map((node) => [node.handle, node]));
  const fields: SemanticLightDomField[] = [];
  for (const node of semanticTree) {
    if (node.roleName !== 'textbox') {
      continue;
    }
    const formHandle = resolveFormHandle(snapshot, semanticByHandle, node.handle);
    if (formHandle === null) {
      continue;
    }
    const metadata = resolveTextInputMetadata(snapshot, node.handle, getMetadata);
    if (metadata?.hostAutofillHint === null || metadata?.hostAutofillHint === undefined) {
      continue;
    }
    fields.push({
      handle: node.handle,
      formHandle,
      bounds: node.bounds,
      multiline: node.state.multiline === true,
      readOnly: node.state.readonly === true,
      disabled: node.state.disabled === true,
      kind: metadata.kind,
      autofillHint: metadata.hostAutofillHint,
      semanticLabel: node.label,
      stableFieldName: resolveStableFieldName(snapshot, node.handle),
      text: textByHandle[node.handle] ?? '',
    });
  }
  return fields;
}
