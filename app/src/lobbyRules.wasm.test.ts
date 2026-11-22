import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath, URL as NodeURL } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { WasmClient } from './wasmClient';
import { buildSetup, newLobby } from './lobby';

const available = existsSync(fileURLToPath(new NodeURL('./wasm-pkg/legatum.js', import.meta.url)));
let makeClient: () => WasmClient;

describe.skipIf(!available)('lobby rule experiments on the Rust engine', () => {
  beforeAll(async () => {
    const path = './wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new NodeURL('./wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeClient = () => new WasmClient(mod.Game);
  });

  it('a single-player game starts with the switch the lobby set, and with it off by default', async () => {
    const start = async (rules?: { objectives_deck?: boolean }) => {
      const client = makeClient();
      await client.newGame(buildSetup({ ...newLobby('solo', '42'), rule_experiments: rules }));
      return client.view(0).rule_experiments;
    };
    const off = await start();
    expect(off?.objectives_deck).toBeFalsy();
    expect(await start({ objectives_deck: true })).toMatchObject({ objectives_deck: true });
  });
});
