import { buildField, worldFromArrays } from './world';
import { sculptRelief } from './flora';

interface FieldRequest { id: number; w: number; h: number; cls: Int8Array; fog: Uint8Array }

self.onmessage = ({ data }: MessageEvent<FieldRequest>) => {
  const start = performance.now();
  const world = worldFromArrays(data.w, data.h, data.cls, data.fog);
  const base = buildField(world);
  const field = { ...base, pos: base.pos.slice(), col: base.col.slice(), height: base.height.slice() };
  sculptRelief(world, field);
  const workerMs = performance.now() - start;
  self.postMessage({ id: data.id, field, base, workerMs }, { transfer: [field.pos.buffer, field.col.buffer, field.fog.buffer, field.shore.buffer, field.height.buffer, field.materials.buffer, base.pos.buffer, base.col.buffer, base.height.buffer] });
};
