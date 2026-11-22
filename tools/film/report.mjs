#!/usr/bin/env node
// Morning report: node tools/film/report.mjs <run-dir>  ->  <run-dir>/report.html
// A self-contained page (inline CSS, fonts and SVG, no scripts) in the editorial variant.
// It reads only the run log (docs/archive/ai-evolution-log.md). Every number names the generation, the
// checkpoint or the game it comes from (pillar 5).
import fs from "node:fs";
import { toolBrandCss, wordmark } from "../design/brand.mjs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadRun, eras as erasOf, archetypesOf, archetypeColor, matrixOf } from "./lib/runlog.mjs";

const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const pct = (x, d = 0) => (x == null ? "–" : `${(x * 100).toFixed(d)}%`);
const num = (x, d = 2) => (x == null ? "–" : Number(x).toFixed(d));
const human = (s) => String(s || "").replaceAll("_", " ");
const cite = (label) => `<span class="cite">${esc(label)}</span>`;
const gid = (id) => `<code class="gid">${esc(id)}</code>`;
const KIND = { emerged: "Emerged", disappeared: "Disappeared", dominant: "Dominant", countered: "Countered", weakness: "Weakness", new_best: "New best", archetype_takeover: "Takeover", archetype_extinct: "Extinct", league_add: "Added to league", proposal_adopted: "Proposal adopted", proposal_rejected: "Proposal rejected", stall: "Stall", guardrail: "Guardrail" };

// ---------- charts (static SVG, scaled by CSS) ----------
function axes(W, H, P, first, last, x, yTicks) {
  let g = "";
  for (const [v, label, yy] of yTicks) g += `<line x1="${P.L}" x2="${W - P.R}" y1="${yy}" y2="${yy}" class="grid"/><text x="${P.L - 8}" y="${yy + 4}" text-anchor="end">${label}</text>`;
  const span = Math.max(1, last - first);
  const step = [1, 2, 5, 10, 20, 25, 50, 100, 200, 500, 1000].find((s) => span / s <= 10) || 1000;
  for (let v = Math.ceil(first / step) * step; v <= last; v += step) g += `<text x="${x(v)}" y="${H - 10}" text-anchor="middle">${v}</text>`;
  g += `<text x="${W - P.R}" y="${H - 26}" text-anchor="end" class="axis-title">generation</text>`;
  return g;
}
function lineSvg(gens, { get, band, mean, domain, fmt, color, baseline, eraStarts, label }) {
  const pts = gens.filter((g) => get(g) != null);
  if (!pts.length) return `<p class="muted">No ${esc(label)} in the log.</p>`;
  const W = 900, H = 260, P = { L: 56, R: 18, T: 18, B: 34 };
  const first = gens[0].gen, last = Math.max(gens.at(-1).gen, first + 1);
  const vals = pts.flatMap((g) => [get(g), ...(band ? band.map((b) => b(g)) : [])]).filter((v) => v != null);
  const [lo, hi] = domain || [Math.min(0, ...vals), Math.max(...vals) * 1.08 || 1];
  const x = (gen) => P.L + ((gen - first) / (last - first)) * (W - P.L - P.R);
  const y = (v) => P.T + (1 - (v - lo) / (hi - lo || 1)) * (H - P.T - P.B);
  let g = axes(W, H, P, first, last, x, [0, 1, 2, 3, 4].map((i) => lo + ((hi - lo) * i) / 4).map((v) => [v, fmt(v), y(v)]));
  for (const s of eraStarts || []) g += `<line x1="${x(s - 0.5)}" x2="${x(s - 0.5)}" y1="${P.T}" y2="${H - P.B}" class="era"/>`;
  if (band) {
    const b = pts.filter((p) => band[0](p) != null && band[1](p) != null);
    if (b.length) g += `<path d="M${b.map((p) => `${x(p.gen)},${y(band[1](p))}`).join("L")}L${[...b].reverse().map((p) => `${x(p.gen)},${y(band[0](p))}`).join("L")}Z" fill="${color}" fill-opacity=".13"/>`;
  }
  if (mean) g += `<polyline points="${pts.filter((p) => mean(p) != null).map((p) => `${x(p.gen)},${y(mean(p))}`).join(" ")}" fill="none" stroke="${color}" stroke-opacity=".6" stroke-dasharray="5 4" stroke-width="1.6"/>`;
  if (baseline != null) g += `<line x1="${P.L}" x2="${W - P.R}" y1="${y(baseline)}" y2="${y(baseline)}" class="base"/><text x="${W - P.R}" y="${y(baseline) - 6}" text-anchor="end">baseline ${fmt(baseline)} (run.json)</text>`;
  g += `<polyline points="${pts.map((p) => `${x(p.gen)},${y(get(p))}`).join(" ")}" fill="none" stroke="${color}" stroke-width="2.6" stroke-linejoin="round"/>`;
  for (const p of pts) g += `<circle cx="${x(p.gen)}" cy="${y(get(p))}" r="2.6" fill="${color}"><title>Generation ${p.gen}: ${fmt(get(p))}</title></circle>`;
  return `<svg viewBox="0 0 ${W} ${H}" role="img" aria-label="${esc(label)} by generation">${g}</svg>`;
}
function shareSvg(gens, names, eraList) {
  const W = 900, H = 280, P = { L: 56, R: 18, T: 26, B: 34 };
  const first = gens[0].gen, last = Math.max(gens.at(-1).gen, first + 1);
  const x = (gen) => P.L + ((gen - first) / (last - first)) * (W - P.L - P.R);
  const y = (v) => P.T + (1 - v) * (H - P.T - P.B);
  let g = axes(W, H, P, first, last, x, [0, 0.5, 1].map((v) => [v, pct(v), y(v)]));
  let base = gens.map(() => 0);
  const xs = gens.length === 1 ? [first, last] : gens.map((p) => p.gen);
  for (const n of names) {
    const top = gens.map((p, i) => base[i] + (p.archetype_share?.[n] || 0));
    const up = gens.length === 1 ? [top[0], top[0]] : top, dn = gens.length === 1 ? [base[0], base[0]] : base;
    g += `<path d="M${xs.map((v, i) => `${x(v)},${y(up[i])}`).join("L")}L${xs.map((v, i) => `${x(v)},${y(dn[i])}`).reverse().join("L")}Z" fill="${archetypeColor(n)}" fill-opacity=".88"><title>${esc(n)}</title></path>`;
    base = top;
  }
  for (const e of eraList.slice(1)) g += `<line x1="${x(e.from - 0.5)}" x2="${x(e.from - 0.5)}" y1="${P.T}" y2="${H - P.B}" class="era"/>`;
  for (const e of eraList) if (x(e.to) - x(e.from) > 50) g += `<text x="${x((e.from + e.to) / 2)}" y="${P.T - 9}" text-anchor="middle" class="era-label">${esc(e.dominant)}</text>`;
  return `<svg viewBox="0 0 ${W} ${H}" role="img" aria-label="Archetype share of the population by generation">${g}</svg>`;
}

// ---------- the page ----------
export function buildReport(data) {
  const { run, generations: gens, events, checkpoints, coach } = data;
  if (!gens.length) throw new Error("the run has no generations yet");
  const names = archetypesOf(gens);
  const eraList = erasOf(gens);
  const first = gens[0], last = gens.at(-1);
  const best = gens.reduce((a, g) => ((g.best?.fitness ?? -Infinity) > (a.best?.fitness ?? -Infinity) ? g : a), first);
  const cpFirst = checkpoints[0], cpLast = checkpoints.at(-1);
  const wall = gens.reduce((a, g) => a + (g.wall_s || 0), 0);
  const hours = run.started_at && (run.finished_at || last.t) ? (new Date(run.finished_at || last.t) - new Date(run.started_at)) / 3600e3 : null;
  const legend = names.map((n) => `<span><i class="sw" style="background:${archetypeColor(n)}"></i>${esc(n)}</span>`).join("");
  const byGen = (list, a, b) => list.filter((x) => x.gen >= a && x.gen <= b);

  const showcaseHTML = (s, gen) => `<div class="show"><p><b>${esc(s.why)}</b><br>${gid(s.game_id)} · seed ${esc(s.seed)} · checkpoint ${gen}</p>${(s.highlight_turns || []).length ? `<ul>${s.highlight_turns.map((h) => `<li><span class="turn">Turn ${h.turn}</span> ${esc(h.note)} ${cite(`player ${h.player + 1}`)}</li>`).join("")}</ul>` : ""}${s.archive_path ? `<p class="muted small">Tape: <code>${esc(s.archive_path)}</code></p>` : ""}</div>`;
  const obsHTML = (o, gen) => `<li><span class="kind k-${esc(o.kind)}">${esc(KIND[o.kind] || o.kind)}</span> <b>${esc(human(o.behavior))}</b>${o.metric_change ? `: ${esc(o.metric_change.name)} ${fmtMetric(o.metric_change.before, o.metric_change.name)} → <b>${fmtMetric(o.metric_change.after, o.metric_change.name)}</b>` : ""} ${o.confidence != null ? `<span class="muted">(confidence ${esc(o.confidence)})</span>` : ""} ${cite(`coach/${gen}.json`)}
    ${(o.evidence || []).length ? `<ul class="evidence">${o.evidence.map((e) => `<li>${gid(e.game_id)} turn ${e.turn}, player ${e.player + 1}: ${esc(e.detail)}</li>`).join("")}</ul>` : ""}</li>`;
  const fmtMetric = (v, name) => (v == null ? "–" : Math.abs(v) <= 1 && !/fitness/i.test(name || "") ? pct(v, Math.abs(v) < 0.1 && v !== 0 ? 1 : 0) : num(v));

  const chapters = eraList.map((e, i) => {
    const g0 = gens.find((g) => g.gen === e.from), g1 = gens.find((g) => g.gen === e.to);
    const evs = byGen(events, e.from, e.to);
    const co = byGen(coach, e.from, e.to);
    const cps = byGen(checkpoints, e.from, e.to);
    const shares = Object.entries(e.mean_share).filter(([, v]) => v >= 0.05).sort((a, b) => b[1] - a[1]);
    const stage = run.standin?.stages?.filter((s) => s.to_gen >= e.from && s.from_gen <= e.to) || [];
    return `<section class="chapter"><h3><span class="ch">Chapter ${i + 1}</span> Generations ${e.from}–${e.to}: the ${esc(e.dominant)} era</h3>
      ${stage.length ? `<p class="standin-note">Stand-in stage${stage.length > 1 ? "s" : ""}: ${stage.map((s) => `${esc(s.name)} (generations ${s.from_gen}–${s.to_gen})`).join("; ")}.</p>` : ""}
      <p>${esc(e.dominant)} was the most common strategy, on average ${pct(e.dominant_share)} of the population ${cite(`generations.jsonl, gens ${e.from}–${e.to}`)}. The mix: ${shares.map(([n, v]) => `${esc(n)} ${pct(v)}`).join(", ")}.
      Best fitness went from ${num(g0.best?.fitness)} ${cite(`gen ${g0.gen}`)} to ${num(g1.best?.fitness)} ${cite(`gen ${g1.gen}`)}; the era's best was ${num(e.best.fitness)} (${gid(e.best.id)}, ${cite(`gen ${e.best.gen}`)}).
      ${e.proxy_win_rate ? `The Proxy won ${pct(e.proxy_win_rate.first)} of games at generation ${e.from} and ${pct(e.proxy_win_rate.last)} at generation ${e.to} ${cite("best.proxy_win_rate")}.` : ""}</p>
      ${evs.length ? `<h4>Milestones</h4><ul class="events">${evs.map((x) => `<li><span class="kind k-${esc(x.kind)}">${esc(KIND[x.kind] || x.kind)}</span> ${esc(x.detail)} ${cite(`gen ${x.gen}`)}${x.game_id ? ` ${gid(x.game_id)}` : ""}</li>`).join("")}</ul>` : ""}
      ${co.map((c) => `<h4>What the coach saw at generation ${c.gen}</h4><blockquote>${esc(c.summary)}</blockquote>${c.dominant_strategy ? `<p>Dominant strategy: <b>${esc(c.dominant_strategy.archetype)}</b>, ${pct(c.dominant_strategy.share)} of the population${c.dominant_strategy.counter_proposal ? `. Proposed counter: ${esc(typeof c.dominant_strategy.counter_proposal === "string" ? c.dominant_strategy.counter_proposal : JSON.stringify(c.dominant_strategy.counter_proposal))}` : ""} ${cite(`coach/${c.gen}.json`)}</p>` : ""}${(c.observations || []).length ? `<ul class="obs">${c.observations.map((o) => obsHTML(o, c.gen)).join("")}</ul>` : '<p class="muted">No observations at this checkpoint.</p>'}${(c.proposals || []).length ? `<p>${c.proposals.length} proposal(s) recorded ${cite(`coach/${c.gen}.json`)}.</p>` : ""}`).join("")}
      ${cps.map((c) => `<h4>Showcase games at checkpoint ${c.gen}</h4>${(c.showcase || []).map((s) => showcaseHTML(s, c.gen)).join("") || '<p class="muted">None.</p>'}`).join("")}
    </section>`;
  }).join("");

  // Before and after.
  const benchRows = cpFirst && cpLast && cpFirst !== cpLast ? [
    ["Proxy win rate (lower is better)", pct(cpFirst.benchmark?.proxy_win_rate), pct(cpLast.benchmark?.proxy_win_rate)],
    ["AI survival at turn 150", pct(cpFirst.benchmark?.ai_survival_t150), pct(cpLast.benchmark?.ai_survival_t150)],
    ["Games in the benchmark sample", cpFirst.benchmark?.sample?.game_ids?.length ?? "–", cpLast.benchmark?.sample?.game_ids?.length ?? "–"],
  ] : [];
  const keyed = (a, b) => [...new Set([...Object.keys(a || {}), ...Object.keys(b || {})])].sort();
  const winTable = (title, key) => {
    if (!cpFirst || !cpLast || cpFirst === cpLast) return "";
    const A = cpFirst.benchmark?.[key] || {}, B = cpLast.benchmark?.[key] || {};
    const rows = keyed(A, B);
    if (!rows.length) return "";
    return `<h4>${esc(title)}</h4><div class="tw"><table><thead><tr><th></th><th>Generation ${cpFirst.gen}</th><th>Generation ${cpLast.gen}</th></tr></thead><tbody>${rows.map((r) => `<tr><th>${key === "archetype_win_rates" ? `<i class="sw" style="background:${archetypeColor(r)}"></i>` : ""}${esc(r)}</th><td>${pct(A[r])}</td><td>${pct(B[r])}</td></tr>`).join("")}</tbody></table></div>`;
  };
  const m = matrixOf(cpLast?.benchmark?.matchup_matrix);
  const matrixHTML = m ? `<h4>Matchup matrix at generation ${cpLast.gen}</h4><div class="tw"><table class="matrix"><thead><tr><th></th>${m.names.map((n) => `<th>${esc(n)}</th>`).join("")}</tr></thead><tbody>${m.names.map((a, i) => `<tr><th><i class="sw" style="background:${archetypeColor(a)}"></i>${esc(a)}</th>${m.rate[i].map((v, j) => `<td style="background:${v == null ? "transparent" : v >= 0.5 ? `rgba(111,207,151,${Math.min(0.7, (v - 0.5) * 1.6 + 0.08)})` : `rgba(224,122,95,${Math.min(0.7, (0.5 - v) * 1.6 + 0.08)})`}">${v == null ? "·" : pct(v)}${m.pairs?.[i]?.[j] ? `<small>${m.pairs[i][j]}</small>` : ""}</td>`).join("")}</tr>`).join("")}</tbody></table></div><p class="muted small">${esc(m.definition || "How often the row archetype finishes ahead of the column archetype.")} Small numbers are pairings. ${cite(`checkpoints/${cpLast.gen}.json`)}</p>` : "";

  // What's still unsolved.
  const unsolved = [];
  const lastCoach = coach.at(-1);
  for (const o of lastCoach?.observations || []) if (o.kind === "weakness") unsolved.push(`A weakness the coach flagged at generation ${lastCoach.gen}: <b>${esc(human(o.behavior))}</b>${o.metric_change ? ` (${esc(o.metric_change.name)} ${fmtMetric(o.metric_change.before, o.metric_change.name)} → ${fmtMetric(o.metric_change.after, o.metric_change.name)})` : ""}. ${cite(`coach/${lastCoach.gen}.json`)}`);
  if (lastCoach?.dominant_strategy) unsolved.push(`One strategy dominates: <b>${esc(lastCoach.dominant_strategy.archetype)}</b> is ${pct(lastCoach.dominant_strategy.share)} of the population (pillar 10 wants no dominant strategy)${lastCoach.dominant_strategy.counter_proposal ? "" : "; no counter has been proposed yet"}. ${cite(`coach/${lastCoach.gen}.json`)}`);
  const pendingProposals = coach.reduce((a, c) => a + (c.proposals || []).length, 0) - events.filter((e) => e.kind === "proposal_adopted" || e.kind === "proposal_rejected").length;
  if (pendingProposals > 0) unsolved.push(`${pendingProposals} coach proposal(s) without an adopted or rejected event. ${cite("coach/*.json, events.jsonl")}`);
  if (last.best?.faction_ratio != null && last.best.faction_ratio > 1.5) unsolved.push(`Faction balance: the best faction wins ${num(last.best.faction_ratio)}× as often as the worst (target at most 1.5×). ${cite(`gen ${last.gen}`)}`);
  if (last.best?.proxy_win_rate != null && last.best.proxy_win_rate > 0.25) unsolved.push(`The Proxy still wins ${pct(last.best.proxy_win_rate)} of games at generation ${last.gen}; the goal is under 25% against Hard AIs (docs/ai-competitiveness.md). ${cite(`gen ${last.gen}`)}`);
  for (const e of events.filter((x) => x.kind === "guardrail" || x.kind === "stall").slice(-5)) unsolved.push(`${esc(KIND[e.kind])} at generation ${e.gen}: ${esc(e.detail)} ${cite(`events.jsonl gen ${e.gen}`)}`);
  const extinct = events.filter((e) => e.kind === "archetype_extinct");
  const revived = new Set(names.filter((n) => (last.archetype_share?.[n] || 0) > 0));
  for (const e of extinct) { const n = names.find((x) => e.detail.includes(x)); if (n && !revived.has(n)) unsolved.push(`${esc(n)} play went extinct at generation ${e.gen} and has not come back ${cite(`events.jsonl gen ${e.gen}`)}: is it countered, or just unrewarded?`); }

  const title = `AI evolution ${data.id}`;
  const summaryBits = [
    `${gens.length} generation${gens.length === 1 ? "" : "s"} (${first.gen}–${last.gen})`,
    run.config?.population != null ? `a population of ${run.config.population}` : null,
    run.config?.games_per_candidate != null ? `${run.config.games_per_candidate} game${run.config.games_per_candidate === 1 ? "" : "s"} per candidate` : null,
    run.config?.speed ? `${run.config.speed} speed` : null,
    run.config?.coach_every ? `a coach review every ${run.config.coach_every} generations` : null,
  ].filter(Boolean);

  return `<!doctype html>
<html lang="en" data-look="court" data-variant="editorial"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<style>
${toolBrandCss("editorial")}
* { box-sizing: border-box; }
html { background: var(--bg); }
body { margin:0; background: var(--bg); color: var(--text); font: 15px/1.6 var(--sans); }
main { max-width: 980px; margin: 0 auto; padding: 28px 24px 60px; }
header .brand .wordmark { width:auto; height:24px; margin-bottom:12px; }
header .brand { font-family: var(--display); font-weight: 700; letter-spacing: .3em; color: var(--gold); font-size: 13px; }
h1 { font: 700 34px/1.2 var(--display); color: var(--gold); margin: 6px 0 4px; overflow-wrap: anywhere; }
h2 { font: 600 13px var(--sans); letter-spacing: .14em; text-transform: uppercase; color: var(--muted); margin: 36px 0 12px; border-bottom: 1px solid var(--line); padding-bottom: 6px; }
h3 { font: 500 22px/1.3 var(--display); color: var(--gold); margin: 0 0 8px; }
h3 .ch { display:block; font: 600 11px var(--sans); letter-spacing: .14em; text-transform: uppercase; color: var(--muted); }
h4 { font: 600 13px var(--sans); color: var(--text); margin: 18px 0 6px; }
p { margin: 8px 0; }
.muted { color: var(--muted); } .small { font-size: 12px; }
.meta { color: var(--muted); font-size: 13px; }
.card, .chapter { background: var(--panel); border: 1px solid var(--line); border-radius: 0; padding: 16px 18px; margin: 12px 0; }
.standin { border: 1px solid var(--control-border); background: var(--panel-2); border-radius: 0; padding: 10px 14px; margin: 14px 0; font-size: 14px; }
.standin-note { color: var(--warn); font-size: 13px; }
.stats { display:grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px; margin: 14px 0; }
.stat { background: var(--panel); border:1px solid var(--line); border-radius: 0; padding: 10px 12px; }
.stat b { display:block; font-size: 22px; font-weight: 600; }
.stat span { color: var(--muted); font-size: 11px; text-transform: uppercase; letter-spacing: .08em; }
.stat-range { white-space: nowrap; }
.stat .cite { margin: 6px 0 0; display: inline-block; }
svg { width: 100%; height: auto; display: block; font: 12px var(--sans); }
svg text { fill: var(--muted); } svg .axis-title { fill: var(--muted); }
svg .grid { stroke: var(--rule); } svg .era { stroke: var(--gold); stroke-opacity: .5; stroke-dasharray: 3 4; } svg .base { stroke: var(--muted); stroke-dasharray: 6 4; }
svg .era-label { fill: var(--gold); font-family: var(--sans); font-size: 13px; }
.legend { display:flex; flex-wrap: wrap; gap: 6px 16px; font-size: 13px; margin-top: 8px; }
.legend span { display:inline-flex; align-items:center; gap: 6px; }
.sw { display:inline-block; width: 11px; height: 11px; border-radius: 50%; border: 1px solid #ffffff55; margin-right: 6px; vertical-align: -1px; }
.cite { display:inline-block; font: 500 10.5px var(--sans); color: var(--gold); border: 1px solid var(--gold-dim); border-radius: 0; padding: 0 5px; margin: 0 2px; white-space: nowrap; vertical-align: 1px; }
code, .gid { font: 12px var(--sans); color: var(--blue); overflow-wrap: anywhere; }
blockquote { margin: 8px 0; padding: 8px 14px; border-left: 3px solid var(--y-influence); background: var(--panel-2); border-radius: 0 8px 8px 0; }
ul { padding-left: 20px; } li { margin: 4px 0; }
ul.obs, ul.events { list-style: none; padding: 0; }
ul.evidence { font-size: 13px; color: var(--text); }
.kind { font: 600 10px var(--sans); letter-spacing: .08em; text-transform: uppercase; padding: 2px 7px; border-radius: 0; border: 1px solid currentColor; color: var(--blue); white-space: nowrap; }
.k-emerged, .k-new_best { color: var(--ok); } .k-dominant, .k-archetype_takeover { color: var(--warn); } .k-disappeared, .k-archetype_extinct { color: var(--muted); } .k-weakness, .k-stall, .k-guardrail { color: var(--bad); }
.show { border-top: 1px solid var(--line); padding-top: 6px; margin-top: 8px; }
.turn { font: 500 12px var(--sans); color: var(--gold); margin-right: 4px; }
.tw { overflow-x: auto; }
table { border-collapse: collapse; width: 100%; font-size: 13px; }
th, td { border-bottom: 1px solid var(--line); padding: 6px 8px; text-align: left; vertical-align: top; }
thead th { color: var(--muted); font-weight: 600; }
td { font-family: var(--sans); }
table.matrix td { text-align: center; border: 1px solid var(--bg); }
table.matrix small { display:block; color: var(--muted); font-size: 10px; }
footer { margin-top: 40px; color: var(--muted); font-size: 12px; }
@media (max-width: 600px) { main { padding: 18px 16px 40px; } h1 { font-size: 26px; } h3 { font-size: 19px; } .card, .chapter { padding: 12px; } th, td { padding: 5px 6px; } }
</style></head>
<body><main>
<header><div class="brand">${wordmark}<span>Morning report</span></div>
<h1>${esc(title)}</h1>
<p class="meta">Status: <b>${esc(run.status)}</b> · started ${esc(run.started_at || "–")}${run.finished_at ? ` · finished ${esc(run.finished_at)}` : ""}${hours != null ? ` · ${hours.toFixed(1)} hours` : ""} · written ${esc(new Date().toISOString())} from <code>${esc(path.basename(data.dir))}/</code></p></header>
${run.standin ? `<div class="standin">${esc(run.standin.note)}${run.standin.definitions ? `<br><span class="small">Definitions: ${Object.entries(run.standin.definitions).map(([k, v]) => `<b>${esc(k)}</b>: ${esc(v)}`).join("; ")}.</span>` : ""}</div>` : ""}

<h2>What the run did</h2>
<p>The run went through ${summaryBits.join(", ")}. Simulation time added up to ${(wall / 60).toFixed(1)} minutes ${cite("sum of wall_s")}.
${last.best?.proxy_win_rate != null && first.best?.proxy_win_rate != null ? `The Proxy's win rate went from ${pct(first.best.proxy_win_rate)} ${cite(`gen ${first.gen}`)} to ${pct(last.best.proxy_win_rate)} ${cite(`gen ${last.gen}`)}.` : ""}
The best candidate overall was ${gid(best.best?.id)} with fitness ${num(best.best?.fitness)} ${cite(`gen ${best.gen}`)}, playing ${esc(best.best?.archetype || "an unclassified strategy")}.
${eraList.length > 1 ? `The population went through ${eraList.length} eras: ${eraList.map((e) => `${esc(e.dominant)} (generations ${e.from}–${e.to})`).join(", then ")}.` : `One strategy led throughout: ${esc(eraList[0]?.dominant)}.`}
${coach.length ? `The coach reviewed ${coach.length} checkpoint${coach.length === 1 ? "" : "s"} and made ${coach.reduce((a, c) => a + (c.observations || []).length, 0)} observations.` : "The coach has not reviewed a checkpoint yet."}</p>
<div class="stats">
  <div class="stat"><b>${last.gen}</b><span>last generation</span></div>
  <div class="stat"><b>${num(best.best?.fitness)}</b><span>best fitness</span>${cite(`gen ${best.gen}`)}</div>
  <div class="stat"><b>${pct(last.best?.proxy_win_rate)}</b><span>Proxy win rate now</span>${cite(`gen ${last.gen}`)}</div>
  <div class="stat"><b>${pct(run.baseline?.proxy_win_rate)}</b><span>Proxy win rate at the start</span>${cite("run.json baseline")}</div>
  <div class="stat"><b>${num(last.winner_entropy)}</b><span>winner diversity <span class="stat-range">(0–1)</span></span>${cite(`gen ${last.gen}`)}</div>
</div>

<h2>The curves</h2>
<div class="card"><h4 style="margin-top:0">Best fitness <span class="muted small">(solid: best; dashed: mean; band: 10th–90th percentile)</span></h4>
${lineSvg(gens, { get: (g) => g.best?.fitness, band: [(g) => g.fitness_p10, (g) => g.fitness_p90], mean: (g) => g.mean_fitness, fmt: (v) => num(v), color: "var(--accent)", eraStarts: eraList.slice(1).map((e) => e.from), label: "Best fitness" })}</div>
<div class="card"><h4 style="margin-top:0">Proxy win rate <span class="muted small">(lower means the AI stops the Proxy more often)</span></h4>
${lineSvg(gens, { get: (g) => g.best?.proxy_win_rate, domain: [0, 1], fmt: (v) => pct(v), color: "var(--y-science)", baseline: run.baseline?.proxy_win_rate, eraStarts: eraList.slice(1).map((e) => e.from), label: "Proxy win rate" })}</div>
<div class="card"><h4 style="margin-top:0">Which strategies the population uses</h4>${shareSvg(gens, names, eraList)}<div class="legend">${legend}</div></div>

<h2>Chapters</h2>
<p class="muted">An era is a span of generations where the most common strategy stayed the same (on a three-generation rolling average; spans shorter than ${Math.max(2, Math.floor(gens.length / 12))} generations fold into their neighbour).</p>
${chapters}

<h2>Before and after</h2>
${benchRows.length ? `<div class="tw"><table><thead><tr><th>Benchmark</th><th>Checkpoint ${cpFirst.gen}</th><th>Checkpoint ${cpLast.gen}</th></tr></thead><tbody>${benchRows.map((r) => `<tr><th>${esc(r[0])}</th><td>${esc(r[1])}</td><td>${esc(r[2])}</td></tr>`).join("")}</tbody></table></div><p class="muted small">From <code>checkpoints/${cpFirst.gen}.json</code> and <code>checkpoints/${cpLast.gen}.json</code>${cpLast.benchmark?.sample?.gens ? `; the later sample pools generations ${cpLast.benchmark.sample.gens.join(", ")}` : ""}.</p>` : '<p class="muted">Only one checkpoint so far; nothing to compare yet.</p>'}
${winTable("Win rate by faction (AI seats)", "faction_win_rates")}
${winTable("Win rate by archetype", "archetype_win_rates")}
${matrixHTML}

<h2>What's still unsolved</h2>
${unsolved.length ? `<ul>${unsolved.map((u) => `<li>${u}</li>`).join("")}</ul>` : '<p class="muted">Nothing in the log flags an open problem: no weaknesses at the last checkpoint, no dominant strategy, no guardrail or stall events.</p>'}

<h2>Sources</h2>
<p class="small muted">Every number above comes from <code>${esc(path.basename(data.dir))}/</code>: <code>run.json</code>, <code>generations.jsonl</code> (${gens.length} lines), <code>events.jsonl</code> (${events.length}), <code>checkpoints/</code> (${checkpoints.map((c) => c.gen).join(", ") || "none"}) and <code>coach/</code> (${coach.map((c) => c.gen).join(", ") || "none"}). Game ids are archive tapes; open them in the dashboard's Evolution panel or replay them with <code>legatum-sim --replay &lt;tape&gt;</code>.</p>
<footer>Written by <code>tools/film/report.mjs</code>. The film of this run is <code>~/LegatumData/films/${esc(data.id)}.mp4</code>.</footer>
</main></body></html>
`;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const dir = process.argv[2];
  if (!dir) {
    console.error("usage: node tools/film/report.mjs <run-dir>");
    process.exit(2);
  }
  const data = loadRun(path.resolve(dir));
  const out = path.join(data.dir, "report.html");
  fs.writeFileSync(`${out}.tmp`, buildReport(data));
  fs.renameSync(`${out}.tmp`, out);
  console.log(out);
}
