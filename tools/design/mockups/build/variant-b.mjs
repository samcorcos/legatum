// Direction B: Atlas of Heirs. Printed cartography: paper panels, ink double rules, an illustrated atlas map.
// Type: Cormorant Garamond (display, italic map lettering) + Libre Franklin (labels, numbers).
import {
  ALL_ICONS, CITIES, GAME, PLAYERS, TARGET, UNITS, YIELD_LABEL, YIELD_ORDER,
  at, f1, icon, iconSprite, page, R, reachEdge, reachShapes, renderMap, ringEdge, tile,
} from './shared.mjs';

const C = {
  ink: '#2b2419', red: '#9a3322',
  food: '#5e8c3b', production: '#b8662a', goldY: '#a8841c', science: '#2c7fab', influence: '#8358bd',
};
const YC = { food: C.food, production: C.production, gold: C.goldY, science: C.science, influence: C.influence };

export const MAP_B = {
  pal: {
    Ocean: '#93b6bf', Coast: '#b4cfd1', Grassland: '#b3bf85', Plains: '#d8cb93', Forest: '#8ea473',
    Hills: '#cdb283', Desert: '#e8d4a0', Tundra: '#d9dcd2', Mountains: '#c2b194',
  },
  deep: '#8fb1ba', shallow: '#bcd6d6', seaMottle: 0.22, mottleBlend: 'soft-light',
  light: '#fffbea', shadow: '#4a3920', relief: 0.32, grain: 0.1, mottle: 0.35, blend: 5,
  tree: { dark: '#5b7649', light: '#8aa56b', shadow: 'rgba(70,55,30,.22)' },
  peak: { light: '#f1e8d4', dark: '#a8967a', snow: '#fcf9f1', line: '#4a3e2e', base: 'rgba(80,60,30,.16)' },
  hill: { fill: 'rgba(120,90,50,.1)', line: '#7c6547' },
  dune: '#b09459', snow: 'rgba(255,255,255,.7)', tuft: { grass: '#7c8f55', plains: '#a89a5e' },
  wave: 'rgba(60,100,110,.4)', ruin: '#6b4a2a',
  town: { ground: '#eadcc0', roof: '#9d4b32', roofLight: '#cf7d5e', shadow: 'rgba(60,40,20,.35)' },
  grid: 'rgba(70,52,28,.14)', gridWater: 'rgba(40,80,90,.1)',
  coast: '#3c4745', coastWidth: 1.1, waveLine: '#557f89', waveAlpha: 0.7,
  unexplored: '#ebe2cb', unexploredEdge: 'rgba(60,45,25,.55)', unexploredBlur: 1.5,
  unexploredPattern: `<pattern id="unx" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(-35)"><line x1="0" y1="0" x2="0" y2="7" stroke="rgba(110,85,50,.14)" stroke-width="1"/></pattern>`,
  fog: { desat: 0.55, tint: '#efe6cf', alpha: 0.42, hatch: 'rgba(100,78,45,.35)', hatchAlpha: 0.55, edge: 'rgba(60,45,25,.25)', glyph: 0.75 },
  border: { width: 1.3, band: 8, bandAlpha: 0.42, tint: 0.07, blur: 3.5 },
  vignette: 0.28, vignetteColor: '#5a4526',
};

export const STAR = 'M0 -5L.95 -.95L5 0L.95 .95L0 5L-.95 .95L-5 0L-.95 -.95Z';
const star = (s = 10, cls = 'star') => `<svg class="${cls}" width="${s}" height="${s}" viewBox="-5 -5 10 10" aria-hidden="true"><path d="${STAR}" fill="currentColor"/></svg>`;
const crest = (faction, color, size) =>
  `<span class="crest" style="--pc:${color};width:${size}px;height:${Math.round(size * 1.15)}px">${icon('faction-' + faction.toLowerCase(), Math.round(size * 0.66), 'ic', `color:${color}`)}</span>`;
const ROMAN = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII'];

function overlay() {
  const pc = (id) => PLAYERS[id].color;
  const s = at(UNITS.find((u) => u.selected).pos);
  const mid = at([25, 13]);
  const t = at(TARGET);
  let o = '';
  o += `<path d="${reachShapes()}" fill="rgba(255,250,235,.28)"/>`;
  o += `<path d="${reachEdge()}" fill="none" stroke="${C.ink}" stroke-width="1.2" stroke-dasharray="1.5 3" stroke-linecap="round"/>`;
  o += `<path d="${ringEdge(TARGET, 1)}" fill="none" stroke="${pc(0)}" stroke-width="1.6" stroke-dasharray="6 3"/>`;
  const d = `M${f1(s.x)} ${f1(s.y)}Q${f1(mid.x + 10)} ${f1(mid.y + 4)} ${f1(t.x)} ${f1(t.y + 12)}`;
  o += `<path d="${d}" fill="none" stroke="${C.red}" stroke-width="2" stroke-dasharray="6 4" stroke-linecap="round"/>`;
  o += `<g transform="translate(${f1(t.x)} ${f1(t.y)})"><circle r="11" fill="#f4ecd9" stroke="${C.red}" stroke-width="1.4"/><circle r="8" fill="none" stroke="${C.red}" stroke-width=".6"/><text y="4.5" text-anchor="middle" font-family="Cormorant" font-weight="700" font-size="13" fill="${C.red}">I</text></g>`;
  // Selection: an inked double ring.
  o += `<g transform="translate(${f1(s.x)} ${f1(s.y)})" fill="none" stroke="${C.red}"><circle r="24" stroke-width="1.6"/><circle r="27.5" stroke-width=".7"/></g>`;
  // Counters: square wargame chits.
  for (const u of UNITS) {
    const p = at(u.pos);
    const col = pc(u.owner);
    const w = 13;
    o += `<g transform="translate(${f1(p.x)} ${f1(p.y - 1)})">
      <rect x="${-w + 2}" y="${-w + 2.5}" width="${2 * w}" height="${2 * w}" fill="rgba(50,38,20,.35)"/>
      <rect x="${-w}" y="${-w}" width="${2 * w}" height="${2 * w}" fill="${col}" stroke="${C.ink}" stroke-width="1.2"/>
      <rect x="${-w + 2.5}" y="${-w + 2.5}" width="${2 * w - 5}" height="${2 * w - 5}" fill="none" stroke="rgba(255,250,235,.55)" stroke-width=".8"/>
      <use href="#i-unit-${u.kind.toLowerCase()}" x="-9" y="-9" width="18" height="18" style="color:#fbf6ea"/>
      ${u.hp < 100 ? `<rect x="${-w}" y="${w + 3}" width="${2 * w}" height="3" fill="#f4ecd9" stroke="${C.ink}" stroke-width=".6"/><rect x="${-w}" y="${w + 3}" width="${f1((2 * w * u.hp) / 100)}" height="3" fill="${C.ink}"/>` : ''}
      ${u.fortified ? `<use href="#i-fortify" x="${w - 4}" y="${-w - 6}" width="12" height="12" style="color:${C.ink}"/>` : ''}
    </g>`;
  }
  return o;
}

function banners() {
  return CITIES.map((ci) => {
    const p = at(ci.pos);
    const col = PLAYERS[ci.owner].color;
    const fog = tile(...ci.pos).vis === 'explored';
    return `<div class="cartouche ${fog ? 'fog' : ''}" style="left:${f1(p.x)}px;top:${f1(p.y - R - 10)}px;--pc:${col}">
      <span class="k-pop num">${ci.pop}</span>
      <span class="k-name">${ci.capital ? star(8, 'k-cap') : ''}${ci.name}</span>
      <span class="k-turns num">${ci.prod ? ci.turns : '?'}</span>
    </div>`;
  }).join('');
}

function topbar() {
  const g = GAME;
  const pct = (g.research.progress / g.research.cost) * 100;
  return `<header class="top">
    <svg class="logo" viewBox="0 0 236 50" width="120" height="26" aria-label="Legatum"><use href="#i-logo-legatum"/></svg>
    <div class="tb-stats">
      <div class="tb-cell">${icon('yield-gold', 16, 'ic', `color:${C.goldY}`)}<b class="num">${g.gold}</b><span class="pos num">+${g.goldPt}</span></div>
      <div class="tb-cell">${icon('yield-science', 16, 'ic', `color:${C.science}`)}<b class="num">+${g.sciencePt}</b></div>
      <div class="tb-cell research"><span class="lbl">Researching</span><span class="r-name">${g.research.name}</span><span class="r-bar"><i style="width:${pct}%"></i></span><span class="num r-t">${g.research.turns} turns</span></div>
    </div>
    <div class="tb-turn"><span class="lbl">Turn</span><b class="num">${g.turn}</b><span class="of">of ${g.limit}</span></div>
    <nav class="tb-nav"><button>Victory</button><button>Technologies</button><button>Menu</button></nav>
  </header>`;
}

function vpPanel() {
  const rows = GAME.vp.map((t, i) => {
    const w = (t.vp / t.target) * 100;
    const fill = t.members.length > 1
      ? `linear-gradient(90deg, ${PLAYERS[t.members[0]].color} 0 50%, ${PLAYERS[t.members[1]].color} 50% 100%)`
      : PLAYERS[t.members[0]].color;
    const ticks = Array.from({ length: t.target - 1 }, (_, k) => `<i style="left:${((k + 1) / t.target) * 100}%"></i>`).join('');
    return `<tr class="${t.mine ? 'mine' : ''}">
      <td class="v-rank">${ROMAN[i]}</td>
      <td class="v-crest">${t.members.map((m) => crest(PLAYERS[m].faction, PLAYERS[m].color, 20)).join('')}</td>
      <td class="v-name"><span>${t.label}</span><span class="v-track">${ticks}<span class="v-fill" style="width:${w}%;background:${fill}"></span></span></td>
      <td class="v-num num"><b>${t.vp}</b><span>/${t.target}</span></td>
    </tr>`;
  }).join('');
  return `<section class="panel vp">
    <header class="ph"><h3>The Race for the Legacy</h3><p>Victory points, live · first to target wins</p></header>
    <table class="vp-table">${rows}</table>
    <footer class="pf"><button class="link">Titles &amp; milestones</button></footer>
  </section>`;
}

function cityPanel() {
  const c = GAME.city;
  const col = PLAYERS[0].color;
  const yields = YIELD_ORDER.map((k) => `<div class="y ${k === 'production' ? 'hot' : ''}"><span class="y-v">${icon('yield-' + k, 16, 'ic', `color:${YC[k]}`)}<b class="num">${c.yields[k]}</b></span><span class="y-l">${YIELD_LABEL[k]}</span></div>`).join('');
  const row = (o) => `<tr class="${o.active ? 'on' : ''}"><td class="p-ic">${icon(o.icon, 18)}</td><td class="p-name">${o.key}${o.note ? ` <em>${o.note}</em>` : ''}</td><td class="p-lead"></td><td class="p-cost num">${o.cost}</td><td class="p-turns num">${o.turns} t</td></tr>`;
  return `<section class="panel city" style="--pc:${col}">
    <header class="c-head">
      <div class="c-kicker">${star(8)}Capital of the Japan${star(8)}</div>
      <h2>${c.name}</h2>
      <div class="c-meta"><span class="c-pop"><b class="num">${c.pop}</b> population</span><span>Strength <b class="num">${c.str}</b></span><span>Health <b class="num">${c.hp}</b>/${c.maxHp}</span></div>
    </header>
    <div class="yields">${yields}</div>
    <div class="c-block">
      <div class="c-line"><span class="lbl">Growth</span><span class="num">${c.food}/${c.growAt} · <i>${c.growTurns} turns</i></span></div>
      <div class="bar"><i style="width:${(c.food / c.growAt) * 100}%;background:${C.food}"></i></div>
    </div>
    <div class="c-block">
      <div class="c-line"><span class="lbl">Now producing</span><span class="num">${c.prodStored}/${c.prodCost} · <i>${c.prodTurns} turns</i></span></div>
      <div class="now">${icon('unit-settler', 22, 'ic', `color:${C.ink}`)}<span class="now-name">${c.producing}</span><button class="btn">Buy ${icon('yield-gold', 13, 'ic', `color:${C.goldY}`)}<span class="num">${c.buy}</span></button></div>
      <div class="bar"><i style="width:${(c.prodStored / c.prodCost) * 100}%;background:${C.production}"></i></div>
    </div>
    <table class="prod">
      <tr class="th"><td colspan="3">Units</td><td class="num">${icon('yield-production', 11, 'ic', `color:${C.production}`)}</td><td></td></tr>
      ${c.options.filter((o) => o.kind === 'unit').map(row).join('')}
      <tr class="th"><td colspan="5">Buildings</td></tr>
      ${c.options.filter((o) => o.kind === 'building').map(row).join('')}
    </table>
    <div class="focus"><span class="lbl">Focus</span><div class="seg"><button class="on">Balanced</button><button>Growth</button><button>Production</button></div></div>
    <footer class="c-foot"><span><i>Built:</i> Monument, Granary</span><span><i>Borders:</i> <span class="num">${c.tiles}</span> tiles · next at <span class="num">${c.influence}/${c.nextTile}</span> ${icon('yield-influence', 12, 'ic', `color:${C.influence}`)}</span></footer>
  </section>`;
}

function tooltip() {
  const b = GAME.city.breakdown;
  return `<div class="tip">
    <div class="t-head">${icon('yield-production', 14, 'ic', `color:${C.production}`)}<span>${b.title}</span><i>per turn</i></div>
    ${b.rows.map(([l, v]) => `<div class="t-row"><span>${l}</span><i></i><span class="num ${v < 0 ? 'neg' : ''}">${v > 0 ? '+' : '−'}${Math.abs(v)}</span></div>`).join('')}
    <div class="t-row total"><span>Total</span><i></i><span class="num">${b.total}</span></div>
    <p class="t-note">${b.note}</p>
  </div>`;
}

function unitBar() {
  const u = GAME.unit;
  const col = PLAYERS[0].color;
  return `<section class="panel unit">
    <div class="u-id">
      <span class="u-chit" style="background:${col}">${icon('unit-settler', 26, 'ic', 'color:#fbf6ea')}</span>
      <div><h3>${u.kind}</h3><p>Ada · <i>Civilian</i></p><p class="u-status">${u.status}</p></div>
    </div>
    <dl class="u-stats">
      <div><dt>Health</dt><dd><span class="bar"><i style="width:100%;background:${C.food}"></i></span><b class="num">${u.hp}</b></dd></div>
      <div><dt>Moves</dt><dd><span class="pips"><i class="on"></i><i class="on"></i></span><b class="num">${u.moves}/${u.maxMoves}</b></dd></div>
      <div><dt>Strength</dt><dd><i class="muted">civilian</i><b class="num">${u.strength}</b></dd></div>
    </dl>
    <div class="acts">
      <button class="btn primary">Found City<kbd>B</kbd></button>
      <button class="btn">Hold<kbd>F</kbd></button>
      <button class="btn">Skip<kbd>Space</kbd></button>
      <button class="btn quiet">Disband</button>
    </div>
  </section>
  <p class="u-hint">Right-click a tile to move · drag from the unit to plot a path</p>`;
}

function seatPanel() {
  const seats = GAME.seats.map((s, i) => `<div class="seat ${s.on ? 'on' : ''} ${s.ready ? 'ready' : ''}" style="--pc:${s.color}">
      ${crest(s.faction, s.color, 36)}
      <div class="s-id"><b>${s.name}</b><span><i>the ${s.faction}</i> · <span class="num">${s.orders}</span> orders</span></div>
      <span class="s-state">${s.ready ? '✓ Ready' : 'Planning'}</span><kbd>${i + 1}</kbd>
    </div>`).join('');
  return `<section class="panel seats">
    <header class="ph"><h3>This Turn</h3><p>Both seats plan at once</p></header>
    ${seats}
    <button class="orders">Pending orders <b class="num">3</b><span>▾</span></button>
    <div class="cta-wrap"><button class="cta"><span class="cta-l">${GAME.needing} units need orders</span><span class="cta-s">Next unit<kbd>Space</kbd></span></button>
    <button class="ready-any">Ready anyway<kbd>Enter</kbd></button></div>
  </section>`;
}

export const TOKENS_B = `
  --paper:#f2ead8; --paper-2:#e9dfc8; --paper-3:#dfd2b5; --vellum:#f7f1e3;
  --ink:#2b2419; --ink-2:#5b503f; --muted:#8a7c65; --rule:#bcac8e; --rule-2:#9d8c6c;
  --red:#9a3322; --ochre:#a47b2c; --good:#4f7f33; --bad:#a3392a;
  --display:'Cormorant', 'Garamond', serif; --sans:'Libre Franklin', 'Franklin Gothic', sans-serif;`;

const CSS = `
:root{${TOKENS_B}}
*{box-sizing:border-box;margin:0;padding:0}
body{width:1440px;height:900px;overflow:hidden;background:#8fb1ba;color:var(--ink);font:400 13px/1.35 var(--sans);font-feature-settings:"lnum" 1,"tnum" 1;-webkit-font-smoothing:antialiased;position:relative}
button{font:inherit;color:inherit;background:none;border:0;cursor:pointer}
table{border-collapse:collapse;width:100%}
.map{position:absolute;inset:0}
.num{font-variant-numeric:tabular-nums lining-nums}
.muted{color:var(--muted)} .pos{color:var(--good)} .neg{color:var(--bad)}
i{font-family:var(--display);font-style:italic;font-size:1.12em}
.ic{display:inline-block;vertical-align:middle;flex:none}
.lbl{font:600 10px/1 var(--sans);letter-spacing:.14em;text-transform:uppercase;color:var(--ink-2)}
kbd{display:inline-block;margin-left:8px;padding:2px 4px;min-width:16px;font:500 10px/1 var(--sans);color:var(--ink-2);border:1px solid var(--rule);text-align:center;background:var(--vellum)}
.star{display:inline-block;color:var(--red);flex:none}
.link{font:italic 500 15px/1 var(--display);color:var(--ink);border-bottom:1px solid var(--rule-2)}

/* Panels: square, opaque paper, an ink double rule. */
.panel{position:absolute;background:var(--paper);border:1px solid var(--ink);box-shadow:inset 0 0 0 3px var(--paper),inset 0 0 0 4px var(--rule-2)}
.ph{text-align:center;padding:16px 16px 12px;border-bottom:1px solid var(--rule)}
.ph h3{font:600 20px/1.1 var(--display);letter-spacing:.02em}
.ph p{margin-top:4px;font:italic 400 14px/1.2 var(--display);color:var(--muted)}
.pf{padding:12px 16px 16px;text-align:center}
.crest{display:inline-grid;place-items:center;flex:none;background:var(--vellum);border:1px solid var(--ink);
  clip-path:polygon(0 0,100% 0,100% 62%,50% 100%,0 62%)}

/* Top bar */
.top{position:absolute;left:0;right:0;top:0;height:52px;background:var(--paper);border-bottom:1px solid var(--ink);box-shadow:0 3px 0 -2px var(--rule-2);display:flex;align-items:center;z-index:5}
.top::after{content:'';position:absolute;left:0;right:0;bottom:-4px;height:1px;background:var(--ink)}
.logo{color:var(--ink);margin:0 32px 0 24px}
.tb-stats{display:flex;align-items:center;height:100%;border-left:1px solid var(--rule)}
.tb-cell{display:flex;align-items:center;gap:8px;height:100%;padding:0 20px;border-right:1px solid var(--rule)}
.tb-cell b{font:600 16px/1 var(--sans)}
.research{gap:12px}
.r-name{font:600 18px/1 var(--display)}
.r-bar{width:96px;height:3px;background:var(--paper-3)}
.r-bar i{display:block;height:100%;background:${C.science}}
.r-t{font-size:12px;color:var(--ink-2)}
.tb-turn{margin-left:auto;display:flex;align-items:baseline;gap:8px;padding:0 24px}
.tb-turn b{font:600 30px/1 var(--display)}
.tb-turn .of{font:italic 400 16px/1 var(--display);color:var(--ink-2)}
.tb-nav{display:flex;height:100%}
.tb-nav button{padding:0 20px;border-left:1px solid var(--rule);font:600 11px/1 var(--sans);letter-spacing:.14em;text-transform:uppercase;color:var(--ink-2)}

/* Victory race */
.vp{right:16px;top:76px;width:320px}
.vp-table{margin:8px 0}
.vp-table td{padding:6px 0;vertical-align:middle}
.vp-table tr+tr td{border-top:1px solid rgba(157,140,108,.35)}
.v-rank{width:40px;text-align:center;font:600 14px/1 var(--display);color:var(--muted)}
.v-crest{width:52px}
.v-crest .crest+.crest{margin-left:2px}
.v-name span:first-child{display:block;font:500 16px/1.1 var(--display)}
.v-track{display:block;position:relative;height:4px;margin-top:4px;background:var(--paper-3)}
.v-track i{position:absolute;top:0;bottom:0;width:1px;background:var(--paper)}
.v-fill{position:absolute;left:0;top:0;bottom:0}
.v-num{width:56px;text-align:right;padding-right:16px !important;font-size:12px;color:var(--muted)}
.v-num b{font:600 15px/1 var(--sans);color:var(--ink)}
.vp-table tr.mine td{background:var(--paper-2)}
.vp-table tr.mine .v-rank{color:var(--red)}
.vp-table tr.mine .v-name span:first-child{font-weight:700}

/* City panel */
.city{left:16px;top:76px;width:352px}
.c-head{text-align:center;padding:20px 16px 16px}
.c-kicker{display:flex;align-items:center;justify-content:center;gap:8px;font:600 10px/1 var(--sans);letter-spacing:.2em;text-transform:uppercase;color:var(--red)}
.c-head h2{margin-top:8px;font:600 44px/1 var(--display);letter-spacing:.02em}
.c-meta{display:flex;justify-content:center;gap:16px;margin-top:8px;font-size:12px;color:var(--ink-2)}
.c-meta b{color:var(--ink);font-weight:600}
.c-pop b{display:inline-grid;place-items:center;width:20px;height:20px;margin-right:4px;background:var(--pc);color:#fff;border:1px solid var(--ink)}
.yields{display:grid;grid-template-columns:repeat(5,1fr);border-top:1px solid var(--ink);border-bottom:1px solid var(--ink);margin:0 4px;box-shadow:0 2px 0 -1px var(--paper),0 3px 0 -1px var(--rule-2)}
.y{display:flex;flex-direction:column;align-items:center;gap:4px;padding:8px 0;border-left:1px solid var(--rule);position:relative}
.y:first-child{border-left:0}
.y-v{display:flex;align-items:center;gap:4px}
.y b{font:600 17px/1 var(--sans)}
.y-l{font:600 9px/1 var(--sans);letter-spacing:.12em;text-transform:uppercase;color:var(--muted)}
.y.hot{background:var(--paper-2)}
.y.hot::after{content:'';position:absolute;left:8px;right:8px;bottom:-1px;height:3px;background:var(--ink)}
.c-block{padding:16px 20px 0}
.c-line{display:flex;justify-content:space-between;align-items:baseline;margin-bottom:8px;font-size:12px;color:var(--ink-2)}
.bar{display:block;height:4px;background:var(--paper-3)}
.bar i{display:block;height:100%}
.now{display:flex;align-items:center;gap:8px;margin-bottom:8px}
.now-name{font:600 24px/1 var(--display)}
.btn{display:inline-flex;align-items:center;justify-content:center;gap:4px;height:32px;padding:0 12px;border:1px solid var(--ink);background:var(--vellum);font:600 11px/1 var(--sans);letter-spacing:.1em;text-transform:uppercase;color:var(--ink);white-space:nowrap}
.now .btn{margin-left:auto;height:28px}
.btn.primary{background:var(--ink);color:var(--paper)}
.btn.primary kbd{background:transparent;color:var(--paper-3);border-color:rgba(242,234,216,.4)}
.btn.quiet{border-color:transparent;background:none;color:var(--bad)}
.prod{margin-top:16px}
.prod td{height:30px;padding:0 4px;vertical-align:middle}
.prod .th td{height:28px;padding:8px 20px 0;font:italic 600 15px/1 var(--display);color:var(--red);border-bottom:1px solid var(--rule)}
.prod .th td.num{padding:8px 4px 0;text-align:right}
.p-ic{width:44px;padding-left:20px !important;color:var(--ink-2)}
.p-name{font-size:14px;white-space:nowrap}
.p-name em{font:italic 400 13px/1 var(--display);color:var(--muted)}
.p-lead{width:100%}
.p-lead::after{content:'';display:block;border-bottom:1px dotted var(--rule-2);transform:translateY(4px)}
.p-cost{text-align:right;font-size:12px}
.p-turns{width:44px;text-align:right;padding-right:20px !important;font-size:12px;color:var(--ink-2)}
.prod tr.on td{background:var(--paper-2)}
.prod tr.on .p-name{font-weight:700}
.prod tr.on .p-ic{box-shadow:inset 3px 0 0 var(--ink);color:var(--ink)}
.focus{display:flex;align-items:center;gap:16px;padding:16px 20px 0}
.seg{display:flex;flex:1;border:1px solid var(--ink)}
.seg button{flex:1;height:28px;font:500 12px/1 var(--sans);color:var(--ink-2);border-left:1px solid var(--rule)}
.seg button:first-child{border-left:0}
.seg .on{background:var(--ink);color:var(--paper)}
.c-foot{display:flex;flex-direction:column;gap:4px;margin-top:16px;padding:12px 20px 16px;border-top:1px solid var(--rule);font-size:12px;color:var(--ink-2)}

/* Tooltip: a slip of vellum, one ink rule. */
.tip{position:absolute;left:380px;top:152px;width:272px;padding:12px 16px;background:var(--vellum);border:1px solid var(--ink)}
.tip::before{content:'';position:absolute;left:-7px;top:24px;width:12px;height:12px;background:var(--vellum);border-left:1px solid var(--ink);border-bottom:1px solid var(--ink);transform:rotate(45deg)}
.t-head{display:flex;align-items:baseline;gap:8px;padding-bottom:8px;border-bottom:1px solid var(--ink);font:600 18px/1 var(--display)}
.t-head i{margin-left:auto;font-size:14px;color:var(--muted)}
.t-row{display:flex;align-items:baseline;gap:8px;height:24px;line-height:24px;font-size:13px}
.t-row i{flex:1;border-bottom:1px dotted var(--rule-2);transform:translateY(-6px)}
.t-row .num{min-width:24px;text-align:right;font-weight:600}
.t-row.total{margin-top:4px;border-top:1px solid var(--ink);height:32px;line-height:32px;font:600 18px/32px var(--display)}
.t-row.total i{border:0}
.t-row.total .num{font:600 15px/32px var(--sans)}
.t-note{margin-top:4px;font:italic 400 14px/18px var(--display);color:var(--ink-2)}

/* Unit bar */
.unit{left:384px;bottom:16px;width:712px;height:104px;display:grid;grid-template-columns:212px 168px 1fr;align-items:center}
.acts .btn{padding:0 8px}
.acts kbd{margin-left:6px}
.u-id{display:flex;align-items:center;gap:12px;padding:0 16px 0 20px}
.u-chit{width:48px;height:48px;display:grid;place-items:center;flex:none;border:1px solid var(--ink);box-shadow:inset 0 0 0 3px rgba(255,250,235,.4)}
.u-id h3{font:600 26px/1 var(--display)}
.u-id p{font-size:12px;color:var(--ink-2)}
.u-status{margin-top:4px;font:600 10px/1 var(--sans) !important;letter-spacing:.1em;white-space:nowrap;text-transform:uppercase;color:var(--red) !important}
.u-stats{display:flex;flex-direction:column;gap:4px;padding:0 16px;height:72px;justify-content:center;border-left:1px solid var(--rule);border-right:1px solid var(--rule)}
.u-stats div{display:grid;grid-template-columns:64px 1fr;align-items:center;height:20px}
.u-stats dt{font:600 9px/1 var(--sans);letter-spacing:.12em;text-transform:uppercase;color:var(--muted)}
.u-stats dd{display:flex;align-items:center;gap:8px;justify-content:space-between}
.u-stats dd .bar{flex:1}
.u-stats b{font-weight:600}
.pips{display:flex;gap:4px}
.pips i{width:8px;height:8px;border:1px solid var(--ink)}
.pips i.on{background:var(--ink)}
.acts{display:grid;grid-template-columns:1fr 1fr 1fr;grid-template-rows:36px 30px;gap:8px;padding:0 16px}
.acts .btn{height:auto}
.acts .primary{grid-column:1/4}
.u-hint{position:absolute;left:384px;width:712px;bottom:128px;text-align:center;font:italic 500 15px/1 var(--display);color:var(--ink)}

/* Seats */
.seats{right:16px;bottom:16px;width:320px}
.seat{display:grid;grid-template-columns:36px 1fr auto auto;gap:12px;align-items:center;height:64px;padding:0 16px;border-bottom:1px solid var(--rule)}
.seat.on{background:var(--paper-2);box-shadow:inset 3px 0 0 var(--pc)}
.s-id{display:flex;flex-direction:column;gap:2px}
.s-id b{font:600 20px/1 var(--display)}
.s-id span{font-size:11px;color:var(--ink-2)}
.s-state{font:600 10px/1 var(--sans);letter-spacing:.12em;text-transform:uppercase;color:var(--ink-2)}
.seat.ready .s-state{color:var(--good)}
.seat kbd{margin:0}
.orders{display:flex;align-items:center;gap:8px;width:100%;height:40px;padding:0 16px;font-size:12px;color:var(--ink-2);border-bottom:1px solid var(--rule)}
.orders b{color:var(--ink)}
.orders span{margin-left:auto}
.cta-wrap{padding:16px 16px 8px}
.cta{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:4px;width:100%;height:64px;background:var(--ink);color:var(--paper);box-shadow:inset 0 0 0 3px var(--ink),inset 0 0 0 4px rgba(242,234,216,.45)}
.cta-l{font:600 22px/1 var(--display)}
.cta-s{font:600 10px/1 var(--sans);letter-spacing:.16em;text-transform:uppercase;color:var(--paper-3)}
.cta kbd{background:transparent;color:var(--paper-3);border-color:rgba(242,234,216,.4)}
.ready-any{display:flex;align-items:center;justify-content:center;width:100%;height:32px;margin-top:4px;font-size:12px;color:var(--ink-2)}

/* City cartouches on the map */
.cartouche{position:absolute;transform:translate(-50%,-100%);display:flex;align-items:center;height:26px;padding:0 4px;background:var(--vellum);border:1px solid var(--ink);box-shadow:inset 0 0 0 2px var(--vellum),inset 0 0 0 2.6px var(--rule-2);white-space:nowrap}
.cartouche::before,.cartouche::after{content:'';position:absolute;top:50%;width:8px;height:8px;background:var(--vellum);border:1px solid var(--ink);transform:translateY(-50%) rotate(45deg)}
.cartouche::before{left:-5px;border-right:0;border-top:0}
.cartouche::after{right:-5px;border-left:0;border-bottom:0}
.cartouche.fog{opacity:.85}
.k-pop{display:grid;place-items:center;width:18px;height:18px;background:var(--pc);color:#fff;font:700 11px/1 var(--sans);border:1px solid var(--ink)}
.k-name{display:flex;align-items:center;gap:4px;padding:0 8px;font:700 15px/1 var(--display);letter-spacing:.06em;font-variant:small-caps}
.k-cap{color:var(--red)}
.k-turns{padding:0 4px 0 8px;border-left:1px solid var(--rule);font:500 italic 14px/1 var(--display);color:var(--ink-2)}
.map-label{position:absolute;font:italic 500 20px/1 var(--display);letter-spacing:.3em;color:rgba(40,70,78,.75);white-space:nowrap}
.map-label.land{font-style:normal;font-weight:600;font-size:13px;letter-spacing:.6em;text-transform:uppercase;color:rgba(70,52,28,.55)}
.compass{position:absolute}
`;

function compass(x, y) {
  return `<svg class="compass" style="left:${x - 36}px;top:${y - 36}px" width="72" height="72" viewBox="-36 -36 72 72">
    <circle r="28" fill="none" stroke="${C.ink}" stroke-width=".8" opacity=".6"/><circle r="24" fill="none" stroke="${C.ink}" stroke-width=".5" opacity=".5"/>
    <path d="M0 -34L4 -4L0 0L-4 -4Z" fill="${C.ink}"/><path d="M0 -34L-4 -4L0 0Z" fill="#f7f1e3"/>
    <path d="M0 34L4 4L0 0L-4 4Z" fill="#f7f1e3" stroke="${C.ink}" stroke-width=".6"/>
    <path d="M-30 0L-4 3L0 0L-4 -3Z" fill="#f7f1e3" stroke="${C.ink}" stroke-width=".6"/><path d="M30 0L4 3L0 0L4 -3Z" fill="#f7f1e3" stroke="${C.ink}" stroke-width=".6"/>
    <text y="-26" text-anchor="middle" dy="-3" font-family="Cormorant" font-size="10" font-weight="700" fill="${C.ink}" transform="translate(0 -2)">N</text>
  </svg>`;
}

export function pages() {
  const body = `${iconSprite(ALL_ICONS)}
  ${renderMap(MAP_B, overlay())}
  <div class="map-label" style="left:396px;top:474px">The Sundering Sea</div>
  <div class="map-label land" style="left:880px;top:196px;transform:rotate(58deg);transform-origin:left">Spine of Tesk</div>
  <div class="map-label land" style="left:896px;top:656px">Verdance</div>
  ${compass(1060, 520)}
  ${banners()}
  ${topbar()}
  ${cityPanel()}
  ${tooltip()}
  ${unitBar()}
  ${vpPanel()}
  ${seatPanel()}`;
  return {
    'variant-b.html': page({
      title: 'Legatum: Atlas of Heirs',
      fonts: 'family=Cormorant:ital,wght@0,400..700;1,400..700&family=Libre+Franklin:wght@400;500;600;700',
      css: CSS,
      body,
    }),
  };
}
