import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';
import { newLobby, buildSetup } from '../lobby';
import { Settings } from './lobby/Settings';
import { RULES } from '../data';
import { Victory } from './LobbyScreen';

describe('game settings', () => {
  it('has no turn limit, barbarians or AI alliances setting; the game ends at the default last turn', () => {
    const lobby = newLobby('host', '42');
    const html = renderToStaticMarkup(createElement(Settings, { lobby, onChange: () => {} }));
    expect(buildSetup(lobby)).not.toHaveProperty('speed');
    for (const gone of ['Turn limit', 'Barbarians', 'AI alliances']) expect(html).not.toContain(`aria-label="${gone}"`);
    expect(renderToStaticMarkup(createElement(Victory, { lobby }))).toContain(`the most VP at turn ${RULES.defaultTurnLimit}.`);
  });

  it('keeps coach questions and the map seed in a closed Advanced section', () => {
    const html = renderToStaticMarkup(createElement(Settings, { lobby: newLobby('solo', '42'), onChange: () => {} }));
    const adv = html.slice(html.indexOf('<details class="lobby-advanced"'));
    expect(adv).toMatch(/^<details class="lobby-advanced">/);
    expect(adv).toContain('aria-label="Coach questions"');
    expect(adv).toContain('aria-label="Map seed"');
  });

  it('opens Advanced when the map seed needs fixing', () => {
    const html = renderToStaticMarkup(createElement(Settings, { lobby: { ...newLobby('solo', '42'), seed: 'abc' }, onChange: () => {} }));
    expect(html).toContain('<details class="lobby-advanced" open="">');
  });

  it('hides Advanced from a viewer who cannot change it', () => {
    const html = renderToStaticMarkup(createElement(Settings, { lobby: newLobby('host', '42') }));
    expect(html).not.toContain('lobby-advanced');
  });
});
