import { cloneSemanticTree, HiddenDomProjector, parseSemanticBuffer } from '../../semantic';
import type { SemanticNode, UiModule } from '../../core-types';
import type { BridgeInteractionState } from '../local-types';
import { extractSemanticBuffer } from '../utils/heap';
import type { TextDocumentController } from './text-documents';
import type { DebugTreeSnapshot } from '../../debug-tree';
import { buildSemanticLightDomFields } from './editable-form-model';
import type { BridgePlatformHost } from '../host/platform-host';

const SEMANTIC_ANNOUNCEMENT_DELAY_MS = 50;
const SEMANTIC_SCROLL_IDLE_DELAY_MS = 80;

export class SemanticController {
  private readonly projector: HiddenDomProjector;
  private semanticTree: SemanticNode[] = [];
  private semanticTextLayoutsByHandle: Record<string, {
    readonly bounds: SemanticNode['bounds'];
  } | undefined> = {};
  private semanticAnnouncementTimer: number | null = null;
  private scheduledSemanticAnnouncementHandle: string | null = null;
  private semanticProjectionTimer: number | null = null;
  private semanticProjectionDeferred = false;

  public constructor(
    canvas: HTMLCanvasElement,
    private readonly ui: UiModule,
    private readonly interactionState: BridgeInteractionState,
    private readonly textDocuments: TextDocumentController,
    private readonly host: BridgePlatformHost,
    private readonly getDebugTree: () => DebugTreeSnapshot,
    private readonly getTextInputMetadata: (handle: string) => { readonly kind: 'text' | 'password' | 'email'; readonly hostAutofillHint: string | null } | null,
  ) {
    this.projector = new HiddenDomProjector(canvas);
  }

  public syncSize(logicalWidth: number, logicalHeight: number): void {
    this.projector.syncSize(logicalWidth, logicalHeight);
  }

  public syncViewportTransform(scale: number, offsetX: number, offsetY: number): void {
    this.projector.syncViewportTransform(scale, offsetX, offsetY);
  }

  public syncSemanticState(): void {
    this.semanticTree = parseSemanticBuffer(extractSemanticBuffer(this.ui));
    this.interactionState.reconcileLiveHandles(this.semanticTree.map((node) => node.handle));
    this.semanticTextLayoutsByHandle = this.buildSemanticTextLayouts();
    this.host.publishSemanticTree(cloneSemanticTree(this.semanticTree));
    if (this.semanticProjectionDeferred) {
      this.scheduleDeferredProjection();
      return;
    }
    this.projectCurrentSemanticState();
  }

  public deferProjectionUntilScrollIdle(): void {
    this.semanticProjectionDeferred = true;
    this.scheduleDeferredProjection();
  }

  private projectCurrentSemanticState(): void {
    const debugTree = this.getDebugTree();
    const semanticLightDomFields = buildSemanticLightDomFields(
      debugTree,
      this.semanticTree,
      this.interactionState.textByHandle,
      this.interactionState.textRevisionsByHandle,
      this.getTextInputMetadata,
    );
    const omittedHandles = new Set(semanticLightDomFields.map((field) => field.handle));
    const liveHandles = new Set(Object.keys(debugTree.nodesByHandle));
    this.projector.update(
      this.semanticTree,
      this.interactionState.textByHandle,
      this.interactionState.textRevisionsByHandle,
      this.semanticTextLayoutsByHandle,
      omittedHandles,
      liveHandles,
    );
    this.projector.updateLightDomSemanticForms(
      semanticLightDomFields,
      (handle, editor) => {
        this.interactionState.registerSemanticTextEditor(handle, editor);
      },
      liveHandles,
    );
    this.interactionState.syncActiveTextInputViewport();
    const focusedHandle = this.interactionState.getFocusedHandle();
    for (const handle of this.interactionState.consumePendingSemanticAnnouncements()) {
      if (focusedHandle !== null && focusedHandle === handle) {
        this.scheduleSemanticAnnouncement(handle);
      }
    }
  }

  private scheduleDeferredProjection(): void {
    if (this.semanticProjectionTimer !== null) {
      this.host.clearTimer(this.semanticProjectionTimer);
    }
    this.semanticProjectionTimer = this.host.setTimer(() => {
      this.semanticProjectionTimer = null;
      this.semanticProjectionDeferred = false;
      this.projectCurrentSemanticState();
    }, SEMANTIC_SCROLL_IDLE_DELAY_MS);
  }

  public getSemanticTree(): readonly SemanticNode[] {
    return this.semanticTree;
  }

  public getBoundingBox(handle: string): SemanticNode['bounds'] | null {
    const node = this.semanticTree.find((entry) => entry.handle === handle);
    return node === undefined ? null : { ...node.bounds };
  }

  public destroy(): void {
    this.cancelPendingSemanticAnnouncement();
    if (this.semanticProjectionTimer !== null) {
      this.host.clearTimer(this.semanticProjectionTimer);
      this.semanticProjectionTimer = null;
    }
    this.semanticTree = [];
    this.semanticTextLayoutsByHandle = {};
    this.host.publishSemanticTree([]);
    this.projector.destroy();
  }

  private cancelPendingSemanticAnnouncement(): void {
    if (this.semanticAnnouncementTimer !== null) {
      this.host.clearTimer(this.semanticAnnouncementTimer);
      this.semanticAnnouncementTimer = null;
    }
    this.scheduledSemanticAnnouncementHandle = null;
  }

  private scheduleSemanticAnnouncement(handle: string): void {
    this.scheduledSemanticAnnouncementHandle = handle;
    if (this.semanticAnnouncementTimer !== null) {
      this.host.clearTimer(this.semanticAnnouncementTimer);
    }
    this.semanticAnnouncementTimer = this.host.setTimer(() => {
      this.semanticAnnouncementTimer = null;
      const targetHandle = this.scheduledSemanticAnnouncementHandle;
      this.scheduledSemanticAnnouncementHandle = null;
      if (targetHandle === null || this.interactionState.getFocusedHandle() !== targetHandle) {
        return;
      }
      this.projector.announceNode(targetHandle, this.semanticTree, this.interactionState.textByHandle);
    }, SEMANTIC_ANNOUNCEMENT_DELAY_MS);
  }

  private buildSemanticTextLayouts(): Record<string, {
    readonly bounds: SemanticNode['bounds'];
  } | undefined> {
    const layouts = Object.create(null) as Record<string, {
      readonly bounds: SemanticNode['bounds'];
    } | undefined>;
    for (const node of this.semanticTree) {
      if (node.label.length === 0) {
        continue;
      }
      const visibleTextBounds = this.textDocuments.readVisibleTextBounds(node.handle);
      if (visibleTextBounds === null) {
        continue;
      }
      const left = Math.max(node.bounds.x, visibleTextBounds.x);
      const top = Math.max(node.bounds.y, visibleTextBounds.y);
      const right = Math.min(node.bounds.x + node.bounds.width, visibleTextBounds.x + visibleTextBounds.width);
      const bottom = Math.min(node.bounds.y + node.bounds.height, visibleTextBounds.y + visibleTextBounds.height);
      if (right <= left || bottom <= top) {
        continue;
      }
      layouts[node.handle] = {
        bounds: {
          x: left,
          y: top,
          width: right - left,
          height: bottom - top,
        },
      };
    }
    return layouts;
  }
}
