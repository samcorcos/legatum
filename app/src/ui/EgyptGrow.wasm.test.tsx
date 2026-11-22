// @vitest-environment happy-dom
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath, URL as NodeURL } from 'node:url';
import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { beforeAll, expect, it, vi } from 'vitest';
import { WasmClient } from '../wasmClient';
import { useGame } from '../store';
import { CityPanel } from './CityPanel';

const available = existsSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum.js', import.meta.url)));
let makeClient: () => WasmClient;

beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js';
  const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeClient = () => new WasmClient(mod.Game);
});

// The Nile: an Egyptian city that builds doesn't grow, so its panel offers Grow, and its prices read in Food.
it.skipIf(!available)('offers Egypt a Grow button that sets the Grow project, and prices builds in Food', async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  const client = makeClient();
  const setup = await client.standardSetup('42', 'Egypt', 'Rome', 'Ada', 'Bo');
  setup.players = setup.players.slice(0, 2); setup.players[1].team = 1;
  await client.newGame(setup);
  const view = client.view(0);
  const city = view.cities.find((c) => c.owner === 0)!;
  useGame.setState({ client, humans: [0], views: [view], seat: 0, orders: [[]], errors: [[]], ready: [false] });
  const container = document.body.appendChild(document.createElement('div'));
  const root = createRoot(container);
  try {
    act(() => root.render(<CityPanel city={city} view={view} />));
    const grow = container.querySelector<HTMLButtonElement>('button[aria-label="Run Grow"]');
    expect(grow, container.textContent ?? '').not.toBeNull();
    expect(grow!.textContent).toMatch(/Grows in \d+ turns?|No surplus Food/);
    expect(container.textContent).toContain('Growth');
    expect(container.textContent).not.toContain('Industry projects');
    act(() => grow!.click());
    expect(useGame.getState().orders[0]).toContainEqual(expect.objectContaining({ city: city.id }));
    expect(JSON.stringify(useGame.getState().orders[0])).toContain('"Grow"');
    // Build cards price in Food, not the Production gear.
    const costLabels = [...container.querySelectorAll('.prod-cost [aria-label]')].map((e) => e.getAttribute('aria-label'));
    expect(costLabels.length).toBeGreaterThan(0);
    expect(costLabels).not.toContain('Production');
  } finally {
    act(() => root.unmount());
    container.remove();
  }
});
