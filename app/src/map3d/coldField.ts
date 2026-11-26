import { buildField, gridFor, type Field, type World } from './world';
import { sculptRelief } from './flora';

function buildCold(world: World): { field: Field; base: Field } {
  const base = buildField(world, gridFor(world));
  const field = { ...base, pos: base.pos.slice(), col: base.col.slice(), height: base.height.slice() };
  sculptRelief(world, field);
  return { field, base };
}

interface FieldResult { id: number; field: Field; base: Field; workerMs: number }
interface Pending { id: number; start: number; resolve: (ms: number) => void }

/** Runs a genuine cold field build off the main thread and discards superseded results. */
export class ColdField {
  private worker: Worker | null = null;
  private serial = 0;
  private current: Pending | null = null;
  private world: World | null = null;
  private ready: Promise<number> = Promise.resolve(0);

  constructor(private apply: (field: Field, base: Field, workerMs: number, complete: () => number | null) => void) {
    try {
      this.worker = new Worker(new URL('./fieldWorker.ts', import.meta.url), { type: 'module' });
      this.worker.onmessage = ({ data }: MessageEvent<FieldResult>) => {
        if (data.id === this.current?.id) this.deliver(data.field, data.base, data.workerMs);
      };
      this.worker.onerror = () => {
        this.worker?.terminate(); this.worker = null;
        if (this.current && this.world) { const { field, base } = buildCold(this.world); this.deliver(field, base, 0); }
      };
    } catch { this.worker = null; }
  }

  get pending(): boolean { return this.current !== null; }
  whenReady(): Promise<number> { return this.ready; }

  cancel(): void {
    this.serial++;
    this.current?.resolve(0);
    this.current = null;
    this.world = null;
  }

  dispose(): void { this.cancel(); this.worker?.terminate(); this.worker = null; }

  request(world: World): void {
    this.cancel();
    const id = ++this.serial;
    this.world = world;
    this.ready = new Promise((resolve) => { this.current = { id, start: performance.now(), resolve }; });
    if (this.worker) {
      const cls = world.cls.slice(), fog = world.fog.slice();
      try {
        this.worker.postMessage({ id, w: world.w, h: world.h, cls, fog }, [cls.buffer, fog.buffer]);
        return;
      } catch { this.worker.terminate(); this.worker = null; }
    }
    const { field, base } = buildCold(world); this.deliver(field, base, 0);
  }

  private deliver(field: Field, base: Field, workerMs: number): void {
    const current = this.current;
    if (!current) return;
    this.apply(field, base, workerMs, () => {
      if (this.current !== current) return null;
      const wallMs = Math.round((performance.now() - current.start) * 10) / 10;
      this.current = null;
      this.world = null;
      current.resolve(wallMs);
      return wallMs;
    });
  }
}
