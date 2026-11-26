/** Timing data for the map rebuild. Enabled in development only. */
export type RebuildProfile = Record<string, number>;

export class RebuildTimer {
  private start = performance.now();
  readonly spans: RebuildProfile = {};

  mark(name: string): void {
    const now = performance.now();
    this.spans[name] = Math.round((now - this.start) * 10) / 10;
    this.start = now;
  }
}
