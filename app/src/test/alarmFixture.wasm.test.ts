import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { alarmHeadline, alarmsForTurn } from '../celebrate/alarms';
import { ALARM_CITY, ALARM_EVENTS, ALARM_TURN, alarmCoreSave } from './alarmFixture';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let client: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  client = new WasmClient(mod.Game);
});

it.skipIf(!available)('the alarm fixture loads in core, and each event set raises its one lighter alarm', () => {
  client.load(alarmCoreSave());
  const view = client.view(0);
  expect(view.turn).toBe(ALARM_TURN);
  // The lost city is theirs now and in sight; the ally and the treaty breaker are known empires.
  expect(view.cities.find((c) => c.id === ALARM_CITY.lost)).toMatchObject({ name: 'Brasslock', owner: 2 });
  expect(view.diplomacy.relations.find((r) => r.a === 0 && r.b === 1)?.state).toBe('Alliance');
  const headline = (kind: keyof typeof ALARM_EVENTS) => {
    const alarms = alarmsForTurn(ALARM_EVENTS[kind], view, view);
    expect(alarms.map((a) => a.kind)).toEqual([kind]);
    return alarmHeadline(view, alarms);
  };
  expect(headline('breach')).toBe('The Swiss break their Alliance with you');
  // The betrayal is public: core keeps the record the fixture saved.
  expect(view.diplomacy.betrayals).toEqual([{ by: 3, against: 0, turn: ALARM_TURN, until: ALARM_TURN + 20 }]);
  expect(headline('cityLost')).toBe('Brasslock has fallen');
  expect(headline('allyWar')).toBe('The Spartans attacked the Indians, your ally, and you are at War with them too');
});
