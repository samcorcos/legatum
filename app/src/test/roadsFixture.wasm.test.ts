import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { expect, it } from 'vitest';
import { FixtureClient } from './fixtureEngine';
import { roadsFixture } from './roadsFixture';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));

// The Roadworks and road-art shots build on this fixture. Empires now start with their capital founded and no Settler,
// which once left it with no cities at all ("Cannot read properties of undefined (reading 'owner')").
it.skipIf(!available)('the roads fixture keeps every capital, adds its extra cities, and loads in core', () => {
  for (const opts of [{ hall: false, works: 'built' as const, players: 2, width: 40, height: 26, seed: '12', extra: 8 }, {}]) {
    const { fixture, hall } = roadsFixture(opts);
    expect(fixture.cities.length).toBeGreaterThan(opts.players ?? 4);
    expect(hall.owner).toBe(0);
    const client = new FixtureClient();
    client.load(JSON.stringify(fixture));
    expect(client.debugState().cities.length).toBe(fixture.cities.length);
  }
});
