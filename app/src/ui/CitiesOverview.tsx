// The Cities overview (user, 2026-10-17): "which cities have the highest population... which has the highest
// production". One screen for every empire-wide question about your cities: a table with a sticky header, sortable by
// any column, a row per city, and a click on a row takes you to that city (the map centres on it and its panel
// opens). Every number is the one the city's own panel shows (`citiesOverview.ts` reads it from the view).
import { useEffect, useMemo, useRef, useState } from 'react';
import { useGame } from '../store';
import { Star } from '../brand/ornaments';
import { YieldIcon, Icon, signed } from './bits';
import { ItemArt } from './ItemArt';
import { closeTips, Tip } from './Tip';
import type { CityBuildFocus, Faction } from '../types';
import { FOCUS_CYCLE, focusLabel, setCityAutomation } from '../automation';
import { cityRows, DEFAULT_CITY_SORT, nextSort, sortRows, type CityRow, type CitySort, type CitySortKey } from '../citiesOverview';
import { TOMB_CITY_NOTE, tombOnAfterOrders, tombShort } from '../goalCities';
import './citiesOverview.css';

const SORT_KEY = 'legatum.cities-sort.v1';
const KEYS: CitySortKey[] = ['name', 'population', 'food', 'production', 'gold', 'science', 'defence', 'build'];

function loadSort(): CitySort {
  try {
    const raw = JSON.parse(localStorage.getItem(SORT_KEY) ?? 'null') as CitySort | null;
    if (raw && KEYS.includes(raw.key) && (raw.dir === 'asc' || raw.dir === 'desc')) return raw;
  } catch { /* storage blocked: the default stands */ }
  return DEFAULT_CITY_SORT;
}
function saveSort(sort: CitySort) {
  try { localStorage.setItem(SORT_KEY, JSON.stringify(sort)); } catch { /* storage blocked */ }
}

const COLUMNS: { key: CitySortKey; label: string; yield?: 'food' | 'production' | 'gold' | 'science'; hint: string }[] = [
  { key: 'name', label: 'City', hint: 'Sort by name' },
  { key: 'population', label: 'Population', hint: 'Sort by population, with the turns to the next growth' },
  { key: 'food', label: 'Food', yield: 'food', hint: 'Sort by Food' },
  { key: 'production', label: 'Production', yield: 'production', hint: 'Sort by Production' },
  { key: 'gold', label: 'Gold', yield: 'gold', hint: 'Sort by Gold' },
  { key: 'science', label: 'Science', yield: 'science', hint: 'Sort by Science' },
  { key: 'defence', label: 'Strength', hint: "Sort by city strength. Its Walls are named under it" },
  { key: 'build', label: 'Building', hint: 'Sort by what needs you most: idle cities first, then the nearest to finishing' },
];

function BuildCell({ row, faction, tomb }: { row: CityRow; faction?: Faction; tomb?: boolean }) {
  // Egypt's Royal Tomb work stops every city: the row says so rather than "Idle".
  if (tomb) return <Tip content={TOMB_CITY_NOTE}><span className="co-building co-tomb" data-testid="city-tomb"><span className="co-build-top"><Icon name="goal-royal-tomb" size={14} /><span className="co-build-name">Royal Tomb</span></span><span className="co-sub">{row.build ? `${row.build.name} waits` : 'Nothing queued'}</span></span></Tip>;
  if (row.focus && !row.build) return <span className="co-building" data-testid="city-auto-wait">Choosing at Ready<span className="co-sub">{focusLabel(row.focus, faction)}</span></span>;
  if (row.idle || !row.build) {
    return <span className="co-idle" data-testid="city-idle"><span className="co-idle-mark" aria-hidden="true">!</span><b>Idle</b><span className="co-sub">choose what it builds</span></span>;
  }
  const b = row.build;
  const left = b.state === 'waiting' ? 'waiting' : b.turns === null ? 'stalled' : `${b.turns} turn${b.turns === 1 ? '' : 's'}`;
  const f = b.cost > 0 ? Math.max(0, Math.min(1, b.stored / b.cost)) : 0;
  return (
    <span className="co-building" data-testid="city-build">
      <span className="co-build-top">
        <ItemArt item={b.item} faction={faction} size={16} />
        <span className="co-build-name">{b.name}</span>
        <b className="co-turns">{left}</b>
        {b.pending && <span className="co-pending">pending</span>}
      </span>
      <span className="co-bar" role="progressbar" aria-label={`${b.name} progress`} aria-valuemin={0} aria-valuemax={b.cost} aria-valuenow={b.stored}><i style={{ width: `${f * 100}%` }} /></span>
      <span className="co-sub">{b.stored}/{b.cost} Production</span>
    </span>
  );
}

export function CitiesOverview() {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const orders = useGame((s) => s.orders[s.seat]);
  const open = useGame((s) => s.citiesOpen);
  const set = useGame((s) => s.set);
  const [sort, setSort] = useState<CitySort>(loadSort);
  const [selected, setSelected] = useState<number[]>([]);
  const [focus, setFocus] = useState<CityBuildFocus>('Economy');
  const editOrders = useGame((s) => s.editOrders);
  const dialog = useRef<HTMLDialogElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const shown = open && !!view;
  const rows = useMemo(() => (view ? sortRows(cityRows(view, orders ?? []), sort) : []), [view, orders, sort]);
  useEffect(() => setSelected([]), [view?.me]);
  const chosen = selected.filter((id) => rows.some((row) => row.id === id));
  useEffect(() => {
    const el = dialog.current;
    if (!el || !shown) return;
    closeTips();
    if (!el.open) el.showModal();
    closeButton.current?.focus();
    return () => { if (el.open) el.close(); };
  }, [shown]);
  if (!shown || !view) return null;
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  const close = () => set({ citiesOpen: false });
  const tomb = tombOnAfterOrders(view, orders ?? []);
  const tombNow = view.actions?.tomb;
  const idle = tomb ? 0 : rows.filter((r) => r.idle).length;
  const sortBy = (key: CitySortKey) => { const next = nextSort(sort, key); setSort(next); saveSort(next); };
  const go = (row: CityRow) => {
    set({ citiesOpen: false });
    useGame.getState().select({ kind: 'city', id: row.id }, true);
    requestAnimationFrame(() => document.querySelector<HTMLElement>('[data-game-map]')?.focus({ preventScroll: true }));
  };
  const apply = (next: CityBuildFocus | null) => editOrders((list) => chosen.reduce((orders, id) => setCityAutomation(orders, id, next), list));
  return (
    <dialog ref={dialog} className="co-sheet paper-grain" aria-labelledby="co-title" data-dismissible="cities" data-testid="cities-overview"
      onCancel={(e) => { e.preventDefault(); close(); }}
      onPointerDown={(e) => {
        if (e.target !== e.currentTarget) return;
        const r = e.currentTarget.getBoundingClientRect();
        if (e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom) close();
      }}
      onKeyDown={(e) => {
        // Keep game shortcuts out of the sheet, but retain native control keys; Tab loops inside it.
        e.stopPropagation();
        if (e.key === 'Escape' || ((e.key === 'c' || e.key === 'C') && !e.metaKey && !e.ctrlKey && !e.altKey)) { e.preventDefault(); close(); return; }
        if (e.key !== 'Tab') return;
        const stops = [...e.currentTarget.querySelectorAll<HTMLElement>('button:not(:disabled)')];
        const first = stops[0], last = stops.at(-1);
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
      }}>
      <header className="co-head">
        <div>
          <span className="co-kick"><Star size={8} /> Your empire · turn {view.turn}</span>
          <h2 id="co-title">Cities</h2>
        </div>
        <p className="co-count" data-testid="cities-count"><b>{rows.length}</b> {rows.length === 1 ? 'city' : 'cities'}{idle > 0 && <> · <b className="co-idle-count">{idle} idle</b></>}{tomb && tombNow && <> · <span className="co-tomb-count" data-testid="cities-tomb">{tombShort({ working: true, turns: tombNow.turns, target: tombNow.target, left: tombNow.target - tombNow.turns })}: no city builds or grows</span></>}</p>
        <p className="co-help">Click a heading to sort. Click a city to go to it.</p>
        <button ref={closeButton} className="btn quiet co-close" onClick={close} aria-label="Close cities">Close <kbd className="kbd">Esc</kbd></button>
      </header>
      <div className="co-automation" aria-label="Automate selected cities">
        <span>{chosen.length ? `${chosen.length} selected` : 'Select cities to automate'}</span>
        <select aria-label="Building focus for selected cities" value={focus} onChange={(e) => setFocus(e.target.value as CityBuildFocus)}>
          {FOCUS_CYCLE.map((f) => <option key={f} value={f}>{focusLabel(f, faction)}</option>)}
        </select>
        <button type="button" className={`btn${chosen.length ? ' primary' : ''}`} disabled={!chosen.length} onClick={() => apply(focus)}>Automate</button>
        <button type="button" className="btn" disabled={!chosen.length} onClick={() => apply(null)}>Stop automating</button>
      </div>
      <div className="co-scroll" tabIndex={-1}>
        {rows.length === 0 ? <p className="co-empty">You have no cities yet. Found one with a Settler.</p> : (
          <table className="co-table">
            <thead>
              <tr>
                <th className="co-select"><input type="checkbox" aria-label="Select all cities" checked={chosen.length === rows.length && rows.length > 0} onChange={(e) => setSelected(e.target.checked ? rows.map((r) => r.id) : [])} /></th>
                {COLUMNS.map((c) => {
                  const active = sort.key === c.key;
                  return (
                    <th key={c.key} scope="col" className={`co-${c.key}${active ? ' sorted' : ''}`} aria-sort={active ? (sort.dir === 'asc' ? 'ascending' : 'descending') : 'none'}>
                      <button type="button" className="co-sort" title={c.hint} data-sort={c.key} onClick={() => sortBy(c.key)}>
                        {c.yield && <YieldIcon k={c.yield} size={14} />}
                        <span>{c.label}</span>
                        <Icon name={active ? (sort.dir === 'asc' ? 'ui-chevron-up' : 'ui-chevron-down') : 'ui-chevron-down'} size={10} className={`co-arrow${active ? ' on' : ''}`} />
                      </button>
                    </th>
                  );
                })}
                <th className="co-go" aria-hidden="true" />
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.id} data-testid="city-row" data-city={row.id} className={row.idle && !tomb ? 'idle' : undefined} onClick={() => go(row)}>
                  <td className="co-select" onClick={(e) => e.stopPropagation()}><input type="checkbox" aria-label={`Select ${row.name}`} checked={selected.includes(row.id)} onChange={(e) => setSelected((ids) => e.target.checked ? [...ids, row.id] : ids.filter((id) => id !== row.id))} /></td>
                  <th scope="row" className="co-name">
                    <button type="button" className="co-city" onClick={(e) => { e.stopPropagation(); go(row); }} aria-label={`Go to ${row.name}`}>
                      {row.isCapital && <span className="co-capital" title="Capital"><Star size={9} /></span>}
                      {row.name}
                    </button>
                    {row.focus && <span className="co-auto-badge">{focusLabel(row.focus, faction)}</span>}
                  </th>
                  <td className="co-pop"><b>{row.population}</b><span className={`co-sub${row.growthBad ? ' bad' : ''}`}>{row.growthText}</span></td>
                  <td className="co-num"><span className={row.foodNet < 0 ? 'bad' : undefined}>{signed(row.foodNet)}</span><span className="co-sub" title={`${row.food} Food from tiles, ${row.food - row.foodNet} eaten by its population`}>{row.food} grown</span></td>
                  <td className="co-num">{row.production}</td>
                  <td className="co-num">{row.gold}</td>
                  <td className="co-num">{row.science}</td>
                  <td className="co-def"><Tip content={row.strengthRule}><span><b>{row.strength}</b><span className="co-sub">{row.fort ?? 'No Walls'}{row.hp < row.maxHp && <span className="co-hurt"> · HP {row.hp}/{row.maxHp}</span>}</span></span></Tip></td>
                  <td className="co-buildcell"><BuildCell row={row} faction={faction} tomb={tomb} /></td>
                  <td className="co-go" aria-hidden="true"><span className="co-chevron">›</span></td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </dialog>
  );
}

/** The top bar's entry: opens the overview. The shortcut is C. */
export function CitiesButton() {
  const open = useGame((s) => s.citiesOpen);
  const set = useGame((s) => s.set);
  return (
    <button className="tb-nav tb-nav-cities" data-testid="cities-button" aria-haspopup="dialog" aria-expanded={open} aria-keyshortcuts="C" aria-label="Cities"
      title="Every city of yours in one table: population, yields, defence and what each builds (C)" onClick={() => set({ citiesOpen: !open })}>
      <Icon name="city" size={15} /><span className="tb-label">Cities</span>
    </button>
  );
}
