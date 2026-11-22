import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';
import { newLobby, buildSetup } from '../lobby';
import { Settings } from './lobby/Settings';
import { Victory } from './LobbyScreen';

describe('game settings', () => {
  it('offers no turn limit by default', () => {
    const lobby = newLobby('solo', '42');
    const html = renderToStaticMarkup(createElement(Settings, { lobby, onChange: () => {} }));
    expect(buildSetup(lobby)).not.toHaveProperty('speed');
    expect(html).toMatch(/aria-checked="true"[^>]*><span>Off<\/span>/);
    expect(renderToStaticMarkup(createElement(Victory, { lobby }))).toContain('No turn limit.');
  });

  it('keeps explicit numeric turn limits', () => {
    const lobby = { ...newLobby('host', '42'), turnLimit: 333 };
    const html = renderToStaticMarkup(createElement(Settings, { lobby }));
    expect(html).toMatch(/aria-checked="true"[^>]*><span>On<\/span>/);
    expect(html).toContain('value="333"');
    expect(html).toContain('readOnly=""');
    expect(renderToStaticMarkup(createElement(Victory, { lobby }))).toContain('most VP at turn 333');
  });
});
