/* Legatum evolution film: scene renderer.
   Every frame is a pure function of (scene, time): there is no requestAnimationFrame, no clock and
   no randomness, so tools/film/render.mjs can capture frame N at time N/30 and never drop a frame.
   Scenes are building blocks the storyboard composes in any order: title, text, population, graph,
   map, versus, matrix, stat, end. Map scenes draw real showcase tapes exported by legatum-sim in
   the dashboard replay viewer's format (terrain, territory, cities, armies). */
(() => {
  "use strict";
  const W = 1920, H = 1080;
  const canvas = document.getElementById("film");
  const ctx = canvas.getContext("2d");
  // The renderer supplies Court colors with Atlas type from tools/design/brand.mjs.
  let C, DISPLAY, SANS, wordmark;
  function drawWordmark(y, width, alpha) {
    ctx.save(); ctx.globalAlpha *= alpha;
    ctx.drawImage(wordmark, (W - width) / 2, y, width, width * 50 / 236);
    ctx.restore();
  }
  // Replay viewer palette (tools/dashboard/replay.js): Okabe-Ito players, muted terrain.
  const PLAYER = ["#E69F00", "#56B4E9", "#009E73", "#F0E442", "#0072B2", "#D55E00", "#CC79A7", "#D4D8DD"];
  const TERRAIN = { Ocean: "#172c40", Coast: "#254354", Grassland: "#47584a", Plains: "#625f48", Forest: "#2d463e", Hills: "#626052", Desert: "#7c7155", Tundra: "#667477", Mountains: "#484d56" };
  let P = null; // payload
  const pop = { gens: [], colors: [] };

  // ---------- helpers ----------
  const clamp = (x, a = 0, b = 1) => Math.max(a, Math.min(b, x));
  const lerp = (a, b, t) => a + (b - a) * t;
  const ease = (t) => { t = clamp(t); return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2; };
  const phase = (t, a, b) => clamp((t - a) / (b - a || 1));
  const hash = (n) => { let x = (n + 0x9e3779b9) | 0; x = Math.imul(x ^ (x >>> 16), 0x85ebca6b); x = Math.imul(x ^ (x >>> 13), 0xc2b2ae35); return ((x ^ (x >>> 16)) >>> 0) / 4294967296; };
  const hex2rgb = (h) => { const n = parseInt(h.slice(1), 16); return [(n >> 16) & 255, (n >> 8) & 255, n & 255]; };
  const mix = (a, b, t) => { const x = hex2rgb(a), y = hex2rgb(b); return `rgb(${x.map((v, i) => Math.round(lerp(v, y[i], clamp(t)))).join(",")})`; };
  const shade = (h, k) => { const x = hex2rgb(h); return `rgb(${x.map((v) => Math.round(clamp(v * k, 0, 255))).join(",")})`; };
  const colorOf = (name) => P.run.archetypes.find((a) => a.name === name)?.color || "#8f9aab";
  const pct = (x) => (x == null ? "–" : `${Math.round(x * 100)}%`);
  function font(size, { family = SANS, weight = 400, style = "normal" } = {}) { ctx.font = `${style} ${weight} ${size}px ${family}`; }
  function text(s, x, y, { size = 40, color = C.parch, align = "left", base = "alphabetic", weight = 400, family = SANS, style = "normal", alpha = 1, track = 0 } = {}) {
    font(size, { family, weight, style });
    ctx.save();
    ctx.globalAlpha *= alpha;
    ctx.fillStyle = color; ctx.textAlign = align; ctx.textBaseline = base;
    if (track) { ctx.letterSpacing = `${track}px`; }
    ctx.fillText(s, x, y);
    ctx.restore();
  }
  function wrap(s, maxW, size, opts = {}) {
    font(size, opts);
    const words = String(s).split(/\s+/), lines = [];
    let line = "";
    for (const w of words) {
      const t = line ? `${line} ${w}` : w;
      if (ctx.measureText(t).width > maxW && line) { lines.push(line); line = w; } else line = t;
    }
    if (line) lines.push(line);
    return lines;
  }
  function roundRect(x, y, w, h, r) { ctx.beginPath(); ctx.roundRect(x, y, w, h, r); }

  function background(t) {
    const g = ctx.createRadialGradient(W * 0.65, -H * 0.2, 100, W * 0.6, H * 0.4, W * 0.9);
    g.addColorStop(0, C.navy3); g.addColorStop(1, C.navy);
    ctx.fillStyle = g; ctx.fillRect(0, 0, W, H);
    // A faint hex lattice drifting slowly: the game's map, as texture.
    ctx.save(); ctx.globalAlpha = 0.05; ctx.strokeStyle = C.gold; ctx.lineWidth = 1;
    const R = 46, dx = Math.sqrt(3) * R, off = (t * 6) % dx;
    for (let row = -1; row < H / (1.5 * R) + 1; row++) for (let col = -1; col < W / dx + 2; col++) {
      const cx = col * dx + (row & 1 ? dx / 2 : 0) - off, cy = row * 1.5 * R;
      ctx.beginPath();
      for (let k = 0; k < 6; k++) { const a = Math.PI / 180 * (60 * k - 30); const px = cx + R * Math.cos(a), py = cy + R * Math.sin(a); k ? ctx.lineTo(px, py) : ctx.moveTo(px, py); }
      ctx.closePath(); ctx.stroke();
    }
    ctx.restore();
  }
  function kicker(s, alpha = 1) { text(s.toUpperCase(), 96, 104, { size: 24, color: C.gold, weight: 600, alpha, track: 5 }); }

  function captions(scene, t) {
    const cap = (scene.captions || []).find((c) => t >= c.start && t < c.end);
    if (!cap) return;
    const a = Math.min(phase(t, cap.start, cap.start + 0.18), 1 - phase(t, cap.end - 0.18, cap.end));
    const lines = wrap(cap.text, 1480, 40, { family: SANS, weight: 500 });
    const lh = 54, h = lines.length * lh + 34, y = H - 70 - h;
    ctx.save(); ctx.globalAlpha = a;
    ctx.fillStyle = "rgba(6,10,26,0.82)"; roundRect(W / 2 - 800, y, 1600, h, 14); ctx.fill();
    ctx.strokeStyle = C.rule; ctx.lineWidth = 1.5; ctx.stroke();
    lines.forEach((l, i) => text(l, W / 2, y + 17 + lh * i + 40, { size: 40, weight: 500, align: "center", color: C.parch }));
    ctx.restore();
  }

  // ---------- blobs (Primer-style creatures) ----------
  function blob(x, y, r, color, { t = 0, id = 0, alpha = 1, hop = 0, look = 0 } = {}) {
    const bob = Math.sin(t * 2.2 + id * 1.7) * r * 0.05;
    const squash = 1 + Math.sin(t * 2.2 + id * 1.7) * 0.04 - hop * 0.12;
    const yy = y + bob - hop * r * 0.9;
    ctx.save(); ctx.globalAlpha *= alpha;
    // shadow
    ctx.fillStyle = "rgba(0,0,0,0.28)"; ctx.beginPath(); ctx.ellipse(x, y + r * 0.95, r * (0.8 - hop * 0.25), r * 0.16, 0, 0, Math.PI * 2); ctx.fill();
    ctx.translate(x, yy); ctx.scale(1 / Math.sqrt(squash), squash);
    // body: a soft dome with a flatter base
    ctx.beginPath();
    ctx.moveTo(-r, r * 0.55);
    ctx.bezierCurveTo(-r * 1.08, -r * 0.35, -r * 0.62, -r * 1.02, 0, -r * 1.02);
    ctx.bezierCurveTo(r * 0.62, -r * 1.02, r * 1.08, -r * 0.35, r, r * 0.55);
    ctx.bezierCurveTo(r * 0.8, r * 0.95, -r * 0.8, r * 0.95, -r, r * 0.55);
    ctx.closePath();
    const g = ctx.createLinearGradient(0, -r, 0, r);
    g.addColorStop(0, shade(color.startsWith("#") ? color : "#8f9aab", 1.12)); g.addColorStop(1, color.startsWith("#") ? shade(color, 0.82) : color);
    ctx.fillStyle = color.startsWith("#") ? g : color; ctx.fill();
    ctx.lineWidth = Math.max(1.5, r * 0.07); ctx.strokeStyle = "rgba(10,17,40,0.55)"; ctx.stroke();
    // eyes, with a blink every few seconds
    const blink = ((t + hash(id) * 5) % 4.3) < 0.13 ? 0.12 : 1;
    for (const s of [-1, 1]) {
      const ex = s * r * 0.33 + look * r * 0.06, ey = -r * 0.28;
      ctx.fillStyle = "#fbf8ef"; ctx.beginPath(); ctx.ellipse(ex, ey, r * 0.19, r * 0.23 * blink, 0, 0, Math.PI * 2); ctx.fill();
      if (blink > 0.5) { ctx.fillStyle = "#101629"; ctx.beginPath(); ctx.arc(ex + look * r * 0.07, ey + r * 0.04, r * 0.1, 0, Math.PI * 2); ctx.fill(); }
    }
    ctx.restore();
  }

  // Stable population colors: blob i keeps its color unless its archetype shrank.
  function preparePopulation() {
    const gens = P.run.gens, names = P.run.archetypes.map((a) => a.name);
    const N = Math.max(16, Math.min(64, P.run.population || 32));
    const order = [...Array(N).keys()].sort((a, b) => hash(a * 7 + 3) - hash(b * 7 + 3));
    let cur = null;
    pop.gens = gens.map((g) => g.gen);
    pop.colors = gens.map((g) => {
      const want = names.map((n) => (g.share[n] || 0) * N);
      const counts = want.map(Math.floor);
      let left = N - counts.reduce((a, b) => a + b, 0);
      want.map((w, i) => [w - Math.floor(w), i]).sort((a, b) => b[0] - a[0]).forEach(([, i]) => { if (left > 0) { counts[i]++; left--; } });
      if (!cur) {
        cur = []; let k = 0;
        names.forEach((n, i) => { for (let c = 0; c < counts[i]; c++) cur[order[k++]] = n; });
        while (k < N) cur[order[k++]] = names[0];
      } else {
        cur = [...cur];
        const have = names.map((n) => cur.filter((c) => c === n).length);
        const free = [];
        names.forEach((n, i) => {
          let extra = have[i] - counts[i];
          for (const idx of [...cur.keys()].filter((j) => cur[j] === n).sort((a, b) => hash(a * 13 + g.gen) - hash(b * 13 + g.gen))) { if (extra-- <= 0) break; free.push(idx); }
        });
        names.forEach((n, i) => { let need = counts[i] - have[i]; while (need-- > 0 && free.length) cur[free.shift()] = n; });
      }
      return cur;
    });
    pop.N = N;
  }
  const genIndex = (gen) => { let i = 0; pop.gens.forEach((g, j) => { if (g <= gen) i = j; }); return i; };

  function scenePopulation(s, t) {
    const p = s.params, a = genIndex(p.gen_from ?? pop.gens[0]), b = genIndex(p.gen_to ?? p.gen_from ?? pop.gens.at(-1));
    const steps = Math.max(0, b - a);
    const pos = steps ? lerp(0, steps, ease(phase(t, s.dur * 0.12, s.dur * 0.82))) : 0;
    const k = Math.min(b, a + Math.floor(pos)), f = pos - Math.floor(pos);
    const prevCols = pop.colors[k], nextCols = pop.colors[Math.min(b, k + 1)];
    const N = pop.N, cols = N <= 24 ? 6 : N <= 40 ? 8 : 10, rows = Math.ceil(N / cols);
    const gx = 150, gy = 250, cw = 1020 / cols, rh = Math.min(150, 610 / rows), r = Math.min(cw, rh) * 0.34;
    const gen = pop.gens[k + (f > 0.5 && k < b ? 1 : 0)];
    kicker(p.kicker || "The population", 1);
    text(`Generation ${gen}`, 96, 190, { size: 76, family: DISPLAY, weight: 600, color: C.parch });
    const stage = (P.run.stages || []).find((st) => gen >= st.from_gen && gen <= st.to_gen);
    if (stage) text(`stand-in stage: ${stage.name}`, 96 + 20, 232, { size: 26, color: C.muted, style: "italic" });
    for (let i = 0; i < N; i++) {
      const c0 = prevCols[i], c1 = nextCols[i], changing = c0 !== c1 && k < b;
      const localF = changing ? clamp((f - hash(i) * 0.35) / 0.5) : 0;
      const color = changing ? mix(colorOf(c0), colorOf(c1), localF) : colorOf(c0);
      const hl = p.archetype && (changing ? c1 : c0) !== p.archetype ? 0.35 : 1;
      const x = gx + cw * (i % cols) + cw / 2 + (Math.floor(i / cols) & 1 ? cw * 0.18 : 0), y = gy + rh * Math.floor(i / cols) + rh / 2 + 40;
      blob(x, y, r, color, { t, id: i, alpha: hl * ease(phase(t, 0.05 + (i % 7) * 0.04, 0.6 + (i % 7) * 0.04)), hop: changing ? Math.sin(Math.PI * localF) : 0, look: Math.sin(t * 0.7 + i) });
    }
    // Legend: live shares as bars.
    const g0 = P.run.gens[k], g1 = P.run.gens[Math.min(b, k + 1)];
    const names = P.run.archetypes.map((x) => x.name);
    const x0 = 1240, y0 = 300;
    text("Share of the population", x0, y0 - 40, { size: 26, color: C.muted, weight: 600, track: 2 });
    names.forEach((n, i) => {
      const v = lerp(g0.share[n] || 0, g1.share[n] || 0, k < b ? f : 0), y = y0 + i * 78;
      const dim = p.archetype && n !== p.archetype ? 0.4 : 1;
      ctx.save(); ctx.globalAlpha = dim;
      blob(x0 + 22, y + 18, 18, colorOf(n), { t, id: 100 + i });
      text(n, x0 + 60, y + 30, { size: 34, weight: 600, color: C.parch });
      ctx.fillStyle = "rgba(255,255,255,0.08)"; roundRect(x0 + 240, y + 6, 280, 28, 14); ctx.fill();
      ctx.fillStyle = colorOf(n); roundRect(x0 + 240, y + 6, Math.max(6, 280 * v), 28, 14); ctx.fill();
      text(pct(v), x0 + 540, y + 30, { size: 30, color: C.parch2 });
      ctx.restore();
    });
  }

  // ---------- graphs that draw themselves ----------
  function sceneGraph(s, t) {
    const p = s.params, metric = p.metric || "archetype_share";
    const all = P.run.gens.filter((g) => g.gen >= (p.gen_from ?? -Infinity) && g.gen <= (p.gen_to ?? Infinity));
    const gens = all.length ? all : P.run.gens;
    const L = 220, R = 1740, T = 300, B = 750;
    const first = gens[0].gen, last = Math.max(gens.at(-1).gen, first + 1);
    const x = (gen) => L + ((gen - first) / (last - first)) * (R - L);
    const reveal = ease(phase(t, s.dur * 0.08, s.dur * 0.8));
    const upto = lerp(first, last, reveal);
    const title = { archetype_share: "Which strategies the population uses", proxy_win_rate: "How often the Proxy wins", best_fitness: "Best fitness", mean_fitness: "Mean fitness" }[metric] || metric;
    kicker(p.kicker || "The numbers");
    text(p.heading || title, 96, 190, { size: 64, family: DISPLAY, weight: 600, color: C.parch });
    ctx.save(); ctx.globalAlpha = ease(phase(t, 0, 0.6));
    ctx.strokeStyle = "rgba(242,234,216,0.35)"; ctx.lineWidth = 2;
    ctx.beginPath(); ctx.moveTo(L, T); ctx.lineTo(L, B); ctx.lineTo(R, B); ctx.stroke();
    const span = last - first, step = [1, 2, 5, 10, 20, 50, 100, 200, 500].find((st) => span / st <= 10) || 1000;
    for (let v = Math.ceil(first / step) * step; v <= last; v += step) text(String(v), x(v), B + 44, { size: 28, color: C.muted, align: "center" });
    text("generation", R + 24, B + 10, { size: 26, color: C.muted, align: "left", style: "italic" });
    ctx.restore();
    ctx.save(); ctx.beginPath(); ctx.rect(0, 0, x(upto) + 1, H); ctx.clip();
    let yv, fmt;
    if (metric === "archetype_share") {
      yv = (v) => B - v * (B - T); fmt = pct;
      let base = gens.map(() => 0);
      for (const a of P.run.archetypes) {
        const top = gens.map((g, i) => base[i] + (g.share[a.name] || 0));
        ctx.beginPath();
        gens.forEach((g, i) => (i ? ctx.lineTo(x(g.gen), yv(top[i])) : ctx.moveTo(x(g.gen), yv(top[i]))));
        [...gens].reverse().forEach((g, j) => ctx.lineTo(x(g.gen), yv(base[gens.length - 1 - j])));
        ctx.closePath(); ctx.fillStyle = a.color; ctx.globalAlpha = p.archetype && a.name !== p.archetype ? 0.3 : 0.92; ctx.fill();
        base = top;
      }
      ctx.globalAlpha = 1;
    } else {
      const key = { proxy_win_rate: "pwr", best_fitness: "best", mean_fitness: "mean" }[metric] || "best";
      const vals = gens.map((g) => g[key]).filter((v) => v != null);
      const lo = metric === "proxy_win_rate" ? 0 : Math.min(0, ...vals), hi = metric === "proxy_win_rate" ? 1 : Math.max(...vals) * 1.1 || 1;
      yv = (v) => B - ((v - lo) / (hi - lo)) * (B - T); fmt = metric === "proxy_win_rate" ? pct : (v) => v.toFixed(2);
      const color = metric === "proxy_win_rate" ? C.blue : C.gold;
      ctx.strokeStyle = color; ctx.lineWidth = 6; ctx.lineJoin = "round"; ctx.lineCap = "round";
      ctx.beginPath(); let started = false;
      for (const g of gens) { if (g[key] == null) continue; started ? ctx.lineTo(x(g.gen), yv(g[key])) : ctx.moveTo(x(g.gen), yv(g[key])); started = true; }
      ctx.stroke();
      ctx.restore(); ctx.save();
      // pen dot and value at the tip
      const pts = gens.filter((g) => g[key] != null);
      const j = pts.findIndex((g) => g.gen > upto);
      const a = j <= 0 ? pts.at(j === 0 ? 0 : -1) : pts[j - 1], b = j <= 0 ? a : pts[j];
      const f = b.gen === a.gen ? 0 : (upto - a.gen) / (b.gen - a.gen);
      const tipGen = lerp(a.gen, b.gen, f), tipV = lerp(a[key], b[key], f);
      const shown = f < 0.5 ? a : b; // the label names a logged generation's value, never an interpolation
      ctx.fillStyle = color; ctx.beginPath(); ctx.arc(x(tipGen), yv(tipV), 12, 0, Math.PI * 2); ctx.fill();
      const right = x(tipGen) > R - 160, lx = right ? x(tipGen) - 22 : x(tipGen) + 22, al = right ? "right" : "left";
      text(`${fmt(shown[key])}`, lx, yv(tipV) - 18, { size: 36, weight: 600, color, align: al });
      text(`generation ${shown.gen}`, lx, yv(tipV) + 26, { size: 24, color: C.muted, align: al });
      for (let i = 0; i <= 4; i++) { const v = lo + ((hi - lo) * i) / 4; text(fmt(v), L - 20, yv(v) + 10, { size: 26, color: C.muted, align: "right" }); }
    }
    ctx.restore();
    if (metric === "archetype_share") {
      for (const v of [0, 0.5, 1]) text(pct(v), L - 20, yv(v) + 10, { size: 26, color: C.muted, align: "right" });
      // legend
      P.run.archetypes.forEach((a, i) => { const lx = L + i * 250; blob(lx + 14, 836, 14, a.color, { t, id: 200 + i }); text(a.name, lx + 40, 848, { size: 28, color: C.parch2 }); });
    }
    // Stand-in stage boundaries and event markers appear as the pen passes them.
    const marks = [
      ...(P.run.stages || []).slice(1).map((st) => ({ gen: st.from_gen - 0.5, label: st.name, color: C.muted })),
      ...(P.run.events || []).filter((e) => (p.events ?? ["archetype_takeover", "archetype_extinct"]).includes(e.kind)).map((e) => ({ gen: e.gen, label: e.short || e.kind, color: C.gold })),
    ].filter((m) => m.gen >= first && m.gen <= last).sort((a, b) => a.gen - b.gen).slice(0, 8);
    // Stack labels on up to three rows so neighbours never overlap.
    const rowEnd = [-Infinity, -Infinity, -Infinity];
    font(26, { style: "italic" });
    for (const m of marks) {
      const w = ctx.measureText(m.label).width, x0 = x(m.gen) + 10;
      m.row = rowEnd.findIndex((e) => x0 > e + 16);
      if (m.row < 0) m.row = rowEnd.indexOf(Math.min(...rowEnd));
      rowEnd[m.row] = x0 + w;
    }
    marks.forEach((m) => {
      const a = ease(phase(upto, m.gen, m.gen + (last - first) * 0.04));
      if (a <= 0) return;
      ctx.save(); ctx.globalAlpha = a; ctx.strokeStyle = m.color; ctx.setLineDash([8, 8]); ctx.lineWidth = 2;
      ctx.beginPath(); ctx.moveTo(x(m.gen), T - 88 + m.row * 30); ctx.lineTo(x(m.gen), B); ctx.stroke(); ctx.restore();
      text(m.label, x(m.gen) + 10, T - 62 + m.row * 30, { size: 26, color: m.color, alpha: a, style: "italic" });
    });
  }

  // ---------- maps from real tapes ----------
  const mapCache = new Map();
  function mapInfo(gameId) {
    if (mapCache.has(gameId)) return mapCache.get(gameId);
    const rp = P.replays[gameId];
    if (!rp) return null;
    const h = rp.header;
    const centers = h.terrain.map((_, i) => { const row = Math.floor(i / h.width), col = i % h.width; return [Math.sqrt(3) * (col + 0.5 * (row & 1)), 1.5 * row]; });
    const owners = new Map();
    const info = {
      rp, h, centers,
      bounds: [-1, -1.2, Math.sqrt(3) * (h.width + 0.5) + 1, 1.5 * h.height + 0.4],
      frame: (turn) => rp.frames[clamp(Math.round(turn) - 1, 0, rp.frames.length - 1)],
      owners: (fi) => { if (!owners.has(fi)) owners.set(fi, rp.frames[fi].owners.flatMap(([id, n]) => Array(n).fill(id))); return owners.get(fi); },
      at: ([q, r]) => centers[r * h.width + q + (r - (r & 1)) / 2],
      color: (id) => PLAYER[h.players.find((p) => p.id === id)?.color_index ?? 7],
    };
    mapCache.set(gameId, info);
    return info;
  }
  function focusPoint(m, turn, player) {
    const f = m.frame(turn);
    const pts = [...f.cities.filter((c) => c.owner === player).map((c) => m.at(c.pos)), ...(f.cities.some((c) => c.owner === player) ? [] : f.units.filter((u) => u.owner === player).map((u) => m.at(u.pos)))];
    if (!pts.length) return [(m.bounds[0] + m.bounds[2]) / 2, (m.bounds[1] + m.bounds[3]) / 2];
    return [pts.reduce((a, p) => a + p[0], 0) / pts.length, pts.reduce((a, p) => a + p[1], 0) / pts.length];
  }
  function drawMap(m, turn, view, cam, { focusPlayer = null, t = 0, labels = true } = {}) {
    const [vx, vy, vw, vh] = view;
    const fi = clamp(Math.round(turn) - 1, 0, m.rp.frames.length - 1), frame = m.rp.frames[fi], own = m.owners(fi);
    ctx.save();
    ctx.beginPath(); ctx.rect(vx, vy, vw, vh); ctx.clip();
    ctx.fillStyle = "#0b141e"; ctx.fillRect(vx, vy, vw, vh);
    const S = cam.scale, tx = (wx) => vx + vw / 2 + (wx - cam.x) * S, ty = (wy) => vy + vh / 2 + (wy - cam.y) * S;
    const corners = Array.from({ length: 6 }, (_, k) => { const a = Math.PI / 180 * (60 * k - 30); return [Math.cos(a), Math.sin(a)]; });
    const dirs = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];
    const { width, height, terrain, terrain_kinds } = m.h;
    for (let i = 0; i < terrain.length; i++) {
      const [wx, wy] = m.centers[i], x = tx(wx), y = ty(wy);
      if (x < vx - S * 2 || x > vx + vw + S * 2 || y < vy - S * 2 || y > vy + vh + S * 2) continue;
      ctx.beginPath(); corners.forEach(([cx, cy], k) => (k ? ctx.lineTo(x + cx * S, y + cy * S) : ctx.moveTo(x + cx * S, y + cy * S))); ctx.closePath();
      ctx.fillStyle = TERRAIN[terrain_kinds[terrain[i]]] || "#333"; ctx.fill();
      ctx.strokeStyle = "rgba(12,21,40,0.35)"; ctx.lineWidth = 0.6; ctx.stroke();
      const o = own[i];
      if (o >= 0) {
        ctx.globalAlpha = 0.34; ctx.fillStyle = m.color(o); ctx.fill(); ctx.globalAlpha = 1;
        const r = Math.floor(i / width), q = (i % width) - (r - (r & 1)) / 2;
        ctx.strokeStyle = m.color(o); ctx.lineWidth = Math.max(1.2, S * 0.13);
        for (let k = 0; k < 6; k++) {
          const nr = r + dirs[k][1], nq = q + dirs[k][0], nc = nq + (nr - (nr & 1)) / 2;
          if (nc >= 0 && nc < width && nr >= 0 && nr < height && own[nr * width + nc] === o) continue;
          ctx.beginPath(); ctx.moveTo(x + corners[k][0] * S, y + corners[k][1] * S); ctx.lineTo(x + corners[(k + 1) % 6][0] * S, y + corners[(k + 1) % 6][1] * S); ctx.stroke();
        }
      }
    }
    for (const u of frame.units) {
      if (u.strength <= 0) continue;
      const [wx, wy] = m.at(u.pos), size = Math.min(S * 0.42, S * (0.14 + Math.sqrt(u.strength) * 0.028));
      ctx.beginPath(); ctx.arc(tx(wx) + S * 0.36, ty(wy) - S * 0.26, Math.max(2, size), 0, Math.PI * 2);
      ctx.fillStyle = m.color(u.owner); ctx.fill(); ctx.strokeStyle = "#0b1017"; ctx.lineWidth = Math.max(1, S * 0.06); ctx.stroke();
    }
    for (const c of frame.cities) {
      const [wx, wy] = m.at(c.pos), x = tx(wx), y = ty(wy), size = Math.max(3, S * 0.34);
      ctx.beginPath();
      if (c.capital) { ctx.moveTo(x, y - size * 1.4); ctx.lineTo(x + size, y); ctx.lineTo(x, y + size * 1.4); ctx.lineTo(x - size, y); ctx.closePath(); } else ctx.arc(x, y, size, 0, Math.PI * 2);
      ctx.fillStyle = m.color(c.owner); ctx.fill(); ctx.strokeStyle = "#fff"; ctx.lineWidth = Math.max(1, S * 0.07); ctx.stroke();
      if (focusPlayer != null && c.owner === focusPlayer) {
        const pr = size * (2.2 + 0.5 * Math.sin(t * 3));
        ctx.strokeStyle = C.goldHi; ctx.globalAlpha = 0.8; ctx.lineWidth = 3; ctx.beginPath(); ctx.arc(x, y, pr, 0, Math.PI * 2); ctx.stroke(); ctx.globalAlpha = 1;
      }
      if (labels && S > 16) {
        const fs = Math.min(26, S * 0.55);
        font(fs, { weight: 600 }); ctx.textAlign = "center"; ctx.lineWidth = 4; ctx.strokeStyle = "#0b1017";
        ctx.strokeText(c.name, x, y + size + fs + 2); ctx.fillStyle = "#f0eee6"; ctx.fillText(c.name, x, y + size + fs + 2);
      }
    }
    ctx.restore();
    ctx.strokeStyle = C.rule; ctx.lineWidth = 2; ctx.strokeRect(vx, vy, vw, vh);
    return frame;
  }
  function fitCam(m, view) {
    const [x0, y0, x1, y1] = m.bounds;
    const scale = Math.min(view[2] / (x1 - x0), view[3] / (y1 - y0));
    return { x: (x0 + x1) / 2, y: (y0 + y1) / 2, scale };
  }
  function standings(m, frame, x, y, { max = 4, focus = null, alpha = 1 } = {}) {
    const rows = [...frame.vp].sort((a, b) => b.total - a.total || a.player - b.player).slice(0, max);
    ctx.save(); ctx.globalAlpha = alpha;
    ctx.fillStyle = "rgba(6,10,26,0.8)"; roundRect(x, y, 420, 50 + rows.length * 50, 12); ctx.fill(); ctx.strokeStyle = C.rule; ctx.stroke();
    text("VICTORY POINTS", x + 22, y + 36, { size: 20, color: C.gold, weight: 600, track: 3 });
    rows.forEach((v, i) => {
      const pl = m.h.players.find((p) => p.id === v.player), yy = y + 82 + i * 50;
      ctx.fillStyle = m.color(v.player); ctx.beginPath(); ctx.arc(x + 32, yy - 9, 10, 0, Math.PI * 2); ctx.fill();
      text(`${pl.faction}${v.player === focus ? "  ◂" : ""}${v.alive ? "" : " (out)"}`, x + 54, yy, { size: 28, color: v.player === focus ? C.goldHi : C.parch, weight: v.player === focus ? 600 : 400 });
      text(String(v.total), x + 398, yy, { size: 30, color: C.parch, align: "right", weight: 600 });
    });
    ctx.restore();
  }
  function sceneMap(s, t) {
    const p = s.params, m = mapInfo(p.game_id);
    if (!m) return missing(`Tape ${p.game_id} is not available`);
    const n = m.rp.frames.length;
    const t0 = clamp(p.turn_from ?? p.turn_to ?? 1, 1, n), t1 = clamp(p.turn_to ?? t0, 1, n);
    const turn = Math.round(lerp(t0, t1, ease(phase(t, s.dur * 0.08, s.dur * 0.62))));
    const view = [0, 0, W, H];
    const fit = fitCam(m, view);
    const focus = focusPoint(m, t1, p.focus_player ?? -1);
    const z = p.focus_player != null ? (p.zoom ?? 2.2) : 1;
    const k = ease(phase(t, s.dur * 0.18, s.dur * 0.7));
    const cam = { x: lerp(fit.x, focus[0], k * (z > 1 ? 1 : 0)), y: lerp(fit.y, focus[1], k * (z > 1 ? 1 : 0)), scale: fit.scale * lerp(1, z, k) };
    const frame = drawMap(m, turn, view, cam, { focusPlayer: p.focus_player, t });
    // vignette + header card
    const g = ctx.createLinearGradient(0, 0, 0, 300); g.addColorStop(0, "rgba(10,17,40,0.97)"); g.addColorStop(0.62, "rgba(10,17,40,0.9)"); g.addColorStop(1, "rgba(10,17,40,0)"); ctx.fillStyle = g; ctx.fillRect(0, 0, W, 300);
    kicker(p.kicker || (p.gen != null ? `Generation ${p.gen}` : "A real game"));
    text(p.heading || `Turn ${turn}`, 96, 176, { size: 60, family: DISPLAY, weight: 600, color: C.parch });
    text(`Turn ${turn} of ${m.h.turns} · seed ${m.h.seed} · ${p.game_id}`, W - 96, 104, { size: 22, color: C.muted, align: "right" });
    const fp = m.h.players.find((x) => x.id === p.focus_player);
    if (fp) text(`Watching ${fp.faction} (seat ${fp.id + 1})`, W - 96, 142, { size: 28, color: C.goldHi, align: "right", weight: 600 });
    standings(m, frame, W - 96 - 420, 190, { focus: p.focus_player, alpha: ease(phase(t, 0.6, 1.4)) });
  }
  function sceneVersus(s, t) {
    const p = s.params, A = mapInfo(p.game_id), B = mapInfo(p.game_id_b);
    if (!A || !B) return missing("A tape for the side-by-side is not available");
    const na = Math.min(A.rp.frames.length, B.rp.frames.length);
    const t0 = clamp(p.turn_from ?? 1, 1, na), t1 = clamp(p.turn_to ?? na, 1, Math.max(A.rp.frames.length, B.rp.frames.length));
    const turn = Math.round(lerp(t0, t1, ease(phase(t, s.dur * 0.1, s.dur * 0.75))));
    kicker(p.kicker || "Same map, same seed");
    text(p.heading || "Before and after", 96, 176, { size: 60, family: DISPLAY, weight: 600, color: C.parch });
    const views = [[60, 280, 880, 580], [980, 280, 880, 580]];
    [[A, p.label_a || "Before", p.focus_player], [B, p.label_b || "After", p.focus_player_b ?? p.focus_player]].forEach(([m, label, fpl], i) => {
      const v = views[i], cam = fitCam(m, v);
      const frame = drawMap(m, Math.min(turn, m.rp.frames.length), v, cam, { focusPlayer: fpl, t, labels: false });
      text(label, v[0], v[1] - 20, { size: 34, color: C.gold, weight: 600 });
      const rows = [...frame.vp].sort((a, b) => b.total - a.total).slice(0, 3);
      ctx.fillStyle = "rgba(6,10,26,0.82)"; roundRect(v[0] + 12, v[1] + 12, 330, 24 + rows.length * 40, 10); ctx.fill();
      rows.forEach((r, j) => {
        const pl = m.h.players.find((x) => x.id === r.player), yy = v[1] + 50 + j * 40;
        ctx.fillStyle = m.color(r.player); ctx.beginPath(); ctx.arc(v[0] + 34, yy - 9, 9, 0, Math.PI * 2); ctx.fill();
        text(`${pl.faction}${r.player === fpl ? " ◂" : ""}`, v[0] + 54, yy, { size: 26, color: r.player === fpl ? C.goldHi : C.parch2 });
        text(`${r.total} VP`, v[0] + 326, yy, { size: 26, color: C.parch, align: "right" });
      });
      text(`Turn ${Math.min(turn, m.rp.frames.length)}`, v[0] + v[2], v[1] - 18, { size: 26, color: C.muted, align: "right" });
    });
  }

  // ---------- cards ----------
  function sceneTitle(s, t) {
    const p = s.params;
    const a = ease(phase(t, 0.2, 1.4));
    drawWordmark(222, 360, a);
    const lines = wrap(p.heading || "What the AI learned", 1500, 110, { family: DISPLAY, weight: 600 });
    lines.forEach((l, i) => text(l, W / 2, 440 + i * 120, { size: 110, family: DISPLAY, weight: 600, align: "center", color: C.parch, alpha: ease(phase(t, 0.5, 1.8)) }));
    if (p.body) text(p.body, W / 2, 470 + lines.length * 120, { size: 40, family: DISPLAY, style: "italic", weight: 500, align: "center", color: C.parch2, alpha: ease(phase(t, 1.2, 2.4)) });
    const names = P.run.archetypes.map((x) => x.name);
    for (let i = 0; i < 9; i++) blob(W / 2 + (i - 4) * 120, 850, 34, colorOf(names[i % names.length]), { t, id: 300 + i, alpha: ease(phase(t, 1 + i * 0.12, 1.8 + i * 0.12)), hop: Math.max(0, Math.sin(t * 2.4 - i * 0.5)) * 0.35 });
  }
  function sceneText(s, t) {
    const p = s.params;
    kicker(p.kicker || "", ease(phase(t, 0, 0.6)));
    const head = wrap(p.heading || "", 1300, 84, { family: DISPLAY, weight: 600 });
    head.forEach((l, i) => text(l, 96, 250 + i * 96, { size: 84, family: DISPLAY, weight: 600, color: C.parch, alpha: ease(phase(t, 0.2, 1)) }));
    const paras = String(p.body || "").split(/\n+/).filter(Boolean);
    let y = 250 + head.length * 96 + 60;
    paras.forEach((para, j) => {
      const lines = wrap(para, 1300, 44);
      const a = ease(phase(t, 0.8 + j * 0.9, 1.6 + j * 0.9));
      lines.forEach((l) => { text(l, 100, y, { size: 44, color: C.parch2, alpha: a }); y += 60; });
      y += 24;
    });
    // A small audience of blobs on the right, looking at the text: the cast stays on stage.
    const names = P.run.archetypes.map((x) => x.name);
    const only = p.archetype && names.includes(p.archetype) ? p.archetype : null;
    [[1560, 690, 44], [1700, 640, 36], [1650, 780, 52], [1790, 740, 40], [1500, 820, 34]].forEach(([bx, by, r], i) => {
      blob(bx, by - 230, r, colorOf(only || names[(i * 2) % names.length]), { t, id: 500 + i, alpha: 0.9 * ease(phase(t, 0.6 + i * 0.15, 1.4 + i * 0.15)), look: -1 });
    });
  }
  function sceneStat(s, t) {
    const p = s.params, items = p.items || [];
    kicker(p.kicker || "Before and after");
    text(p.heading || "The benchmark", 96, 190, { size: 64, family: DISPLAY, weight: 600, color: C.parch });
    items.forEach((it, i) => {
      const y = 340 + i * 250, a = ease(phase(t, 0.4 + i * 0.8, 1.2 + i * 0.8)), b = ease(phase(t, 1.4 + i * 0.8, 2.4 + i * 0.8));
      text(it.label, 110, y, { size: 36, color: C.muted, alpha: a });
      text(it.before, 110, y + 110, { size: 104, family: SANS, weight: 600, color: C.parch2, alpha: a });
      text(it.before_label || "", 114, y + 160, { size: 26, color: C.muted, alpha: a });
      text("→", 620, y + 100, { size: 90, color: C.gold, alpha: b });
      text(it.after, 780, y + 110, { size: 104, family: SANS, weight: 600, color: C.goldHi, alpha: b });
      text(it.after_label || "", 784, y + 160, { size: 26, color: C.muted, alpha: b });
    });
  }
  function sceneMatrix(s, t) {
    const p = s.params, cp = (P.run.checkpoints || []).find((c) => c.gen === p.checkpoint_gen) || (P.run.checkpoints || []).filter((c) => c.matrix).at(-1);
    kicker(p.kicker || "Who beats whom");
    text(p.heading || `Matchups at generation ${cp?.gen ?? "–"}`, 96, 190, { size: 64, family: DISPLAY, weight: 600, color: C.parch });
    if (!cp?.matrix) return missing("No matchup matrix in the log");
    const { names, rate } = cp.matrix, n = names.length, cell = Math.min(110, 620 / n), x0 = 520, y0 = 300;
    names.forEach((nm, i) => {
      text(nm, x0 - 24, y0 + cell * i + cell / 2 + 12, { size: 30, align: "right", color: colorOf(nm), weight: 600, alpha: ease(phase(t, 0.3, 1)) });
      ctx.save(); ctx.translate(x0 + cell * i + cell / 2, y0 - 16); ctx.rotate(-Math.PI / 5); text(nm, 0, 0, { size: 26, color: colorOf(nm), weight: 600, alpha: ease(phase(t, 0.3, 1)) }); ctx.restore();
    });
    for (let i = 0; i < n; i++) for (let j = 0; j < n; j++) {
      const v = rate[i][j], a = ease(phase(t, 0.8 + (i + j) * 0.12, 1.4 + (i + j) * 0.12));
      ctx.save(); ctx.globalAlpha = a;
      ctx.fillStyle = v == null ? "rgba(255,255,255,0.04)" : v >= 0.5 ? `rgba(143,207,114,${0.15 + (v - 0.5) * 1.5})` : `rgba(239,127,106,${0.15 + (0.5 - v) * 1.5})`;
      ctx.fillRect(x0 + cell * j + 3, y0 + cell * i + 3, cell - 6, cell - 6);
      ctx.restore();
      if (v != null) text(pct(v), x0 + cell * j + cell / 2, y0 + cell * i + cell / 2 + 11, { size: 28, align: "center", color: C.parch, alpha: a });
    }
    text("Row finishes ahead of column. Green: the row usually wins.", x0, y0 + cell * n + 60, { size: 30, color: C.muted, style: "italic" });
  }
  function sceneEnd(s, t) {
    const p = s.params, a = ease(phase(t, 0.2, 1.2));
    drawWordmark(108, 300, a);
    text(p.heading || "Made from the run log", W / 2, 260, { size: 76, family: DISPLAY, weight: 600, align: "center", color: C.parch, alpha: a });
    const lines = p.lines || [];
    lines.forEach((l, i) => text(l, W / 2, 380 + i * 62, { size: i === 0 ? 36 : 32, align: "center", color: i === 0 ? C.gold : C.parch2, alpha: ease(phase(t, 0.6 + i * 0.25, 1.4 + i * 0.25)) }));
    const names = P.run.archetypes.map((x) => x.name);
    for (let i = 0; i < 5; i++) blob(W / 2 + (i - 2) * 110, 960, 30, colorOf(names[i % names.length]), { t, id: 400 + i, alpha: a });
  }
  function missing(msg) { text(msg, W / 2, H / 2, { size: 44, color: C.bad, align: "center" }); }

  const SCENES = { title: sceneTitle, text: sceneText, population: scenePopulation, graph: sceneGraph, map: sceneMap, zoom: sceneMap, versus: sceneVersus, matrix: sceneMatrix, stat: sceneStat, end: sceneEnd };

  window.FILM = {
    async init(payload) {
      P = payload;
      const b = payload.brand;
      if (!b || !payload.wordmark) throw new Error("Film requires the editorial brand payload");
      C = { navy: b.bg, navy2: b.panel, navy3: b["panel-2"], gold: b.accent, goldHi: b["accent-hi"],
        parch: b.ink, parch2: b["ink-2"], muted: b.muted, rule: b["rule-2"], bad: b.bad, good: b.good, blue: b["y-science"] };
      DISPLAY = b.display; SANS = b.sans;
      document.documentElement.style.background = b.bg;
      wordmark = new Image();
      wordmark.src = payload.wordmark;
      await wordmark.decode();
      for (const f of payload.fonts || []) {
        const face = new FontFace(f.family, `url(data:font/woff2;base64,${f.data})`, { weight: String(f.weight), style: f.style || "normal" });
        document.fonts.add(await face.load());
      }
      await document.fonts.ready;
      preparePopulation();
      return { fonts: [...document.fonts].filter((f) => f.status === "loaded").length };
    },
    draw(index, t) {
      const s = P.scenes[index];
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.globalAlpha = 1;
      background(t);
      (SCENES[s.type] || ((x) => missing(`Unknown scene ${x.type}`)))(s, t);
      captions(s, t);
      // soft fade in and out at scene edges
      const edge = Math.min(phase(t, 0, 0.35), 1 - phase(t, s.dur - 0.35, s.dur));
      if (edge < 1) { ctx.fillStyle = `rgba(10,17,40,${1 - edge})`; ctx.fillRect(0, 0, W, H); }
    },
    frameJpeg(index, t, quality = 0.9) {
      this.draw(index, t);
      return canvas.toDataURL("image/jpeg", quality).slice("data:image/jpeg;base64,".length);
    },
  };
})();
