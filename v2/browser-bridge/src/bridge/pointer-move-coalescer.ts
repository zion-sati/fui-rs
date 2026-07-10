export class PointerMoveCoalescer<T> {
  private pendingMove: T | null = null;
  private frameScheduled = false;

  constructor(private readonly flushMove: (move: T) => void) {}

  enqueue(move: T): void {
    this.pendingMove = move;
    this.scheduleFlush();
  }

  takePending(): T | null {
    const pending = this.pendingMove;
    this.pendingMove = null;
    return pending;
  }

  clear(): void {
    this.pendingMove = null;
  }

  private hasPendingMove(): boolean {
    return this.pendingMove !== null;
  }

  private scheduleFlush(): void {
    if (this.frameScheduled) {
      return;
    }
    this.frameScheduled = true;
    requestAnimationFrame(() => {
      this.frameScheduled = false;
      const pending = this.pendingMove;
      this.pendingMove = null;
      if (pending === null) {
        return;
      }
      this.flushMove(pending);
      if (this.hasPendingMove()) {
        this.scheduleFlush();
      }
    });
  }
}
