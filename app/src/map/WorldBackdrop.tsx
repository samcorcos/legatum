// A fully revealed, fixed world painted by the real map renderer, for the title and setup screens.
import { useEffect, useRef } from 'react';
import type { CityView, PlayerView, TileView } from '../types';
import { Game } from '../wasm-pkg/legatum.js';
import { distance, tileIndex, within } from '../hex';
import { MAP_STYLES } from './style';
import { TerrainCache } from './terrain';
import { HEX_SIZE, HEX_W, ROW_H } from './grid';

const W = 44;
const H = 28;

let world: PlayerView | null = null;
function backdropView(): PlayerView {
  if (world) return world;
  const setup = JSON.parse(Game.standard_setup('771934', 'India', 'India', 'P0', 'P1'));
  setup.width = W; setup.height = H;
  setup.players = setup.players.slice(0, 6).map((player: object, i: number) => ({ ...player, name: `P${i}`, faction: 'India', team: i, is_human: i === 0 }));
  const game = new Game(JSON.stringify(setup));
  const state = JSON.parse(game.save());
  game.free();
  const starts = Object.values(state.units as Record<string, { kind: string; owner: number; pos: { q: number; r: number } }>)
    .filter((unit) => unit.kind === 'Settler').sort((a, b) => a.owner - b.owner).map((unit) => unit.pos);
  const tiles: TileView[] = state.map.tiles.map((tile: { terrain: TileView['terrain'] }) => ({ explored: true, visible: true, terrain: tile.terrain, ruin: false, owner: null }));
  const cities: CityView[] = [];
  starts.forEach((pos, owner) => {
    const i = tileIndex(pos, W, H);
    if (i < 0) return;
    for (const h of within(pos, 2)) {
      const j = tileIndex(h, W, H);
      if (j < 0 || tiles[j].terrain === 'Ocean') continue;
      if (distance(h, pos) === 2 && (h.q * 7 + h.r * 3) % 4 === 0) continue;
      tiles[j].owner = owner;
    }
    cities.push({ id: owner, owner, name: '', pos, population: 4 + (owner % 3), hp: 200, max_hp: 200, strength: 20, is_capital: true, walls: false, great_works: [], visible: true, detail: null });
  });
  world = { width: W, height: H, tiles, cities, units: [], me: 0 } as unknown as PlayerView;
  return world;
}

export function WorldBackdrop() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const c = ref.current!;
    const ctx = c.getContext('2d')!;
    const view = backdropView();
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const cache = new TerrainCache(view, MAP_STYLES.atlas, dpr);
    let raf = 0;
    const paint = () => {
      const w = c.clientWidth;
      const h = c.clientHeight;
      if (c.width !== Math.round(w * dpr) || c.height !== Math.round(h * dpr)) {
        c.width = Math.round(w * dpr);
        c.height = Math.round(h * dpr);
      }
      const mapW = HEX_W * W;
      const mapH = ROW_H * H;
      const zoom = Math.max(w / (mapW * 0.92), h / (mapH * 0.92));
      const more = cache.draw(ctx, { x: mapW / 2 - HEX_SIZE, y: mapH / 2 - HEX_SIZE, zoom }, w, h, 40);
      if (more) raf = requestAnimationFrame(paint);
    };
    paint();
    const ro = new ResizeObserver(() => {
      cancelAnimationFrame(raf);
      paint();
    });
    ro.observe(c);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
    };
  }, []);
  return <canvas ref={ref} className="world-backdrop" />;
}
