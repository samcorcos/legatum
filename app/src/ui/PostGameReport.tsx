// The post-game report, Civ-style: the VP race over the whole game (the headline), each empire's cities,
// population, Science, Gold and military over time, a timeline of the landmark events, and a statistics table.
// Everything comes from the core's recorded history (`PlayerView.history`, only once the game is over).
import { useLayoutEffect, useMemo, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { Star } from '../brand/ornaments';
import { GROWTH_METRICS, METRICS, drawnLength, seriesOf, timelineByEra, type Metric, type PostGame, type ReportEmpire, type StatRow } from '../postgame';
import { playerColor } from '../theme';
import type { PlayerId, PlayerPublic, PlayerView } from '../types';
import { Amounts } from './Amounts';
import { Glyph, RingMedallion } from './ringMedallion';
import { HistoryChart, fmtInt, type ChartLine } from './HistoryChart';
import { PortraitFrame, PortraitRoundel, Ribbon } from './premium';
import type { RevealTone } from './GameReveal';
import { useGame } from '../store';
import { playTimeReportLine, playTimeSummary } from '../playTime';

type Tab = 'race' | 'growth' | 'timeline' | 'stats';
export const POSTGAME_ARMY_TEXT = 'highest army points reached';
const TABS: { id: Tab; label: string; icon: string }[] = [
  { id: 'race', label: 'Victory race', icon: 'vp' },
  { id: 'growth', label: 'Empires over time', icon: 'emblem-growth' },
  { id: 'timeline', label: 'Timeline', icon: 'emblem-milestone' },
  { id: 'stats', label: 'Statistics', icon: 'title-most-advanced' },
];

/** The chart's height from the space its panel actually has (so 1000x600 and 1440x900 both fill without scrolling). */
function useHeight(min = 200) {
  const ref = useRef<HTMLDivElement>(null);
  const [h, setH] = useState(320);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const measure = () => setH(Math.max(min, Math.floor(el.clientHeight)));
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, [min]);
  return [ref, h] as const;
}

function Legend({ pg, values, focus, onFocus, unit }: { pg: PostGame; values: (id: PlayerId) => number; focus: PlayerId | null; onFocus(id: PlayerId | null): void; unit: string }) {
  return (
    <div className="pg-legend" role="list" aria-label="Empires">
      {pg.empires.map((e) => (
        <button key={e.id} type="button" role="listitem" className={`pg-key ${e.mine ? 'mine' : ''} ${focus === e.id ? 'on' : ''} ${focus !== null && focus !== e.id ? 'dim' : ''}`}
          aria-pressed={focus === e.id} onClick={() => onFocus(focus === e.id ? null : e.id)} title={`${e.name}: ${fmtInt(values(e.id))} ${unit} at the end`}>
          <i style={{ background: e.ink }} />
          <PortraitRoundel faction={e.faction} size={20} color={playerColor(e.id)} fallen={!e.alive} />
          <span className="pg-key-name">{e.label}</span>
          <b>{fmtInt(values(e.id))}</b>
          {e.winner && <Star size={8} className="pg-key-star" />}
        </button>
      ))}
    </div>
  );
}

function lines(pg: PostGame, view: PlayerView, metric: Metric): ChartLine[] {
  const h = view.history!;
  return pg.empires.map((empire) => ({ empire, values: seriesOf(h, metric, empire.id), drawn: drawnLength(pg, h, empire.id) }));
}

function RaceTab({ pg, view, focus, onFocus }: { pg: PostGame; view: PlayerView; focus: PlayerId | null; onFocus(id: PlayerId | null): void }) {
  const [box, height] = useHeight(180);
  const data = useMemo(() => lines(pg, view, 'vp'), [pg, view]);
  const final = (id: PlayerId) => seriesOf(view.history!, 'vp', id).at(-1) ?? 0;
  return (
    <div className="pg-tab pg-race" role="tabpanel" aria-label="Victory race">
      <div className="pg-plot">
        <div className="pg-chart" ref={box}>
          <HistoryChart first={pg.firstTurn} last={pg.lastTurn} lines={data} eras={pg.eras} target={pg.target} step height={height}
            label="VP of every empire over the game" unit="VP" markers={pg.markers} focus={focus} onFocus={onFocus} />
        </div>
        <Legend pg={pg} values={final} focus={focus} onFocus={onFocus} unit="VP" />
      </div>
      <p className="pg-note">VP at the end of each turn. Bands are the eras, from the turn the first empire entered each one. Marks below: wars declared, cities captured. Point at the chart for every empire's VP on that turn; click an empire on the right to pick out its line.</p>
    </div>
  );
}

function GrowthTab({ pg, view, focus, onFocus }: { pg: PostGame; view: PlayerView; focus: PlayerId | null; onFocus(id: PlayerId | null): void }) {
  const [metric, setMetric] = useState<Metric>('cities');
  const [box, height] = useHeight(160);
  const data = useMemo(() => lines(pg, view, metric), [pg, view, metric]);
  const final = (id: PlayerId) => seriesOf(view.history!, metric, id).at(-1) ?? 0;
  const m = METRICS[metric];
  return (
    <div className="pg-tab pg-growth" role="tabpanel" aria-label="Empires over time">
      <div className="pg-switch" role="radiogroup" aria-label="Measure">
        {GROWTH_METRICS.map((k) => (
          <button key={k} type="button" role="radio" aria-checked={metric === k} className={`pg-seg ${metric === k ? 'on' : ''}`} onClick={() => setMetric(k)}>
            <Glyph name={METRICS[k].icon} size={15} /> {METRICS[k].label}
          </button>
        ))}
      </div>
      <div className="pg-plot">
        <div className="pg-chart" ref={box}>
          <HistoryChart first={pg.firstTurn} last={pg.lastTurn} lines={data} eras={pg.eras} height={height}
            label={`${m.label} of every empire over the game`} unit={m.short} focus={focus} onFocus={onFocus} />
        </div>
        <Legend pg={pg} values={final} focus={focus} onFocus={onFocus} unit={m.short} />
      </div>
      <p className="pg-note">{m.note}. Point at the chart for every empire's number on that turn; click an empire on the right to pick out its line.</p>
    </div>
  );
}

function TimelineTab({ pg, view }: { pg: PostGame; view: PlayerView }) {
  const [mineOnly, setMineOnly] = useState(true);
  const entries = pg.timeline.filter((t) => !mineOnly || t.yours || t.major);
  const groups = timelineByEra(pg, entries);
  const faction = (id: PlayerId) => view.players.find((p) => p.id === id)?.faction;
  return (
    <div className="pg-tab pg-timeline" role="tabpanel" aria-label="Timeline">
      <div className="pg-switch" role="radiogroup" aria-label="Which events">
        <button type="button" role="radio" aria-checked={mineOnly} className={`pg-seg ${mineOnly ? 'on' : ''}`} onClick={() => setMineOnly(true)}>Your story</button>
        <button type="button" role="radio" aria-checked={!mineOnly} className={`pg-seg ${!mineOnly ? 'on' : ''}`} onClick={() => setMineOnly(false)}>Every empire</button>
        <span className="pg-count">{entries.length} events</span>
      </div>
      <div className="pg-scroll" tabIndex={0} aria-label="Events in turn order">
        {groups.map((g) => (
          <section key={g.era} className="pg-era">
            <h3><span>{g.name} era</span><small>turns {g.from}–{g.to}</small></h3>
            <ol>
              {g.entries.map((t, i) => (
                <li key={`${t.turn}-${i}`} className={`pg-event ${t.tone}`}>
                  <span className="pg-turn">{t.turn}</span>
                  <RingMedallion icon={t.icon} tone={t.tone === 'mine' ? 'gilt' : t.tone === 'against' ? 'bad' : 'neutral'} size={24} sign={t.tone === 'against' ? 'Against you' : undefined} />
                  <span className="pg-faces">
                    {t.who.slice(0, 2).map((id) => { const f = faction(id); return f ? <PortraitRoundel key={id} faction={f} size={20} color={playerColor(id)} /> : null; })}
                  </span>
                  <span className="pg-ev-text"><Amounts text={t.text} /></span>
                </li>
              ))}
            </ol>
          </section>
        ))}
        {!entries.length && <p className="pg-empty">No recorded events.</p>}
      </div>
    </div>
  );
}

const COLS: { key: keyof Omit<StatRow, 'empire'>; label: string; icon: string; title: string }[] = [
  { key: 'vp', label: 'VP', icon: 'vp', title: 'VP at the end' },
  { key: 'peakVp', label: 'Peak VP', icon: 'vp', title: 'Most VP held at any point' },
  { key: 'cities', label: 'Cities', icon: 'city', title: 'Cities at the end (most at any point below)' },
  { key: 'population', label: 'Pop.', icon: 'emblem-growth', title: 'Population at the end' },
  { key: 'science', label: 'Science', icon: 'yield-science', title: 'Science produced over the whole game' },
  { key: 'techs', label: 'Techs', icon: 'emblem-tech', title: 'Technologies known at the end' },
  { key: 'peakArmy', label: 'Army', icon: 'unit-soldier', title: `Army: ${POSTGAME_ARMY_TEXT}` },
  { key: 'founded', label: 'Founded', icon: 'emblem-founded', title: 'Cities founded' },
  { key: 'captured', label: 'Taken', icon: 'emblem-captured', title: 'Cities captured or razed' },
  { key: 'lost', label: 'Lost', icon: 'ui-alert', title: 'Cities lost to capture or razing' },
  { key: 'wars', label: 'Wars', icon: 'unit-soldier', title: 'Wars fought' },
];

function StatsTab({ pg, playTime }: { pg: PostGame; playTime: ReturnType<typeof playTimeSummary> }) {
  const best = Object.fromEntries(COLS.map((c) => [c.key, Math.max(...pg.stats.map((r) => r[c.key]))]));
  return (
    <div className="pg-tab pg-stats" role="tabpanel" aria-label="Statistics">
      {playTime && <p className="pg-play-time">{playTimeReportLine(playTime)}</p>}
      <div className="pg-scroll" tabIndex={0}>
        <table className="pg-table">
          <thead>
            <tr>
              <th scope="col" className="pg-th-name">Empire</th>
              {COLS.map((c) => <th key={c.key} scope="col" title={c.title}><Glyph name={c.icon} size={14} /><span>{c.label}</span></th>)}
            </tr>
          </thead>
          <tbody>
            {pg.stats.map((r) => (
              <tr key={r.empire.id} className={`${r.empire.mine ? 'mine' : ''} ${r.empire.winner ? 'win' : ''} ${r.empire.alive ? '' : 'out'}`}>
                <th scope="row">
                  <span className="pg-row-who">
                    <PortraitRoundel faction={r.empire.faction} size={24} color={playerColor(r.empire.id)} fallen={!r.empire.alive} />
                    <span>{r.empire.label}{r.empire.winner && <Star size={8} className="pg-key-star" />}{r.empire.fell !== undefined && <small>Fell on turn {r.empire.fell}</small>}</span>
                  </span>
                </th>
                {COLS.map((c) => (
                  <td key={c.key} className={r[c.key] === best[c.key] && r[c.key] > 0 ? 'best' : ''}>
                    {fmtInt(r[c.key])}{c.key === 'cities' && <small>/{r.peakCities}</small>}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="pg-note">Cities: held at the end / most ever held. Peak VP: most VP held at any point. Science: produced over the whole game. Army: {POSTGAME_ARMY_TEXT}. Taken and Lost: cities captured or razed. Bold is the highest in each column.</p>
    </div>
  );
}

export function PostGameReport({
  view, pg, titleId, title, ribbon, reason, kicker, hero, tone, onTitle, onMap, onReplay,
}: {
  view: PlayerView;
  pg: PostGame;
  titleId: string;
  title: string;
  ribbon: string;
  reason: string | null;
  kicker: string;
  hero: PlayerPublic[];
  tone: RevealTone;
  onTitle(): void;
  onMap(): void;
  onReplay(): void;
}) {
  const client = useGame((s) => s.client);
  const playTime = playTimeSummary(client?.archive?.());
  const [tab, setTab] = useState<Tab>('race');
  const [focus, setFocus] = useState<PlayerId | null>(null);
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const onTabKey = (e: KeyboardEvent, i: number) => {
    const d = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const j = (i + d + TABS.length) % TABS.length;
    setTab(TABS[j].id);
    tabRefs.current[j]?.focus();
  };
  let body: ReactNode;
  if (tab === 'race') body = <RaceTab pg={pg} view={view} focus={focus} onFocus={setFocus} />;
  else if (tab === 'growth') body = <GrowthTab pg={pg} view={view} focus={focus} onFocus={setFocus} />;
  else if (tab === 'timeline') body = <TimelineTab pg={pg} view={view} />;
  else body = <StatsTab pg={pg} playTime={playTime} />;
  const lead: ReportEmpire | undefined = pg.empires.find((e) => e.id === hero[0]?.id);
  return (
    <div className={`pg pg-${tone}`} data-testid="postgame-report">
      <header className="pg-head">
        <div className="pg-hero">
          {hero.slice(0, 2).map((m) => <PortraitFrame key={m.id} faction={m.faction} fallen={tone === 'fallen'} crestSize={26} />)}
        </div>
        <div className="pg-title">
          <div className="go-kicker"><Star size={7} /> {kicker} · Game report <Star size={7} /></div>
          <div className="pg-title-line">
            <h1 id={titleId}>{title}</h1>
            <Ribbon tone={tone === 'fallen' ? 'red' : tone === 'draw' ? 'gilt' : 'ink'}>{ribbon}</Ribbon>
          </div>
          {reason && <p className="pg-reason"><Amounts text={reason} /></p>}
          {pg.partial && <p className="pg-partial">The record starts on turn {pg.firstTurn}: this game was saved before Legatum kept a history.</p>}
        </div>
        <div className="pg-tabs" role="tablist" aria-label="Report">
          {TABS.map((t, i) => (
            <button key={t.id} ref={(el) => { tabRefs.current[i] = el; }} role="tab" type="button" aria-selected={tab === t.id} tabIndex={tab === t.id ? 0 : -1}
              className={`pg-tab-btn ${tab === t.id ? 'on' : ''}`} onClick={() => setTab(t.id)} onKeyDown={(e) => onTabKey(e, i)} data-tab={t.id}>
              <Glyph name={t.icon} size={15} /> {t.label}
            </button>
          ))}
        </div>
      </header>
      <div className="pg-body">{body}</div>
      <footer className="go-actions pg-actions">
        <button className="btn primary" onClick={onTitle}>Return to title</button>
        <button className="btn" onClick={onMap}>Keep looking at the map</button>
        <button className="btn quiet" onClick={onReplay}>Replay the reveal</button>
        {lead && <span className="pg-sign" aria-hidden="true"><Star size={7} /> {pg.lastTurn - pg.firstTurn + 1} turns recorded</span>}
      </footer>
    </div>
  );
}
