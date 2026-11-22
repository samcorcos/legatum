import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { gameOverCoreSave } from './gameOverFixture';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let client: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  client = new WasmClient(mod.Game);
});

// The tour's game-over screens load these saves; a fixture core refuses shows up there only as "This save file is damaged".
it.skipIf(!available)('every game-over fixture loads in core', () => {
  for (const kind of ['victory', 'team-victory', 'rival-victory', 'defeat'] as const) {
    client.load(gameOverCoreSave(kind));
    expect(client.view(0).turn, kind).toBe(164);
  }
});
