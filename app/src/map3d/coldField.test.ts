import { expect, it, vi } from 'vitest';
import type { PlayerView } from '../types';
import { ColdField } from './coldField';
import { buildField, worldFrom } from './world';

it('ignores a stale worker result after a newer fogged view is requested', async () => {
  class FakeWorker {
    static instance: FakeWorker;
    onmessage: ((event: MessageEvent) => void) | null = null;
    onerror: (() => void) | null = null;
    messages: { id: number }[] = [];
    constructor() { FakeWorker.instance = this; }
    postMessage(data: { id: number }) { this.messages.push(data); }
    terminate() {}
  }
  vi.stubGlobal('Worker', FakeWorker);
  const tiles = (terrain: string) => Array.from({ length: 12 }, () => ({ explored: true, visible: true, terrain, owner: null }));
  const first = worldFrom({ width: 4, height: 3, tiles: tiles('Grassland') } as unknown as PlayerView);
  const latest = worldFrom({ width: 4, height: 3, tiles: tiles('Mountains') } as unknown as PlayerView);
  const applied = [] as ReturnType<typeof buildField>[];
  const cold = new ColdField((field, _base, _workerMs, complete) => { applied.push(field); complete(); });
  try {
    cold.request(first);
    const oldReady = cold.whenReady();
    cold.request(latest);
    const newReady = cold.whenReady();
    const [oldMessage, newMessage] = FakeWorker.instance.messages;
    FakeWorker.instance.onmessage?.({ data: { id: oldMessage.id, field: buildField(first), base: buildField(first), workerMs: 1 } } as MessageEvent);
    expect(applied).toHaveLength(0);
    FakeWorker.instance.onmessage?.({ data: { id: newMessage.id, field: buildField(latest), base: buildField(latest), workerMs: 1 } } as MessageEvent);
    expect(applied).toEqual([buildField(latest)]);
    await expect(oldReady).resolves.toBe(0);
    await expect(newReady).resolves.toBeGreaterThanOrEqual(0);
  } finally {
    cold.dispose();
    vi.unstubAllGlobals();
  }
});
