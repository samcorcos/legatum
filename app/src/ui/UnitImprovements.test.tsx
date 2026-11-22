// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { tileIndex } from '../hex';
import { UnitImprovements, upgradeCheck } from './UnitImprovements';

it('shows the queued-spend balance for both land and Siege upgrades', async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const view = client.view(0);
  const unit = view.units.find((u) => u.owner === view.me && u.kind === 'Spearman')!;
  unit.siege_offer = { price: { total: 50, parts: [['Siege', 50]] }, blocker: null };
  const container = document.body.appendChild(document.createElement('div'));
  const root = createRoot(container);
  try {
    act(() => root.render(<UnitImprovements unit={unit} view={view} gold={40} siegeGold={30}
      check={{ to: 'Phalangite', cost: 60, needs: [{ label: '60 Gold', met: false }], blocker: 'Needs 60 Gold (you have 40)', need: null }}
      siegeBlocker="Needs 50 Gold (you have 30)" primary={null} />));
    const buttons = [...container.querySelectorAll<HTMLButtonElement>('.improve-row button.act')];
    expect(buttons).toHaveLength(2);
    expect(buttons.every((button) => button.disabled)).toBe(true);
    expect(container.textContent).toContain('Needs 60 Gold (you have 40)');
    expect(container.textContent).toContain('Needs 50 Gold (you have 30)');
  } finally {
    act(() => root.unmount());
    container.remove();
    vi.unstubAllGlobals();
  }
});

it('includes supply reserved by an earlier upgrade in the requirement list', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const view = client.view(0);
  const unit = view.units.find((u) => u.owner === view.me && u.kind === 'Spearman')!;
  unit.upgrade_need = 'Iron';
  unit.upgrade_breakdown = { total: 60, parts: [['Upgrade', 60]] };
  view.research.known.push('Mathematics');
  view.tiles[tileIndex(unit.pos, view.width, view.height)].owner = view.me;
  view.supply = [{ resource: 'Iron', spare: 1 } as never];
  const check = upgradeCheck(unit, view, 100, new Map([['Iron', 1]]));
  expect(check?.blocker).toBe('No Iron to spare after your queued orders.');
  expect(check?.needs.find((need) => need.label.includes('Iron supply'))?.met).toBe(false);
});
