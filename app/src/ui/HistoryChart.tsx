// The post-game report's line chart: one line per empire over the game's turns, the eras as vellum bands, an
// optional target rule, and (for the VP race) lanes of event marks beneath it. Hover or arrow keys pick a turn,
// and the readout lists every empire's number on that turn: nothing on the chart is unlabelled.
import { useLayoutEffect, useMemo, useRef, useState, type KeyboardEvent, type PointerEvent } from 'react';
import type { EraBand, LaneKind, Marker, ReportEmpire } from '../postgame';
import type { PlayerId } from '../types';
import { ERA_NAMES } from '../data';
import { PortraitRoundel } from './premium';
import { Icon } from './bits';
import { playerColor, playerInk } from '../theme';

/** Whole numbers with thousands separators, as the rest of the game writes them. */
export const fmtInt = (n: number) => n.toLocaleString('en-US');

export interface ChartLine {
  empire: ReportEmpire;
  /** One value per turn from `first`. */
  values: number[];
  /** How many of `values` to draw (a fallen empire's line ends where it fell). */
  drawn: number;
}

const LANES: { kind: LaneKind; label: string }[] = [
  { kind: 'war', label: 'Wars' },
  { kind: 'capture', label: 'Captures' },
];

/** Round tick steps (1, 2, 5 x 10^k) for about `count` ticks up to `max`. */
function niceTicks(max: number, count = 4): number[] {
  if (max <= 0) return [0];
  const raw = max / count;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? raw;
  const ticks: number[] = [];
  for (let v = 0; v <= max + 1e-9; v += step) ticks.push(Math.round(v));
  if (ticks.at(-1)! < max) ticks.push(ticks.at(-1)! + step);
  return ticks;
}

/** Spread end labels so none overlaps: each keeps its order and is pushed at least `gap` from its neighbour. */
function spreadLabels(ys: number[], gap: number, top: number, bottom: number): number[] {
  const order = ys.map((y, i) => ({ y, i })).sort((a, b) => a.y - b.y);
  const out = order.map((o) => o.y);
  for (let k = 1; k < out.length; k++) out[k] = Math.max(out[k], out[k - 1] + gap);
  const over = out.length ? out.at(-1)! - bottom : 0;
  if (over > 0) for (let k = out.length - 1; k >= 0; k--) out[k] = Math.max(top + k * gap, Math.min(out[k] - over, k < out.length - 1 ? out[k + 1] - gap : bottom));
  const result = new Array<number>(ys.length);
  order.forEach((o, k) => { result[o.i] = out[k]; });
  return result;
}

export function HistoryChart({
  first, last, lines, eras, target, step = false, height, label, unit, markers, focus, onFocus,
}: {
  first: number;
  last: number;
  lines: ChartLine[];
  eras: EraBand[];
  target?: number | null;
  /** Draw as steps (VP changes in whole points). */
  step?: boolean;
  height: number;
  /** The chart's accessible name ("Victory points over the game"). */
  label: string;
  unit: string;
  markers?: Marker[];
  focus: PlayerId | null;
  onFocus(id: PlayerId | null): void;
}) {
  const host = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(820);
  const [turn, setTurn] = useState<number | null>(null);
  const [mark, setMark] = useState<{ x: number; y: number; text: string[] } | null>(null);
  useLayoutEffect(() => {
    const el = host.current;
    if (!el) return;
    const measure = () => setWidth(Math.max(320, Math.round(el.clientWidth)));
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);
  const lanes = markers ? LANES : [];
  const laneH = 15;
  const pad = { l: lanes.length ? 64 : 40, r: 44, t: 22, b: 20 + lanes.length * laneH + (lanes.length ? 8 : 0) };
  const plotW = Math.max(100, width - pad.l - pad.r);
  const plotH = Math.max(80, height - pad.t - pad.b);
  const span = Math.max(1, last - first);
  const x = (t: number) => pad.l + ((t - first) / span) * plotW;
  const maxV = Math.max(target ?? 0, ...lines.map((l) => Math.max(0, ...l.values.slice(0, l.drawn))));
  // Headroom above the highest line or the target, so neither runs along the top edge under the era labels.
  const ticks = niceTicks(Math.ceil(maxV * 1.12) + (target ? 1 : 0), height < 260 ? 3 : 4);
  const top = ticks.at(-1)! || 1;
  const y = (v: number) => pad.t + plotH - (v / top) * plotH;
  const ordered = useMemo(() => [...lines].sort((a, b) => Number(a.empire.mine) - Number(b.empire.mine) || Number(a.empire.winner) - Number(b.empire.winner)), [lines]);
  const pathOf = (l: ChartLine) => {
    const pts = l.values.slice(0, l.drawn);
    if (!pts.length) return '';
    let d = `M${x(first).toFixed(1)} ${y(pts[0]).toFixed(1)}`;
    for (let i = 1; i < pts.length; i++) {
      const px = x(first + i).toFixed(1), py = y(pts[i]).toFixed(1);
      d += step ? `H${px}V${py}` : `L${px} ${py}`;
    }
    return d;
  };
  // End labels: a roundel at each line's last point, spread so none overlaps.
  // End roundels only where they help: your own line, the winner's and the one picked out (the legend carries the rest),
  // and none for a line that ended early (a fallen empire's roundel would sit on the axis).
  const ends = lines
    .filter((l) => (l.empire.mine || l.empire.winner || focus === l.empire.id) && l.drawn === l.values.length)
    .map((l) => ({ l, at: first + Math.max(0, l.drawn - 1), v: l.values[Math.max(0, l.drawn - 1)] ?? 0 }));
  const endYs = spreadLabels(ends.map((e) => y(e.v)), 20, pad.t + 8, pad.t + plotH);
  const xTicks = useMemo(() => {
    const every = span > 240 ? 50 : span > 120 ? 25 : span > 50 ? 10 : 5;
    const out: number[] = [];
    for (let t = Math.ceil(first / every) * every; t <= last; t += every) out.push(t);
    return out;
  }, [first, last, span]);
  const bands = [{ era: 1, name: ERA_NAMES[0] ?? 'Ancient', turn: first, by: null } as EraBand, ...eras.filter((b) => b.turn > first)];
  const pick = (clientX: number, clientY: number) => {
    const r = host.current!.getBoundingClientRect();
    // The event lanes below the plot have their own hover; only the plot picks a turn.
    if (clientY - r.top > pad.t + plotH + 6) { setTurn(null); return; }
    const t = Math.round(first + ((clientX - r.left - pad.l) / plotW) * span);
    setTurn(Math.max(first, Math.min(last, t)));
  };
  const onKey = (e: KeyboardEvent) => {
    const now = turn ?? last;
    const d = e.shiftKey ? 10 : 1;
    if (e.key === 'ArrowLeft') setTurn(Math.max(first, now - d));
    else if (e.key === 'ArrowRight') setTurn(Math.min(last, now + d));
    else if (e.key === 'Home') setTurn(first);
    else if (e.key === 'End') setTurn(last);
    else return;
    e.preventDefault();
    e.stopPropagation();
  };
  const readout = turn !== null ? lines
    .map((l) => ({ l, v: turn - first < l.drawn ? l.values[turn - first] : null }))
    .filter((r) => r.v !== null)
    .sort((a, b) => (b.v ?? 0) - (a.v ?? 0)) : [];
  const grouped = useMemo(() => {
    const m = new Map<string, Marker[]>();
    for (const mk of markers ?? []) {
      const k = `${mk.lane}:${mk.turn}`;
      if (!m.has(k)) m.set(k, []);
      m.get(k)!.push(mk);
    }
    return [...m.values()];
  }, [markers]);
  const laneY = (kind: LaneKind) => pad.t + plotH + 22 + LANES.findIndex((l) => l.kind === kind) * laneH;
  return (
    <div className="hc" ref={host} style={{ height }}>
      <svg width={width} height={height} role="img" aria-label={label} tabIndex={0} onKeyDown={onKey}
        onPointerMove={(e: PointerEvent) => { if (e.pointerType !== 'touch' || e.buttons) pick(e.clientX, e.clientY); }}
        onPointerDown={(e: PointerEvent) => pick(e.clientX, e.clientY)}
        onPointerLeave={() => { setTurn(null); setMark(null); }}
        onBlur={() => setTurn(null)}>
        <defs>
          <clipPath id="hc-plot"><rect x={pad.l} y={pad.t - 4} width={plotW + 2} height={plotH + 8} /></clipPath>
        </defs>
        {bands.map((b, i) => {
          const x0 = x(b.turn), x1 = i + 1 < bands.length ? x(bands[i + 1].turn) : x(last);
          const w = Math.max(0, x1 - x0);
          return (
            <g key={b.era} className={`hc-band ${i % 2 ? 'odd' : ''}`}>
              <rect x={x0} y={pad.t} width={w} height={plotH} />
              {i > 0 && <line className="hc-era-rule" x1={x0} x2={x0} y1={pad.t - 14} y2={pad.t + plotH} />}
              <text className="hc-era" x={x0 + 4} y={pad.t - 6}>{w > b.name.length * 7.5 + 8 ? b.name : w > 30 ? `${b.name.slice(0, 3)}.` : ''}</text>
            </g>
          );
        })}
        {ticks.map((v) => (
          <g key={v} className="hc-grid">
            <line x1={pad.l} x2={pad.l + plotW} y1={y(v)} y2={y(v)} />
            <text x={pad.l - 6} y={y(v) + 4} textAnchor="end">{fmtInt(v)}</text>
          </g>
        ))}
        {xTicks.map((t) => (
          <text key={t} className="hc-xtick" x={x(t)} y={pad.t + plotH + 14} textAnchor="middle">{t}</text>
        ))}
        {target ? (
          <g className="hc-target">
            <line x1={pad.l} x2={pad.l + plotW} y1={y(target)} y2={y(target)} />
            <text x={pad.l + 6} y={y(target) - 5}>Target {target} VP</text>
          </g>
        ) : null}
        <g clipPath="url(#hc-plot)">
          {ordered.map((l) => {
            const d = pathOf(l);
            const dim = focus !== null && focus !== l.empire.id;
            return (
              <g key={l.empire.id} className={`hc-line ${l.empire.mine ? 'mine' : ''} ${dim ? 'dim' : ''} ${focus === l.empire.id ? 'focus' : ''}`}
                onPointerEnter={() => onFocus(l.empire.id)} onPointerLeave={() => onFocus(null)}>
                {l.empire.mine && <path className="hc-glow" d={d} />}
                <path d={d} stroke={l.empire.ink} />
                <path className="hc-hit" d={d} />
              </g>
            );
          })}
        </g>
        {turn !== null && <line className="hc-cursor" x1={x(turn)} x2={x(turn)} y1={pad.t} y2={pad.t + plotH} />}
        {turn !== null && readout.map(({ l, v }) => (
          <circle key={l.empire.id} className="hc-dot" cx={x(turn)} cy={y(v ?? 0)} r={l.empire.mine ? 4 : 3} fill={l.empire.ink} />
        ))}
        {lanes.map((ln) => (
          <g key={ln.kind} className="hc-lane">
            <line x1={pad.l} x2={pad.l + plotW} y1={laneY(ln.kind)} y2={laneY(ln.kind)} />
            <text x={pad.l - 6} y={laneY(ln.kind) + 3.5} textAnchor="end">{ln.label}</text>
          </g>
        ))}
        {grouped.map((ms) => {
          const m0 = ms[0];
          const cx = x(m0.turn), cy = laneY(m0.lane);
          const mine = ms.some((m) => m.mine);
          return (
            <g key={`${m0.lane}:${m0.turn}`} className={`hc-mark ${m0.lane} ${mine ? 'mine' : ''}`}
              onPointerEnter={() => setMark({ x: cx, y: cy, text: ms.map((m) => m.text) })} onPointerLeave={() => setMark(null)}>
              <title>{ms.map((m) => m.text).join('\n')}</title>
              {m0.lane === 'war'
                ? <path d={`M${cx - 4} ${cy - 4}L${cx + 4} ${cy + 4}M${cx + 4} ${cy - 4}L${cx - 4} ${cy + 4}`} stroke={m0.who !== null ? playerInk(m0.who) : 'var(--ink)'} />
                : <rect x={cx - 3.5} y={cy - 3.5} width={7} height={7} fill={m0.who !== null ? playerInk(m0.who) : 'var(--muted)'} />}
              <rect className="hc-mark-hit" x={cx - 6} y={cy - 7} width={12} height={14} />
            </g>
          );
        })}
      </svg>
      <div className="hc-ends" aria-hidden="true">
        {ends.map((e, i) => (
          <span key={e.l.empire.id} className={`hc-end ${focus !== null && focus !== e.l.empire.id ? 'dim' : ''} ${e.l.empire.mine ? 'mine' : ''}`}
            style={{ left: x(e.at) + 6, top: endYs[i] - 9 }} onPointerEnter={() => onFocus(e.l.empire.id)} onPointerLeave={() => onFocus(null)}>
            <PortraitRoundel faction={e.l.empire.faction} size={18} color={playerColor(e.l.empire.id)} fallen={!e.l.empire.alive} />
            {e.l.empire.winner && <Icon name="vp" size={10} className="hc-win" />}
          </span>
        ))}
      </div>
      {turn !== null && readout.length > 0 && (
        <div className={`hc-readout ${x(turn) > width * 0.6 ? 'left' : ''}`} style={x(turn) > width * 0.6 ? { right: width - x(turn) + 10 } : { left: x(turn) + 10 }} role="status">
          <div className="hc-readout-head">Turn {turn}</div>
          {readout.map(({ l, v }) => (
            <div key={l.empire.id} className={`hc-readout-row ${l.empire.mine ? 'mine' : ''}`}>
              <i style={{ background: l.empire.ink }} />
              <span>{l.empire.label}</span>
              <b>{fmtInt(v ?? 0)}</b>
            </div>
          ))}
          <div className="hc-readout-unit">{unit}</div>
        </div>
      )}
      {mark && turn === null && (
        <div className="hc-readout mark" style={mark.x > width * 0.6 ? { right: width - mark.x + 10, top: Math.max(4, mark.y - 60) } : { left: mark.x + 10, top: Math.max(4, mark.y - 60) }} role="status">
          {mark.text.map((t) => <div key={t} className="hc-mark-text">{t}</div>)}
        </div>
      )}
    </div>
  );
}
