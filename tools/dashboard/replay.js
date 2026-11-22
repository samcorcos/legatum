/* Isolated replay viewer: no dependencies and no coupling to the dashboard's refresh loop. */
(() => {
  "use strict";
  const el = (id) => document.getElementById(`replay-${id}`);
  const section = document.getElementById("replays");
  if (!section) return;
  // Okabe–Ito colors, with light gray in place of black on the dark background.
  // Player numbers, labels and chart dash patterns also distinguish empires without color.
  const COLORS = ["#E69F00", "#56B4E9", "#009E73", "#F0E442", "#0072B2", "#D55E00", "#CC79A7", "#D4D8DD"];
  const TERRAIN = { Ocean: "#172c40", Coast: "#254354", Grassland: "#47584a", Plains: "#625f48", Forest: "#2d463e", Hills: "#626052", Desert: "#7c7155", Tundra: "#667477", Mountains: "#484d56" };
  const DASHES = ["", "7 3", "2 3", "10 3 2 3", "12 5", "4 2", "1 4", "8 2 2 2"];
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const human = (s) => String(s).replace(/([a-z])([A-Z])/g, "$1 $2").replaceAll("_", " ");
  const outcome = (s) => ({ vp: "VP victory", conquest: "Conquest", turn_limit: "Turn limit", draw: "Draw", unfinished: "Unfinished tape" })[s] || human(s);
  let archive = [], game = null, frameIndex = 0, timer = null, request = 0, indexRequest = 0;
  let players = new Map(), owners = [], centers = [], radius = 1, eventButtons = [], chartX = () => 0;
  const player = (id) => players.get(id);
  const color = (id) => COLORS[player(id)?.color_index ?? 7];
  const name = (id) => player(id)?.name ?? "Unclaimed";
  const swatch = (id) => `<i class="replay-swatch" style="background:${color(id)}"></i>`;

  // Deep links (#replay=<file>&turn=<n>) come from the Evolution panel's "watch" buttons.
  let pendingTurn = null;
  const hashTarget = () => {
    const m = /^#replay=([^&]+)(?:&turn=(\d+))?/.exec(location.hash);
    return m ? { file: decodeURIComponent(m[1]), turn: Math.max(1, Number(m[2] || 1)) } : null;
  };
  function followHash() {
    const target = hashTarget();
    if (!target || !archive.some((r) => r.file === target.file)) return false;
    section.scrollIntoView({ block: "start" });
    if (game && el("game").value === target.file) { jump(target.turn - 1); return true; }
    el("game").value = target.file;
    pendingTurn = target.turn;
    loadGame();
    return true;
  }

  async function json(url) {
    const response = await fetch(url, { cache: "no-store" });
    if (!response.ok) throw new Error(`Could not load ${url} (HTTP ${response.status})`);
    return response.json();
  }

  function validate(data) {
    const h = data?.header;
    if (data?.version !== 1 || !h || !Array.isArray(data.frames) || !data.frames.length ||
        data.frames.length !== h.turns || !Number.isInteger(h.width) || !Number.isInteger(h.height) ||
        h.width <= 0 || h.height <= 0 || h.width * h.height > 100000 ||
        h.terrain?.length !== h.width * h.height || !h.players?.length || !h.targets) {
      throw new Error("Unsupported or empty replay. Export a tape with resolved turns.");
    }
    const ids = new Set(h.players.map((p) => p.id));
    const posValid = ([q, r]) => {
      const col = q + (r - (r & 1)) / 2;
      return Number.isInteger(q) && Number.isInteger(r) && col >= 0 && col < h.width && r >= 0 && r < h.height;
    };
    if (ids.size !== h.players.length || h.players.some((p) => !Number.isInteger(p.color_index) || p.color_index < 0 || p.color_index >= COLORS.length) ||
        !h.terrain.every((t) => Number.isInteger(t) && h.terrain_kinds[t] in TERRAIN)) throw new Error("Invalid replay map or players.");
    for (const [i, f] of data.frames.entries()) {
      if (f.turn !== i + 1 || f.owners.reduce((n, [owner, count]) => {
        if ((owner !== -1 && !ids.has(owner)) || !Number.isInteger(count) || count <= 0) throw new Error("Invalid territory run.");
        return n + count;
      }, 0) !== h.terrain.length || f.vp.length !== ids.size ||
          !f.vp.every((v) => ids.has(v.player) && Number.isFinite(v.total) && v.total >= 0) ||
          ![...f.cities, ...f.units].every((v) => ids.has(v.owner) && posValid(v.pos)) ||
          !f.events.every((e) => e.turn === f.turn && typeof e.text === "string")) throw new Error(`Invalid replay frame ${i + 1}.`);
    }
  }

  async function loadIndex() {
    const token = ++indexRequest;
    el("refresh").disabled = true;
    try {
      const entries = await json("/replays/index.json");
      if (token !== indexRequest) return;
      if (!Array.isArray(entries) || entries.some((r) => !/^[A-Za-z0-9_-]+\.json$/.test(r.file))) throw new Error("Invalid replay index.");
      const previous = el("game").value;
      archive = entries;
      el("count").textContent = `${archive.length} games`;
      el("game").replaceChildren(...archive.map((r) => new Option(`${r.label} · seed ${r.seed} · ${outcome(r.outcome)} · ${r.why_interesting}`, r.file)));
      el("game").disabled = !archive.length;
      if (!archive.length) {
        ++request;
        pause(); game = null;
        el("game").add(new Option("No replays yet", ""));
        el("content").hidden = true;
        el("status").textContent = "No games in the archive yet. Generate a simulation run with replay exports, then refresh games.";
      } else {
        if (archive.some((r) => r.file === previous)) el("game").value = previous;
        if (!followHash()) await loadGame();
      }
    } catch (error) {
      if (token === indexRequest) el("status").textContent = `Replay archive unavailable: ${error.message}`;
    } finally {
      if (token === indexRequest) el("refresh").disabled = false;
    }
  }

  async function loadGame() {
    const entry = archive.find((r) => r.file === el("game").value);
    if (!entry) return;
    const token = ++request;
    pause(); game = null;
    el("content").hidden = true;
    el("status").textContent = "Loading game…";
    try {
      const data = await json(`/replays/${encodeURIComponent(entry.file)}`);
      if (token !== request) return;
      validate(data);
      game = data;
      players = new Map(game.header.players.map((p) => [p.id, p]));
      const h = game.header;
      el("title").textContent = `${h.label} · seed ${h.seed}`;
      const winners = h.players.filter((p) => p.team === h.winner).map((p) => p.name).join(" & ");
      el("result").textContent = `${outcome(h.outcome)}${winners ? ` · ${winners} won` : ""} · ${h.turns} turns`;
      el("why").textContent = entry.why_interesting;
      el("timeline").max = game.frames.length - 1;
      el("map").style.aspectRatio = `${Math.sqrt(3) * (h.width + .5) + 2} / ${1.5 * h.height + 2.5}`;
      el("content").hidden = false;
      el("status").textContent = "";
      drawChart();
      buildEvents();
      setFrame(pendingTurn ? pendingTurn - 1 : 0);
      pendingTurn = null;
    } catch (error) {
      if (token !== request) return;
      game = null;
      el("content").hidden = true;
      el("status").textContent = `Could not open this replay: ${error.message}`;
    }
  }

  function pause() {
    clearInterval(timer); timer = null;
    el("play").textContent = "Play";
    el("play").setAttribute("aria-label", "Play replay");
    el("play").setAttribute("aria-pressed", "false");
  }
  function play() {
    if (!game) return;
    if (timer) return pause();
    if (frameIndex === game.frames.length - 1) setFrame(0);
    el("play").textContent = "Pause";
    el("play").setAttribute("aria-label", "Pause replay");
    el("play").setAttribute("aria-pressed", "true");
    timer = setInterval(() => {
      setFrame(frameIndex + 1);
      if (frameIndex === game.frames.length - 1) pause();
    }, 850 / Number(el("speed").value));
  }
  function jump(index) { pause(); setFrame(index); }

  function setFrame(index) {
    if (!game) return;
    frameIndex = Math.max(0, Math.min(game.frames.length - 1, index));
    const frame = game.frames[frameIndex];
    owners = frame.owners.flatMap(([id, count]) => Array(count).fill(id));
    el("timeline").value = frameIndex;
    el("timeline").setAttribute("aria-valuetext", `Turn ${frame.turn} of ${game.header.turns}`);
    el("turn").textContent = `Turn ${frame.turn} / ${game.header.turns}`;
    el("first").disabled = el("prev").disabled = frameIndex === 0;
    el("last").disabled = el("next").disabled = frameIndex === game.frames.length - 1;
    drawMap();
    standings(frame);
    const cursor = el("cursor");
    cursor.setAttribute("x1", chartX(frame.turn)); cursor.setAttribute("x2", chartX(frame.turn));
    el("cursor-label").setAttribute("x", chartX(frame.turn));
    el("cursor-label").textContent = `T${frame.turn}`;
    let first = null;
    for (const button of eventButtons) {
      const current = Number(button.dataset.turn) === frame.turn;
      if (current) { button.setAttribute("aria-current", "true"); first ??= button; }
      else button.removeAttribute("aria-current");
    }
    if (first) {
      const list = el("events");
      list.scrollTop += first.getBoundingClientRect().top - list.getBoundingClientRect().top;
    }
    el("inspect").textContent = "Hover a tile for terrain, city, and army details.";
  }

  function standings(frame) {
    const expanded = new Set([...el("standings").querySelectorAll("details[open]")].map((d) => d.dataset.player));
    el("standings").innerHTML = [...frame.vp].sort((a, b) => b.total - a.total || a.player - b.player).map((v) => {
      const p = player(v.player), cities = frame.cities.filter((c) => c.owner === p.id);
      return `<li class="${v.alive ? "" : "replay-eliminated"}"><div class="replay-score-row">${swatch(p.id)}<span>${esc(p.name)}<small class="replay-faction">${esc(p.faction)} · team ${p.team + 1}${v.alive ? ` · ${cities.length} cities` : " · eliminated"}</small></span><b>${v.total}</b></div><details data-player="${p.id}" ${expanded.has(String(p.id)) ? "open" : ""}><summary>VP sources</summary>${Object.entries(v.sources).map(([s, n]) => `${esc(s)}: ${n}`).join("<br>") || "No VP yet"}</details></li>`;
    }).join("");
    el("titles").innerHTML = Object.entries(frame.titles).map(([title, id]) => `<div><span>${esc(human(title))}</span><span style="color:${color(id)}">${esc(name(id))}</span></div>`).join("");
  }

  function drawMap() {
    if (!game) return;
    const brand = getComputedStyle(section);
    const canvas = el("map"), rect = canvas.getBoundingClientRect(), dpr = window.devicePixelRatio || 1;
    if (!rect.width || !rect.height) return;
    canvas.width = Math.round(rect.width * dpr); canvas.height = Math.round(rect.height * dpr);
    const ctx = canvas.getContext("2d");
    ctx.scale(dpr, dpr);
    const { width, height, terrain, terrain_kinds } = game.header;
    radius = Math.min(rect.width / (Math.sqrt(3) * (width + .5) + 2), rect.height / (1.5 * height + 2.5));
    const hexWidth = Math.sqrt(3) * radius, pad = 1.8 * radius;
    centers = terrain.map((_, i) => [pad + hexWidth * (i % width + .5 * (Math.floor(i / width) & 1)), pad + radius * 1.5 * Math.floor(i / width)]);
    const corners = Array.from({ length: 6 }, (_, i) => { const angle = Math.PI / 180 * (60 * i - 30); return [radius * Math.cos(angle), radius * Math.sin(angle)]; });
    const directions = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];
    for (let i = 0; i < terrain.length; i++) {
      const [x, y] = centers[i], owner = owners[i], r = Math.floor(i / width), q = i % width - (r - (r & 1)) / 2;
      ctx.beginPath(); corners.forEach(([dx, dy], j) => j ? ctx.lineTo(x + dx, y + dy) : ctx.moveTo(x + dx, y + dy)); ctx.closePath();
      ctx.fillStyle = TERRAIN[terrain_kinds[terrain[i]]]; ctx.fill();
      ctx.strokeStyle = "#0c152840"; ctx.lineWidth = .5; ctx.stroke();
      if (owner >= 0) {
        ctx.globalAlpha = .33; ctx.fillStyle = color(owner); ctx.fill(); ctx.globalAlpha = 1;
        ctx.strokeStyle = color(owner); ctx.lineWidth = Math.max(1, radius * .12);
        for (let j = 0; j < 6; j++) {
          const nr = r + directions[j][1], nq = q + directions[j][0], nc = nq + (nr - (nr & 1)) / 2;
          if (nc >= 0 && nc < width && nr >= 0 && nr < height && owners[nr * width + nc] === owner) continue;
          ctx.beginPath(); ctx.moveTo(x + corners[j][0], y + corners[j][1]); ctx.lineTo(x + corners[(j + 1) % 6][0], y + corners[(j + 1) % 6][1]); ctx.stroke();
        }
      }
    }
    const center = ([q, r]) => centers[r * width + q + (r - (r & 1)) / 2];
    const frame = game.frames[frameIndex];
    for (const army of frame.units) {
      if (army.strength <= 0) continue;
      const [x, y] = center(army.pos), size = Math.min(radius * .4, 1.2 + Math.sqrt(army.strength) * .35);
      ctx.beginPath(); ctx.arc(x + radius * .35, y - radius * .25, size, 0, Math.PI * 2);
      ctx.fillStyle = color(army.owner); ctx.fill(); ctx.strokeStyle = brand.getPropertyValue("--bg").trim(); ctx.lineWidth = 1.2; ctx.stroke();
    }
    for (const city of frame.cities) {
      const [x, y] = center(city.pos), size = Math.max(2, radius * .32);
      ctx.beginPath();
      if (city.capital) { ctx.moveTo(x, y - size * 1.4); ctx.lineTo(x + size, y); ctx.lineTo(x, y + size * 1.4); ctx.lineTo(x - size, y); ctx.closePath(); }
      else ctx.arc(x, y, size, 0, Math.PI * 2);
      ctx.fillStyle = color(city.owner); ctx.fill(); ctx.strokeStyle = "#fff"; ctx.lineWidth = 1; ctx.stroke();
    }
    if (el("labels").checked) {
      ctx.font = `600 ${Math.max(7, Math.min(11, radius))}px ${brand.fontFamily}`;
      ctx.textAlign = "center"; ctx.textBaseline = "top";
      for (const city of frame.cities) {
        const [cx, cy] = center(city.pos), label = `${city.owner + 1}· ${city.name}`;
        const half = ctx.measureText(label).width / 2;
        const x = Math.max(half + 2, Math.min(rect.width - half - 2, cx)), y = Math.min(rect.height - 12, cy + radius * .6);
        ctx.lineWidth = 3; ctx.strokeStyle = brand.getPropertyValue("--bg").trim(); ctx.strokeText(label, x, y); ctx.fillStyle = brand.getPropertyValue("--ink").trim(); ctx.fillText(label, x, y);
      }
    }
    canvas.setAttribute("aria-label", `Turn ${frame.turn}: ${frame.cities.length} cities and ${frame.units.length} unit groups. ${[...frame.vp].sort((a, b) => b.total - a.total).map((v) => `${name(v.player)} ${v.total} VP`).join(", ")}`);
  }

  function drawChart() {
    const W = 800, H = 260, L = 35, R = 25, T = 28, B = 30;
    const targets = [...new Set(Object.values(game.header.targets))].sort((a, b) => a - b);
    const max = Math.max(1, ...targets, ...game.frames.flatMap((f) => f.vp.map((v) => v.total))) + 1;
    chartX = (turn) => L + ((turn - 1) / Math.max(1, game.header.turns - 1)) * (W - L - R);
    const y = (vp) => H - B - vp / max * (H - T - B);
    let svg = "";
    for (let v = 0; v <= max; v += Math.max(1, Math.ceil(max / 6))) svg += `<line x1="${L}" x2="${W - R}" y1="${y(v)}" y2="${y(v)}" stroke="var(--rule-2)"/><text x="${L - 8}" y="${y(v) + 4}" text-anchor="end" fill="var(--muted)" font-size="10">${v}</text>`;
    for (const turn of [...new Set([1, Math.round(game.header.turns / 2), game.header.turns])]) svg += `<text x="${chartX(turn)}" y="${H - 8}" text-anchor="middle" fill="var(--muted)" font-size="10">T${turn}</text>`;
    for (const target of targets) svg += `<line data-target="${target}" x1="${L}" x2="${W - R}" y1="${y(target)}" y2="${y(target)}" stroke="var(--accent)" stroke-dasharray="5 5"/><text x="${W - R}" y="${y(target) - 5}" text-anchor="end" fill="var(--accent)" font-size="10">Target ${target}</text>`;
    for (const p of players.values()) {
      const points = game.frames.map((f) => `${chartX(f.turn)},${y(f.vp.find((v) => v.player === p.id).total)}`).join(" ");
      svg += `<polyline data-player="${p.id}" points="${points}" fill="none" stroke="${color(p.id)}" stroke-width="2" stroke-dasharray="${DASHES[p.color_index]}"><title>${esc(p.name)}</title></polyline>`;
    }
    svg += `<line id="replay-cursor" y1="${T}" y2="${H - B}" stroke="#f2f0e5" stroke-width="1.5"/><text id="replay-cursor-label" y="14" text-anchor="middle" fill="#f2f0e5" font-size="10"></text>`;
    el("vp").innerHTML = svg;
    el("target-note").textContent = `Team target: ${targets.join(" / ")} VP`;
    el("legend").innerHTML = [...players.values()].map((p) => `<span>${swatch(p.id)}${p.id + 1}· ${esc(p.name)}</span>`).join("");
  }

  function buildEvents() {
    const events = game.frames.flatMap((f) => f.events);
    el("event-count").textContent = `${events.length} events`;
    el("events").innerHTML = events.length ? events.map((e) => `<button type="button" data-turn="${e.turn}"><span class="replay-event-turn">T${e.turn}</span><span>${esc(e.text)}</span></button>`).join("") : '<p class="empty">No notable events in this tape.</p>';
    eventButtons = [...el("events").querySelectorAll("button")];
  }

  el("map").addEventListener("pointermove", (event) => {
    if (!game) return;
    const rect = el("map").getBoundingClientRect(), x = event.clientX - rect.left, y = event.clientY - rect.top;
    let nearest = -1, distance = radius * radius;
    centers.forEach(([cx, cy], i) => { const d = (cx - x) ** 2 + (cy - y) ** 2; if (d < distance) { nearest = i; distance = d; } });
    if (nearest < 0) return;
    const h = game.header, r = Math.floor(nearest / h.width), q = nearest % h.width - (r - (r & 1)) / 2;
    const frame = game.frames[frameIndex], at = (v) => v.pos[0] === q && v.pos[1] === r;
    const city = frame.cities.find(at), armies = frame.units.filter(at);
    el("inspect").textContent = [`(${q}, ${r}) ${h.terrain_kinds[h.terrain[nearest]]}`, owners[nearest] >= 0 ? name(owners[nearest]) : "Unclaimed", city ? `${city.name} · population ${city.pop}${city.capital ? " · capital" : ""}` : "", ...armies.map((a) => `${name(a.owner)}: ${Object.entries(a.kinds).map(([k, n]) => `${n} ${k}`).join(", ")} · strength ${a.strength}`)].filter(Boolean).join(" — ");
  });
  el("game").addEventListener("change", loadGame);
  addEventListener("hashchange", () => { if (hashTarget() && !followHash()) loadIndex(); });
  el("refresh").addEventListener("click", loadIndex);
  el("play").addEventListener("click", play);
  el("prev").addEventListener("click", () => jump(frameIndex - 1));
  el("next").addEventListener("click", () => jump(frameIndex + 1));
  el("first").addEventListener("click", () => jump(0));
  el("last").addEventListener("click", () => jump(game.frames.length - 1));
  el("timeline").addEventListener("input", () => jump(Number(el("timeline").value)));
  el("speed").addEventListener("change", () => { if (timer) { pause(); play(); } });
  el("labels").addEventListener("change", drawMap);
  el("events").addEventListener("click", (event) => { const button = event.target.closest("button[data-turn]"); if (button) jump(Number(button.dataset.turn) - 1); });
  section.addEventListener("keydown", (event) => {
    if (!game || /INPUT|SELECT|TEXTAREA/.test(event.target.tagName) || event.altKey || event.ctrlKey || event.metaKey) return;
    const actions = { ArrowLeft: () => jump(frameIndex - 1), ArrowRight: () => jump(frameIndex + 1), Home: () => jump(0), End: () => jump(game.frames.length - 1), " ": play };
    if (event.key === " " && event.target.tagName === "BUTTON") return;
    if (actions[event.key]) { event.preventDefault(); actions[event.key](); }
  });
  new ResizeObserver(drawMap).observe(el("map"));
  void document.fonts.ready.then(drawMap);
  document.addEventListener("visibilitychange", () => { if (document.hidden) pause(); });
  loadIndex();
})();
