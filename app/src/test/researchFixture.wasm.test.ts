import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { researchFixture } from './researchFixture';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let client: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  client = new WasmClient(mod.Game);
});

it.skipIf(!available)('the Byzantium research fixture shows an ordinary Library card , with city Science', () => {
  client.load(coreFixture(researchFixture('Byzantium')));
  const v = client.view(0);
  const lab = v.research.unlocks.find((u) => u.item.kind === 'Building' && u.item.building === 'Laboratory')!;
  expect(lab.name).toBe('Library');
  expect(lab.yields.science).toBeGreaterThan(0);
});

it.skipIf(!available)('faction fixtures show their own replacements', () => {
  client.load(coreFixture(researchFixture('Switzerland')));
  expect(client.view(0).research.unlocks.find((u) => u.item.kind === 'Building' && u.item.building === 'Walls')?.name).toBe('Alpine Redoubt');
  client.load(coreFixture(researchFixture('Athens')));
  expect(client.view(0).research.unlocks.find((u) => u.item.kind === 'Building' && u.item.building === 'Laboratory')?.name).toBe('Lyceum');
});
