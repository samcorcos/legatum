import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { warDeclaredCoreSave } from './warDeclaredFixture';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let client: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  client = new WasmClient(mod.Game);
});

it.skipIf(!available).each([false, true])('the real Sparta AI declares war on our team in turn 1 (teammate: %s)', (teammate) => {
  client.load(warDeclaredCoreSave({ teammate }));
  const humans = teammate ? [0, 1] : [0];
  const result = client.endTurn(Object.fromEntries(humans.map((h) => [h, []])));
  for (const me of humans) {
    const seen = result.events_by_player?.[me] ?? [];
    const war = seen.find((e) => e.type === 'WarDeclared');
    expect(war, JSON.stringify(seen.map((e) => e.type))).toBeDefined();
    if (war?.type !== 'WarDeclared') return;
    expect(war.relation.declarers).toEqual([1]);
    expect([war.relation.a, war.relation.b]).toContain(0);
    // A surprise war at plain Peace: in effect in the turn it was declared.
    expect(war.relation.effective_turn).toBe(1);
    // The viewer can see where the threat is: their army and their city.
    expect(client.view(me).units.filter((u) => u.owner === (teammate ? 2 : 1) && u.kind === 'Trooper').length).toBeGreaterThan(0);
    expect(client.view(me).cities.some((c) => c.name === 'Oathhold')).toBe(true);
    expect(client.view(me).diplomacy.relations.find((r) => [r.a, r.b].includes(1) && [r.a, r.b].includes(0))?.state).toBe('War');
  }
});
