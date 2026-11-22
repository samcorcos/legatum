// The online host's save, through the real WASM engine: host save -> file text -> a new host from that save.
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { parseSaveFile } from '../saveErrors';
import { onlineSaveFile } from '../store';
import type { PlayerView } from '../types';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));

describe.skipIf(!available)('host save round trip (WASM)', () => {
  const load = async () => (await import('../wasm-pkg/legatum.js')) as unknown as {
    BrowserHost: new (save?: string | null) => { new_game(setup: string): void; save_game(): string; open_guest_seats(): void; status(local: boolean): string; free(): void };
    Game: { standard_setup(...args: string[]): string; load(save: string): { free(): void } };
  };

  it('a save file hosts again with the same game and seat names, and loads solo', async () => {
    const { BrowserHost, Game } = await load();
    const setup = JSON.parse(Game.standard_setup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
    setup.players = setup.players.map((p: { is_human?: boolean }, i: number) => ({ ...p, is_human: i < 2 }));
    const first = new BrowserHost(null);
    expect(() => first.save_game()).toThrow();
    first.new_game(JSON.stringify(setup));
    const game = first.save_game();
    const names = (host: { status(local: boolean): string }) => (JSON.parse(host.status(true)).seats as { name: string }[]).map((s) => s.name);

    // What the menu writes, and what the title screen reads back.
    const view = { turn: 1, players: [{ is_human: true, name: 'Ada' }, { is_human: true, name: 'Bo' }] } as unknown as PlayerView;
    const file = JSON.stringify(onlineSaveFile(game, [view]));
    const read = parseSaveFile(file);
    expect(read.online).toBe(true);
    expect(read.names).toEqual(['Ada', 'Bo']);
    expect(read.game).toBe(game);

    const second = new BrowserHost(read.game);
    expect(second.save_game()).toBe(game);
    expect(names(second)).toEqual(names(first));
    expect(names(second)).toEqual(['Ada', 'Bo']);
    const reserved = () => (JSON.parse(second.status(true)).seats as { reserved: boolean }[]).map((s) => s.reserved);
    expect(reserved()).toEqual([true, true]);
    second.open_guest_seats();
    expect(reserved()).toEqual([true, false]);
    // The same engine save is a normal solo game.
    Game.load(read.game).free();
    expect(() => new BrowserHost('not a save')).toThrow();
    first.free();
    second.free();
  });
});
