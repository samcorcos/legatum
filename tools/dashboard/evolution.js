/* Evolution panel: the AI evolution run, read from /api/evolution (the server's 15-second cache
   of the run log). Isolated like the replay viewer: its own fetch loop, no coupling to render(). */
(() => {
  "use strict";
  const section = document.getElementById("evolution");
  if (!section) return;
  const $ = (id) => document.getElementById(`evo-${id}`);
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const pct = (x, d = 0) => (x == null ? "–" : `${(x * 100).toFixed(d)}%`);
  const num = (x, d = 2) => (x == null ? "–" : Number(x).toFixed(d));
  const human = (s) => String(s || "").replaceAll("_", " ");
  const KIND = { emerged: "Emerged", disappeared: "Disappeared", dominant: "Dominant", countered: "Countered", weakness: "Weakness", new_best: "New best", archetype_takeover: "Takeover", archetype_extinct: "Extinct", league_add: "League", proposal_adopted: "Adopted", proposal_rejected: "Rejected", stall: "Stall", guardrail: "Guardrail" };
  const store = { get: (k) => { try { return localStorage.getItem(k); } catch { return null; } }, set: (k, v) => { try { localStorage.setItem(k, v); } catch {} } };
  let data = null, lastText = "", chosen = store.get("evo-run") || "";
  const open = new Set();

  async function load() {
    try {
      const r = await fetch(`/api/evolution${chosen ? `?run=${encodeURIComponent(chosen)}` : ""}`, { cache: "no-store" });
      const text = await r.text();
      if (text === lastText) return;
      lastText = text;
      data = JSON.parse(text);
      render();
    } catch {
      /* keep the last view; the main dashboard shows the server as offline */
    }
  }

  // ---------- charts (drawn at the container's pixel width so 11px text stays 11px on a phone) ----------
  function frame(width, height, pad) {
    const W = Math.max(280, Math.round(width)), H = height;
    return { W, H, ...pad, iw: W - pad.L - pad.R, ih: H - pad.T - pad.B };
  }
  function genAxis(f, gens, x) {
    let g = "";
    const first = gens[0].gen, last = gens.at(-1).gen;
    const maxLabels = Math.max(2, Math.floor(f.iw / 70));
    const step = [1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000].find((s) => (last - first) / s <= maxLabels) || 1000;
    for (let v = Math.ceil(first / step) * step; v <= last; v += step) g += `<text x="${x(v)}" y="${f.H - 8}" fill="var(--muted)" font-size="11" text-anchor="middle">${v}</text>`;
    return g;
  }
  function eraLines(f, x) {
    return (data.run.eras || []).slice(1).map((e) => `<line x1="${x(e.from - 0.5)}" x2="${x(e.from - 0.5)}" y1="${f.T}" y2="${f.T + f.ih}" stroke="var(--accent)" stroke-opacity=".45" stroke-dasharray="3 4"/>`).join("");
  }
  function lineChart(width, { key, band, label, domain, format, color, baseline, note }) {
    const gens = data.run.gens.filter((g) => g[key] != null);
    if (!gens.length) return `<div class="empty">No ${esc(label)} in this run's log.</div>`;
    const f = frame(width, width < 560 ? 170 : 200, { L: 42, R: 14, T: 16, B: 30 });
    const first = gens[0].gen, last = Math.max(gens.at(-1).gen, first + 1);
    const vals = gens.flatMap((g) => [g[key], ...(band ? [g[band[0]], g[band[1]]] : [])]).filter((v) => v != null);
    const [lo, hi] = domain || [Math.min(0, ...vals), Math.max(...vals) * 1.08 || 1];
    const x = (gen) => f.L + ((gen - first) / (last - first)) * f.iw;
    const y = (v) => f.T + (1 - (v - lo) / (hi - lo || 1)) * f.ih;
    let g = "";
    for (let i = 0; i <= 4; i++) {
      const v = lo + ((hi - lo) * i) / 4;
      g += `<line x1="${f.L}" x2="${f.L + f.iw}" y1="${y(v)}" y2="${y(v)}" stroke="var(--rule)"/><text x="${f.L - 6}" y="${y(v) + 4}" fill="var(--muted)" font-size="11" text-anchor="end">${format(v)}</text>`;
    }
    g += eraLines(f, x);
    if (band) {
      const pts = gens.filter((p) => p[band[0]] != null && p[band[1]] != null);
      if (pts.length) g += `<path d="M${pts.map((p) => `${x(p.gen)},${y(p[band[1]])}`).join("L")}L${[...pts].reverse().map((p) => `${x(p.gen)},${y(p[band[0]])}`).join("L")}Z" fill="${color}" fill-opacity=".12"/>`;
      const mean = gens.filter((p) => p.mean != null);
      if (mean.length) g += `<polyline points="${mean.map((p) => `${x(p.gen)},${y(p.mean)}`).join(" ")}" fill="none" stroke="${color}" stroke-opacity=".55" stroke-width="1.5" stroke-dasharray="4 3"/>`;
    }
    if (baseline != null) g += `<line x1="${f.L}" x2="${f.L + f.iw}" y1="${y(baseline)}" y2="${y(baseline)}" stroke="var(--muted)" stroke-dasharray="6 4"/><text x="${f.L + f.iw}" y="${y(baseline) - 5}" fill="var(--muted)" font-size="11" text-anchor="end">baseline ${format(baseline)}</text>`;
    g += `<polyline points="${gens.map((p) => `${x(p.gen)},${y(p[key])}`).join(" ")}" fill="none" stroke="${color}" stroke-width="2.5" stroke-linejoin="round"/>`;
    const lastP = gens.at(-1);
    g += `<circle cx="${x(lastP.gen)}" cy="${y(lastP[key])}" r="4" fill="${color}"><title>Generation ${lastP.gen}: ${format(lastP[key])}</title></circle>`;
    for (const p of gens) g += `<circle cx="${x(p.gen)}" cy="${y(p[key])}" r="7" fill="transparent"><title>Generation ${p.gen}: ${esc(label)} ${format(p[key])}</title></circle>`;
    g += genAxis(f, gens, x);
    return `<svg viewBox="0 0 ${f.W} ${f.H}" width="${f.W}" height="${f.H}" role="img" aria-label="${esc(label)} by generation${note ? `; ${esc(note)}` : ""}">${g}</svg>`;
  }
  function shareChart(width) {
    const gens = data.run.gens, names = data.run.archetypes;
    if (!gens.length || !names.length) return '<div class="empty">No archetype shares yet.</div>';
    const f = frame(width, width < 560 ? 190 : 220, { L: 42, R: 14, T: 22, B: 30 });
    const first = gens[0].gen, last = Math.max(gens.at(-1).gen, first + 1);
    const x = (gen) => f.L + ((gen - first) / (last - first)) * f.iw;
    const y = (v) => f.T + (1 - v) * f.ih;
    let g = "", base = gens.map(() => 0);
    for (const a of names) {
      const top = gens.map((p, i) => base[i] + (p.share[a.name] || 0));
      const pts = gens.length === 1 ? [[x(first), 0], [x(last), 0]] : gens.map((p) => [x(p.gen), 0]);
      const upper = gens.length === 1 ? [top[0], top[0]] : top, lower = gens.length === 1 ? [base[0], base[0]] : base;
      g += `<path d="M${pts.map(([px], i) => `${px},${y(upper[i])}`).join("L")}L${pts.map(([px], i) => `${px},${y(lower[i])}`).reverse().join("L")}Z" fill="${a.color}" fill-opacity=".85"><title>${esc(a.name)}</title></path>`;
      base = top;
    }
    for (let v = 0; v <= 1; v += 0.5) g += `<text x="${f.L - 6}" y="${y(v) + 4}" fill="var(--muted)" font-size="11" text-anchor="end">${v * 100}%</text>`;
    g += eraLines(f, x);
    let lastRight = -Infinity;
    for (const e of data.run.eras || []) {
      const cx = x((e.from + e.to) / 2), label = `${e.dominant} era`, w = label.length * 6.2;
      if (cx - w / 2 < Math.max(lastRight, f.L + 4) || cx + w / 2 > f.W || x(e.to) - x(e.from) < 30) continue;
      lastRight = cx + w / 2;
      g += `<text x="${cx}" y="${f.T - 7}" fill="var(--accent)" font-size="11" text-anchor="middle">${esc(label)}</text>`;
    }
    for (const p of gens) g += `<rect x="${x(p.gen) - f.iw / gens.length / 2}" y="${f.T}" width="${Math.max(2, f.iw / gens.length)}" height="${f.ih}" fill="transparent"><title>Generation ${p.gen}: ${names.map((a) => `${esc(a.name)} ${pct(p.share[a.name] || 0)}`).join(", ")}</title></rect>`;
    g += genAxis(f, gens, x);
    return `<svg viewBox="0 0 ${f.W} ${f.H}" width="${f.W}" height="${f.H}" role="img" aria-label="Share of the population using each strategy archetype, by generation">${g}</svg>`;
  }

  // ---------- sections ----------
  function matrixHTML() {
    const m = data.run.matrix;
    if (!m || !m.names?.length) return '<div class="empty">No matchup matrix at a checkpoint yet.</div>';
    const color = (v) => (v == null ? "transparent" : v >= 0.5 ? `rgba(111,207,151,${Math.min(0.75, (v - 0.5) * 1.6 + 0.08)})` : `rgba(224,122,95,${Math.min(0.75, (0.5 - v) * 1.6 + 0.08)})`);
    const short = (n) => (n.length > 5 ? n.slice(0, 4) + "." : n);
    return `<div class="evo-matrix-wrap"><table class="evo-matrix"><thead><tr><th scope="col"><span class="evo-sr">Row archetype</span></th>${m.names.map((n) => `<th scope="col" title="${esc(n)}">${esc(short(n))}</th>`).join("")}</tr></thead><tbody>${m.names.map((a, i) => `<tr><th scope="row"><i class="evo-sw" style="background:${colorOf(a)}"></i>${esc(a)}</th>${m.rate[i].map((v, j) => `<td style="background:${color(v)}" title="${esc(a)} ahead of ${esc(m.names[j])}: ${v == null ? "no games" : pct(v)}${m.pairs?.[i]?.[j] ? ` over ${m.pairs[i][j]} pairings` : ""}">${v == null ? "·" : Math.round(v * 100)}</td>`).join("")}</tr>`).join("")}</tbody></table></div>
      <p class="evo-note">Generation ${m.gen}. ${esc(m.definition || "How often the row archetype finishes ahead of the column archetype.")} Green: row wins more often.</p>`;
  }
  const colorOf = (name) => data.run.archetypes.find((a) => a.name === name)?.color || "var(--muted)";
  const watch = (url, label = "Watch") => (url ? `<a class="evo-watch" href="${esc(url)}">${esc(label)} ▸</a>` : `<span class="evo-pending" title="The replay is exported in the background; it appears within a minute or two.">replay pending</span>`);
  const chip = (kind) => `<span class="evo-kind evo-kind-${esc(kind)}">${esc(KIND[kind] || human(kind))}</span>`;
  function change(mc) {
    if (!mc) return "";
    const f = (v) => (v == null ? "–" : Math.abs(v) <= 1 && !/fitness/i.test(mc.name) ? pct(v, Math.abs(v) < 0.1 && v !== 0 ? 1 : 0) : num(v));
    return `<span class="evo-change">${esc(mc.name)}: ${f(mc.before)} → <b>${f(mc.after)}</b></span>`;
  }
  function timelineHTML() {
    const items = data.run.timeline || [];
    if (!items.length) return '<div class="empty">Nothing yet: coach observations and milestones appear here as the run reaches them.</div>';
    return items.map((it) => {
      const key = `${it.type}-${it.gen}-${it.kind || ""}-${(it.detail || "").slice(0, 40)}`;
      if (it.type === "event") {
        return `<li class="evo-ev"><span class="evo-gen">gen ${it.gen}</span>${chip(it.kind)}<span class="evo-text">${esc(it.detail)}${it.game_id ? ` ${watch(it.watch)}` : ""}</span></li>`;
      }
      if (it.type === "checkpoint") {
        const body = it.showcase.map((s) => `<div class="evo-show"><div class="evo-show-why">${esc(s.why)} ${watch(s.watch)}</div><ul>${s.highlights.map((h) => `<li><span class="evo-turn">T${h.turn}</span><span>${esc(h.note)}</span>${watch(h.watch, "Turn " + h.turn)}</li>`).join("")}</ul><div class="evo-id">${esc(s.game_id)} · seed ${esc(s.seed)}</div></div>`).join("");
        return `<li class="evo-cp"><details data-key="${esc(key)}" ${open.has(key) ? "open" : ""}><summary><span class="evo-gen">gen ${it.gen}</span><span class="evo-kind evo-kind-checkpoint">Checkpoint</span><span class="evo-text">Proxy win rate ${pct(it.proxy_win_rate)} · AI survival at turn 150 ${pct(it.ai_survival_t150)} · ${it.showcase.length} showcase game${it.showcase.length === 1 ? "" : "s"}</span></summary>${body}</details></li>`;
      }
      const obs = it.observations.map((o) => `<div class="evo-obs">${chip(o.kind)}<b>${esc(human(o.behavior))}</b>${o.archetype ? ` <i class="evo-sw" style="background:${colorOf(o.archetype)}"></i>` : ""} ${change(o.metric_change)}${o.confidence != null ? ` <span class="evo-conf">confidence ${esc(o.confidence)}</span>` : ""}
        ${o.evidence.length ? `<ul>${o.evidence.map((e) => `<li><span class="evo-turn">T${e.turn}</span><span>${esc(e.detail)} <span class="evo-id">${esc(e.game_id)}, player ${e.player + 1}</span></span>${watch(e.watch, "Turn " + e.turn)}</li>`).join("")}</ul>` : '<div class="evo-note">No evidence listed.</div>'}</div>`).join("");
      const dom = it.dominant_strategy ? `<div class="evo-note">Dominant strategy: <b>${esc(it.dominant_strategy.archetype)}</b> (${pct(it.dominant_strategy.share)} of the population)${it.dominant_strategy.counter_proposal ? `. Proposed counter: ${esc(typeof it.dominant_strategy.counter_proposal === "string" ? it.dominant_strategy.counter_proposal : JSON.stringify(it.dominant_strategy.counter_proposal))}` : ""}</div>` : "";
      return `<li class="evo-coach"><details data-key="${esc(key)}" ${open.has(key) ? "open" : ""}><summary><span class="evo-gen">gen ${it.gen}</span><span class="evo-kind evo-kind-coach">Coach</span><span class="evo-text">${esc(it.summary)}</span><span class="evo-more">${it.observations.length} observation${it.observations.length === 1 ? "" : "s"}</span></summary>${dom}${obs || '<div class="evo-note">The coach reported no observations at this checkpoint.</div>'}</details></li>`;
    }).join("");
  }

  function render() {
    const run = data?.run;
    section.hidden = !run;
    if (!run) return;
    // Remember which timeline entries are open across refreshes.
    section.querySelectorAll("details[data-key]").forEach((d) => (d.open ? open.add(d.dataset.key) : open.delete(d.dataset.key)));
    const runs = data.runs || [];
    $("pick").hidden = runs.length < 2;
    $("pick").innerHTML = runs.map((r) => `<option value="${esc(r.id)}" ${r.id === run.id ? "selected" : ""}>${esc(r.id)}${r.standin ? " (stand-in)" : ""}</option>`).join("");
    $("status").innerHTML = `<span class="evo-status evo-status-${esc(run.status)}">${esc(run.status || "unknown")}</span>${run.standin ? '<span class="evo-status evo-standin">stand-in</span>' : ""}`;
    $("standin").hidden = !run.standin;
    if (run.standin) $("standin").innerHTML = `${esc(run.standin.note)}${run.standin.stages?.length ? ` Stages: ${run.standin.stages.map((s) => `${esc(s.name)} (generations ${s.from_gen}–${s.to_gen})`).join("; ")}.` : ""}`;
    const l = run.latest, b = run.benchmark;
    const stats = [
      [l ? l.gen : "–", `generation${run.status === "running" ? " (live)" : ""}`],
      [l ? num(l.best?.fitness) : "–", "best fitness"],
      [l ? pct(l.best?.proxy_win_rate) : "–", `Proxy win rate${run.baseline?.proxy_win_rate != null ? ` (start ${pct(run.baseline.proxy_win_rate)})` : ""}`],
      [l ? num(l.winner_entropy) : "–", "winner diversity (0–1)"],
      [run.config.population ?? "–", "population"],
    ];
    $("stats").innerHTML = stats.map(([v, t]) => `<div class="stat"><b>${esc(v)}</b><span>${esc(t)}</span></div>`).join("");
    const film = run.film;
    $("cta").innerHTML = film
      ? `<a class="evo-btn primary" href="#evo-film-player">Watch the film</a>${run.report ? `<a class="evo-btn" href="${esc(run.report)}" target="_blank">Morning report</a>` : ""}`
      : run.report ? `<a class="evo-btn primary" href="${esc(run.report)}" target="_blank">Read the morning report</a><span class="evo-note">The film appears here once <code>tools/film/make-film.sh ${esc(run.dir)}</code> has run.</span>`
      : `<span class="evo-note">Write the morning report with <code>node tools/film/report.mjs ${esc(run.dir)}</code>.</span>`;
    const vid = $("film");
    if (film) {
      const cur = vid.querySelector("video");
      if (!cur || cur.dataset.src !== film.url) {
        vid.innerHTML = `<video id="evo-film-player" controls preload="metadata" playsinline data-src="${esc(film.url)}" ${film.poster ? `poster="${esc(film.poster)}"` : ""}><source src="${esc(film.url)}" type="video/mp4">${film.vtt ? `<track kind="captions" srclang="en" label="English" src="${esc(film.vtt)}">` : ""}</video>
          <p class="evo-note">Made ${esc(new Date(film.made_at).toLocaleString())}.${film.story ? ` <a href="${esc(film.story)}" target="_blank">Why this story</a>.` : ""}${film.srt ? ` <a href="${esc(film.srt)}" download>Subtitles (.srt)</a>.` : ""}</p>`;
      }
      vid.hidden = false;
    } else vid.hidden = true;
    const w = (id) => Math.max(280, $(id).clientWidth || 600);
    $("fitness").innerHTML = lineChart(w("fitness"), { key: "best", band: ["p10", "p90"], label: "Best fitness", format: (v) => num(v), color: "var(--accent)", note: "dashed line is the population mean, band is the 10th to 90th percentile" });
    $("proxy").innerHTML = lineChart(w("proxy"), { key: "pwr", label: "Proxy win rate", domain: [0, 1], format: (v) => pct(v), color: "var(--y-science)", baseline: run.baseline?.proxy_win_rate ?? null });
    $("share").innerHTML = shareChart(w("share"));
    $("legend").innerHTML = run.archetypes.map((a) => `<span><i class="evo-sw" style="background:${a.color}"></i>${esc(a.name)} ${pct(run.gens.at(-1)?.share?.[a.name] || 0)}</span>`).join("");
    $("matrix").innerHTML = matrixHTML();
    $("bench").innerHTML = b ? `Checkpoint generation ${b.gen}: Proxy win rate <b>${pct(b.proxy_win_rate)}</b>${b.before ? ` (generation ${b.before_gen}: ${pct(b.before.proxy_win_rate)})` : ""}, AI survival at turn 150 <b>${pct(b.ai_survival_t150)}</b>${b.before ? ` (was ${pct(b.before.ai_survival_t150)})` : ""}.` : "";
    $("timeline").innerHTML = timelineHTML();
    $("watch-note").textContent = run.watch_note || "";
    $("updated").textContent = l?.t ? `last generation ${new Date(l.t).toLocaleString()}` : "";
  }

  $("pick").addEventListener("change", (e) => { chosen = e.target.value; store.set("evo-run", chosen); lastText = ""; load(); });
  $("cta").addEventListener("click", (e) => {
    if (e.target.closest('a[href="#evo-film-player"]')) { e.preventDefault(); const v = document.getElementById("evo-film-player"); v?.scrollIntoView({ block: "center" }); v?.play?.().catch(() => {}); }
  });
  let t = 0;
  addEventListener("resize", () => { clearTimeout(t); t = setTimeout(() => data && render(), 150); });
  load();
  setInterval(load, 15000);
})();
