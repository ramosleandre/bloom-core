// The session-bus wrapper and Shell extension share one shortcut ownership policy.
export class DesktopService {
  constructor({ capture, grab, emit, watch }) {
    this.capture = capture;
    this.grab = grab;
    this.emit = emit;
    this.watch = watch;
    this.owner = null;
    this.ungrab = null;
    this.unwatch = null;
  }

  pointer() {
    return this.capture();
  }

  setEnabled(sender, enabled) {
    if (this.owner && this.owner !== sender)
      throw new Error('Another Bloom instance already owns the shortcut.');
    if (!enabled) {
      this.release();
      return false;
    }
    if (this.owner === sender) return true;
    // Capture at activation, not registration: the pointer can move while Bloom is idle.
    this.ungrab = this.grab(() => this.emit(this.pointer()));
    this.owner = sender;
    this.unwatch = this.watch(sender, () => this.release());
    return true;
  }

  release() {
    this.ungrab?.();
    this.unwatch?.();
    this.owner = null;
    this.ungrab = null;
    this.unwatch = null;
  }
}
