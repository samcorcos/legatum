// Direction A: Imperial Instrument. Engraved brass on blued steel, chamfered plates.
// Type: Cinzel (display: Roman inscriptional capitals, kin to the wordmark) + IBM Plex Sans (readouts).
import {
  ALL_ICONS, CITIES, GAME, PLAYERS, TARGET, UNITS, YIELD_LABEL, YIELD_ORDER,
  at, f1, icon, iconSprite, page, R, reachEdge, reachShapes, renderMap, ringEdge, tile,
} from './shared.mjs';

const C = {
  gold: '#c9a65c', goldHi: '#e6c883',
  food: '#86c46b', production: '#e0914f', goldY: '#e9c65b', science: '#62b8e2', influence: '#b993ea',
};
const YC = { food: C.food, production: C.production, gold: C.goldY, science: C.science, influence: C.influence };

export const MAP_A = {
  pal: {
    Ocean: '#12283b', Coast: '#1d4459', Grassland: '#5f7c44', Plains: '#8e8752', Forest: '#435f38',
    Hills: '#7e6c4b', Desert: '#b89f67', Tundra: '#8d9795', Mountains: '#6d6862',
  },
  deep: '#0f2335', shallow: '#2a5c70', seaMottle: 0.18,
  light: '#fff4d8', shadow: '#0a0d12', relief: 0.34, grain: 0.2, mottle: 0.3, blend: 4,
  tree: { dark: '#2d4a2a', light: '#587c45', shadow: 'rgba(8,12,8,.32)' },
  peak: { light: '#b3aba0', dark: '#5a534d', snow: '#efece6', line: '#2a2521', base: 'rgba(0,0,0,.22)' },
  hill: { fill: 'rgba(255,240,210,.07)', line: '#4a3e2c' },
  dune: '#8a744a', snow: 'rgba(240,244,244,.55)', tuft: { grass: '#40592f', plains: '#6c6538' },
  wave: 'rgba(170,200,215,.35)', ruin: '#e6c883',
  town: { ground: '#9c907a', roof: '#6f4f3c', roofLight: '#cdbb9c', shadow: 'rgba(0,0,0,.45)' },
  grid: 'rgba(8,10,12,.2)', gridWater: 'rgba(160,200,220,.06)',
  coast: '#eadfc2', coastWidth: 1, waveLine: '#b4ccd6', waveAlpha: 0.55,
  unexplored: '#0b0e12', unexploredEdge: 'rgba(201,166,92,.5)', unexploredBlur: 2,
  unexploredPattern: `<pattern id="unx" width="14" height="14" patternUnits="userSpaceOnUse" patternTransform="rotate(30)"><line x1="0" y1="0" x2="0" y2="14" stroke="rgba(201,166,92,.07)" stroke-width="1"/></pattern>`,
  fog: { desat: 0.6, tint: '#0a0f15', alpha: 0.34, hatch: 'rgba(0,0,0,.5)', hatchAlpha: 0.4, edge: 'rgba(230,200,131,.18)', glyph: 0.85 },
  border: { width: 1.5, band: 4, bandAlpha: 0.3, tint: 0.06, blur: 0 }, vignette: 0.45,
};

// The four-point star from the wordmark's A: the house ornament.
export const STAR = 'M0 -5L.95 -.95L5 0L.95 .95L0 5L-.95 .95L-5 0L-.95 -.95Z';
const star = (s = 10, cls = 'star') => `<svg class="${cls}" width="${s}" height="${s}" viewBox="-5 -5 10 10" aria-hidden="true"><path d="${STAR}" fill="currentColor"/></svg>`;
const crest = (faction, color, size, cls = 'crest') =>
  `<span class="${cls}" style="--pc:${color};width:${size}px;height:${size}px">${icon('faction-' + faction.toLowerCase(), Math.round(size * 0.7), 'ic', `color:${color}`)}</span>`;

function overlay() {
  const pc = (id) => PLAYERS[id].color;
  const s = at(UNITS.find((u) => u.selected).pos);
  const mid = at([25, 13]);
  const t = at(TARGET);
  let o = '';
  o += `<path d="${reachShapes()}" fill="rgba(230,200,131,.10)"/>`;
  o += `<path d="${reachEdge()}" fill="none" stroke="${C.goldHi}" stroke-width="1.2" stroke-dasharray="5 3" opacity=".9"/>`;
  o += `<path d="${ringEdge(TARGET, 1)}" fill="rgba(74,168,255,.08)" stroke="${pc(0)}" stroke-width="1.2" stroke-dasharray="2 3"/>`;
  const d = `M${f1(s.x)} ${f1(s.y)}L${f1(mid.x)} ${f1(mid.y)}L${f1(t.x)} ${f1(t.y + 10)}`;
  o += `<path d="${d}" fill="none" stroke="#0b0e12" stroke-width="5" stroke-linejoin="round" opacity=".55"/>`;
  o += `<path d="${d}" fill="none" stroke="${C.goldHi}" stroke-width="2" stroke-linejoin="round"/>`;
  o += `<circle cx="${f1(mid.x)}" cy="${f1(mid.y)}" r="2.5" fill="${C.goldHi}"/>`;
  o += `<g transform="translate(${f1(t.x)} ${f1(t.y)})"><path d="M0 -12 L11 0 L0 12 L-11 0Z" fill="#0b0e12" stroke="${C.goldHi}" stroke-width="1.2"/><text x="0" y="4" text-anchor="middle" font-family="IBM Plex Sans" font-weight="600" font-size="11" fill="${C.goldHi}">1</text></g>`;
  const b = 26, k = 7;
  o += `<g transform="translate(${f1(s.x)} ${f1(s.y)})" fill="none" stroke="${C.goldHi}" stroke-width="1.6">
    <path d="M${-b} ${-b + k}V${-b}H${-b + k}M${b - k} ${-b}H${b}V${-b + k}M${b} ${b - k}V${b}H${b - k}M${-b + k} ${b}H${-b}V${b - k}"/></g>`;
  for (const u of UNITS) {
    const p = at(u.pos);
    const col = pc(u.owner);
    const w = 15, c = 5;
    const shape = `M${-w + c} ${-w}H${w - c}L${w} ${-w + c}V${w - c}L${w - c} ${w}H${-w + c}L${-w} ${w - c}V${-w + c}Z`;
    o += `<g transform="translate(${f1(p.x)} ${f1(p.y - 2)})">
      <path d="${shape}" transform="translate(1.5 2.5)" fill="rgba(0,0,0,.45)"/>
      <path d="${shape}" fill="#10151b" stroke="${col}" stroke-width="1.6"/>
      <use href="#i-unit-${u.kind.toLowerCase()}" x="-10" y="-10" width="20" height="20" style="color:${col}"/>
      ${u.hp < 100 ? `<rect x="${-w}" y="${w + 3}" width="${2 * w}" height="2.5" fill="#0b0e12"/><rect x="${-w}" y="${w + 3}" width="${f1((2 * w * u.hp) / 100)}" height="2.5" fill="${u.hp > 60 ? '#86c46b' : '#e9c65b'}"/>` : ''}
      ${u.fortified ? `<use href="#i-fortify" x="${w - 5}" y="${-w - 5}" width="11" height="11" style="color:${C.goldHi}"/>` : ''}
    </g>`;
  }
  return o;
}

function banners() {
  return CITIES.map((ci) => {
    const p = at(ci.pos);
    const col = PLAYERS[ci.owner].color;
    const fog = tile(...ci.pos).vis === 'explored';
    const prodIcon = ci.prod && (['Settler', 'Spearman'].includes(ci.prod) ? 'unit-' : 'building-') + ci.prod.toLowerCase();
    return `<div class="banner ${fog ? 'fog' : ''}" style="left:${f1(p.x)}px;top:${f1(p.y - R - 12)}px;--pc:${col}">
      <span class="b-pop">${ci.pop}</span>
      <span class="b-name">${ci.capital ? star(8, 'b-cap') : ''}${ci.name}</span>
      ${ci.prod ? `<span class="b-prod">${icon(prodIcon, 12)}${ci.turns}</span>` : `<span class="b-prod">seen T24</span>`}
    </div>`;
  }).join('');
}

function topbar() {
  const g = GAME;
  const pct = (g.research.progress / g.research.cost) * 100;
  return `<header class="top">
    <div class="tb-left">
      <svg class="logo" viewBox="0 0 236 50" width="112" height="24" aria-label="Legatum"><use href="#i-logo-legatum"/></svg>
      <div class="tb-cell">${icon('yield-gold', 16, 'ic', `color:${C.goldY}`)}<b class="num">${g.gold}</b><span class="delta pos num">+${g.goldPt}</span></div>
      <div class="tb-cell">${icon('yield-science', 16, 'ic', `color:${C.science}`)}<b class="num">+${g.sciencePt}</b></div>
      <div class="tb-cell research">
        <div class="r-top"><span class="lbl">Research</span><span class="r-name">${g.research.name}</span><span class="r-turns num">${g.research.turns} turns</span></div>
        <div class="r-bar"><i style="width:${pct}%"></i></div>
      </div>
    </div>
    <div class="turn-crest"><span class="lbl">Turn</span><b class="turn-n num">${g.turn}</b><span class="turn-of num">of ${g.limit}</span></div>
    <div class="tb-right">
      <button class="tb-btn">${icon('vp', 16)}Victory</button>
      <button class="tb-btn">${icon('milestone-six-techs', 16)}Technologies</button>
      <button class="tb-btn menu" aria-label="Menu"><span></span><span></span><span></span></button>
    </div>
  </header>`;
}

function vpPanel() {
  const rows = GAME.vp.map((t, i) => {
    const w = (t.vp / t.target) * 100;
    const ticks = Array.from({ length: t.target - 1 }, (_, k) => `<i style="left:${((k + 1) / t.target) * 100}%"></i>`).join('');
    const fill = t.members.length > 1
      ? `linear-gradient(90deg, ${PLAYERS[t.members[0]].color} 0 50%, ${PLAYERS[t.members[1]].color} 50% 100%)`
      : PLAYERS[t.members[0]].color;
    const crests = t.members.map((m) => crest(PLAYERS[m].faction, PLAYERS[m].color, 24)).join('');
    return `<div class="vp-row ${t.mine ? 'mine' : ''}">
      <span class="vp-rank">${['I', 'II', 'III', 'IV', 'V', 'VI', 'VII'][i]}</span>
      <span class="vp-crests">${crests}</span>
      <span class="vp-name">${t.label}</span>
      <span class="vp-num num"><b>${t.vp}</b>/${t.target}</span>
      <span class="vp-track">${ticks}<span class="vp-fill" style="width:${w}%;background:${fill}"></span></span>
    </div>`;
  }).join('');
  return `<section class="plate vp"><div class="in">
    <header class="p-head"><span class="p-title">${star(9)}Victory Race</span><button class="link">Titles &amp; milestones</button></header>
    <div class="vp-rows">${rows}</div>
  </div></section>`;
}

function cityPanel() {
  const c = GAME.city;
  const col = PLAYERS[0].color;
  const yields = YIELD_ORDER.map((k) => `<div class="y ${k === 'production' ? 'hot' : ''}">${icon('yield-' + k, 16, 'ic', `color:${YC[k]}`)}<b class="num">${c.yields[k]}</b><span class="y-l">${YIELD_LABEL[k]}</span></div>`).join('');
  const row = (o) => `<div class="prow ${o.active ? 'on' : ''}">${icon(o.icon, 18)}<span class="p-name">${o.key}${o.note ? `<em>${o.note}</em>` : ''}</span><span class="p-cost num">${icon('yield-production', 12, 'ic', `color:${C.production}`)}${o.cost}</span><span class="p-turns num">${o.turns}<small>t</small></span></div>`;
  return `<section class="plate city" style="--pc:${col}"><div class="in">
    <header class="c-head">
      <div class="c-pop"><b class="num">${c.pop}</b></div>
      <div class="c-id"><h2>${c.name}</h2><div class="c-sub">${star(8, 'c-cap')}Capital · ${c.owner} of the ${c.faction}</div></div>
      <div class="c-def"><div><span class="lbl">Str</span><b class="num">${c.str}</b></div><div><span class="lbl">HP</span><b class="num">${c.hp}</b><span class="muted num">/${c.maxHp}</span></div></div>
    </header>
    <div class="yields">${yields}</div>
    <div class="c-block">
      <div class="c-line"><span class="lbl">Growth</span><span class="num">${c.food}<span class="muted">/${c.growAt}</span> · ${c.growTurns} turns</span></div>
      <div class="bar"><i style="width:${(c.food / c.growAt) * 100}%;background:${C.food}"></i></div>
    </div>
    <div class="c-block">
      <div class="c-line"><span class="lbl">Producing</span><span class="num">${c.prodStored}<span class="muted">/${c.prodCost}</span> · ${c.prodTurns} turns</span></div>
      <div class="now">${icon('unit-settler', 24, 'ic', `color:${C.goldHi}`)}<span class="now-name">${c.producing}</span><button class="btn buy">Buy ${icon('yield-gold', 13, 'ic', `color:${C.goldY}`)}<span class="num">${c.buy}</span></button></div>
      <div class="bar"><i style="width:${(c.prodStored / c.prodCost) * 100}%;background:${C.production}"></i></div>
    </div>
    <div class="sect"><span>Units</span></div>
    ${c.options.filter((o) => o.kind === 'unit').map(row).join('')}
    <div class="sect"><span>Buildings</span></div>
    ${c.options.filter((o) => o.kind === 'building').map(row).join('')}
    <div class="focus"><span class="lbl">Focus</span><div class="seg"><button class="on">Balanced</button><button>Growth</button><button>Production</button></div></div>
    <footer class="c-foot"><span><span class="lbl">Built</span>Monument, Granary</span><span><span class="lbl">Borders</span><span class="num">${c.tiles}</span>&nbsp;tiles · ${icon('yield-influence', 12, 'ic', `color:${C.influence}`)}&nbsp;<span class="num">${c.influence}/${c.nextTile}</span></span></footer>
  </div></section>`;
}

function tooltip() {
  const b = GAME.city.breakdown;
  return `<div class="tip-pointer"></div><div class="tip plate"><div class="in">
    <div class="t-head">${icon('yield-production', 14, 'ic', `color:${C.production}`)}<span>${b.title}</span><span class="t-per">per turn</span></div>
    ${b.rows.map(([l, v]) => `<div class="t-row"><span>${l}</span><i></i><span class="num ${v < 0 ? 'neg' : ''}">${v > 0 ? '+' : '−'}${Math.abs(v)}</span></div>`).join('')}
    <div class="t-row total"><span>Total</span><i></i><span class="num">${b.total}</span></div>
    <p class="t-note">${b.note}</p>
  </div></div>`;
}

function unitBar() {
  const u = GAME.unit;
  const col = PLAYERS[0].color;
  return `<section class="plate unit" style="--pc:${col}"><div class="in">
    <div class="u-id">
      <div class="u-tok">${icon('unit-settler', 28, 'ic', `color:${col}`)}</div>
      <div><h3>${u.kind}</h3><div class="c-sub">${u.owner} · Civilian</div><div class="u-status">${u.status}</div></div>
    </div>
    <div class="u-stats">
      <div class="st"><span class="lbl">HP</span><div class="bar"><i style="width:100%;background:${C.food}"></i></div><b class="num">${u.hp}</b></div>
      <div class="st"><span class="lbl">Moves</span><span class="pips"><i class="on"></i><i class="on"></i></span><b class="num">${u.moves}/${u.maxMoves}</b></div>
      <div class="st"><span class="lbl">Str</span><span class="st-note">Civilian</span><b class="num muted">${u.strength}</b></div>
    </div>
    <div class="acts">
      <button class="btn primary">Found City<kbd>B</kbd></button>
      <button class="btn">Hold<kbd>F</kbd></button>
      <button class="btn">Skip<kbd>Space</kbd></button>
      <button class="btn danger">Disband</button>
    </div>
  </div></section>
  <p class="u-hint">Right-click a tile to move · drag from the unit to plot a path</p>`;
}

function seatPanel() {
  const seats = GAME.seats.map((s, i) => `<div class="seat ${s.on ? 'on' : ''} ${s.ready ? 'ready' : ''}" style="--pc:${s.color}">
      ${crest(s.faction, s.color, 40)}
      <div class="s-id"><b>${s.name}</b><span>The ${s.faction} · <span class="num">${s.orders}</span> orders</span></div>
      <span class="s-state">${s.ready ? 'Ready' : 'Planning'}</span><kbd>${i + 1}</kbd>
    </div>`).join('');
  return `<section class="plate seats"><div class="in">
    <header class="p-head"><span class="p-title">${star(9)}Council</span><span class="p-meta">Simultaneous turn</span></header>
    ${seats}
    <button class="orders"><span>Pending orders</span><b class="num">3</b><span class="chev">▾</span></button>
    <button class="cta"><span class="cta-l">${GAME.needing} units need orders</span><span class="cta-s">Next unit<kbd>Space</kbd></span></button>
    <button class="ready-any">Ready anyway<kbd>Enter</kbd></button>
  </div></section>`;
}

export const TOKENS_A = `
  --steel-0:#0a0d11; --steel-1:#10151b; --steel-2:#151b22; --steel-3:#1c242d;
  --rule:#27303a; --rule-2:#364252; --gold:#c9a65c; --gold-hi:#e6c883; --gold-lo:#8a7040;
  --gold-rule:rgba(201,166,92,.42);
  --text:#ece6d6; --text-2:#aeb2b6; --muted:#78808a; --good:#86c46b; --bad:#df6a55;
  --display:'Cinzel', 'Trajan Pro', serif; --sans:'IBM Plex Sans', 'Helvetica Neue', sans-serif;
  --c:8px;`;

const CSS = `
:root{${TOKENS_A}}
*{box-sizing:border-box;margin:0;padding:0}
body{width:1440px;height:900px;overflow:hidden;background:#0a0d11;color:var(--text);font:400 13px/1.35 var(--sans);font-feature-settings:"lnum" 1,"tnum" 1;-webkit-font-smoothing:antialiased;position:relative}
button{font:inherit;color:inherit;background:none;border:0;cursor:pointer}
.map{position:absolute;inset:0}
.num{font-variant-numeric:tabular-nums lining-nums}
.muted{color:var(--muted)} .pos{color:var(--good)} .neg{color:var(--bad)}
.ic{display:inline-block;vertical-align:middle;flex:none}
.lbl{font:600 10px/1 var(--sans);letter-spacing:.16em;text-transform:uppercase;color:var(--gold)}
kbd{display:inline-block;margin-left:8px;padding:2px 4px;min-width:16px;font:500 10px/1 var(--sans);letter-spacing:.04em;color:var(--text-2);border:1px solid var(--rule-2);text-align:center}
.star{display:inline-block;color:var(--gold);margin-right:8px;flex:none}

/* Plate: chamfered, a gold hairline frame around opaque steel. */
.plate{position:absolute;background:var(--gold-rule);padding:1px;
  clip-path:polygon(var(--c) 0,calc(100% - var(--c)) 0,100% var(--c),100% calc(100% - var(--c)),calc(100% - var(--c)) 100%,var(--c) 100%,0 calc(100% - var(--c)),0 var(--c))}
.plate>.in{background:var(--steel-1);height:100%;
  clip-path:polygon(calc(var(--c) - .6px) 0,calc(100% - var(--c) + .6px) 0,100% calc(var(--c) - .6px),100% calc(100% - var(--c) + .6px),calc(100% - var(--c) + .6px) 100%,calc(var(--c) - .6px) 100%,0 calc(100% - var(--c) + .6px),0 calc(var(--c) - .6px))}
.p-head{display:flex;align-items:center;justify-content:space-between;height:40px;padding:0 16px;border-bottom:1px solid var(--rule)}
.p-title{font:600 13px/1 var(--display);letter-spacing:.2em;text-transform:uppercase;color:var(--gold-hi);display:flex;align-items:center}
.p-meta{font-size:11px;color:var(--muted)}
.link{color:var(--text-2);font-size:12px;border-bottom:1px solid var(--rule-2);padding-bottom:1px}
.crest{display:inline-grid;place-items:center;flex:none;background:var(--steel-0);border:1px solid color-mix(in srgb,var(--pc) 55%,transparent);
  clip-path:polygon(25% 0,75% 0,100% 25%,100% 75%,75% 100%,25% 100%,0 75%,0 25%)}

/* Top bar */
.top{position:absolute;left:0;right:0;top:0;height:48px;background:#0c1015;border-bottom:1px solid var(--gold-rule);display:flex;align-items:center;justify-content:space-between;z-index:5}
.top::after{content:'';position:absolute;left:0;right:0;bottom:-4px;height:1px;background:rgba(201,166,92,.16)}
.tb-left{display:flex;align-items:center;height:100%}
.logo{color:var(--gold-hi);margin:0 24px}
.tb-cell{display:flex;align-items:center;gap:8px;height:100%;padding:0 16px;border-left:1px solid var(--rule)}
.tb-cell b{font:600 16px/1 var(--sans)}
.delta{font-size:12px}
.research{flex-direction:column;align-items:stretch;justify-content:center;gap:8px;width:232px;border-right:1px solid var(--rule)}
.r-top{display:flex;align-items:baseline;gap:8px;white-space:nowrap}
.r-name{font:500 14px/1 var(--sans);color:var(--text)}
.r-turns{margin-left:auto;font-size:12px;color:var(--text-2)}
.r-bar{height:2px;background:var(--rule)}
.r-bar i{display:block;height:100%;background:#62b8e2}
.turn-crest{position:absolute;left:50%;top:0;transform:translateX(-50%);height:60px;width:176px;display:flex;align-items:center;justify-content:center;gap:8px;padding-bottom:4px;
  background:#0c1015;clip-path:polygon(0 0,100% 0,100% 44px,calc(100% - 16px) 60px,16px 60px,0 44px)}
.turn-crest::before{content:'';position:absolute;inset:0;background:var(--gold-rule);clip-path:polygon(0 0,100% 0,100% 44px,calc(100% - 16px) 60px,16px 60px,0 44px,0 0,1px 0,1px 43.6px,16.4px 59px,calc(100% - 16.4px) 59px,calc(100% - 1px) 43.6px,calc(100% - 1px) 0)}
.turn-n{font:600 30px/1 var(--display);color:var(--gold-hi)}
.turn-of{font-size:12px;color:var(--text-2)}
.tb-right{display:flex;align-items:center;height:100%}
.tb-btn{height:100%;padding:0 20px;display:flex;align-items:center;gap:8px;border-left:1px solid var(--rule);font:600 11px/1 var(--sans);letter-spacing:.16em;text-transform:uppercase;color:var(--text-2)}
.tb-btn .ic{color:var(--gold)}
.menu{flex-direction:column;justify-content:center;gap:4px;width:56px}
.menu span{display:block;width:16px;height:1px;background:var(--text-2)}

/* Victory race */
.vp{right:16px;top:72px;width:320px}
.vp-rows{padding:8px 0}
.vp-row{display:grid;grid-template-columns:24px auto 1fr auto;grid-template-rows:24px 8px;column-gap:8px;row-gap:4px;padding:4px 16px 8px 8px;align-items:center}
.vp-row.mine{background:var(--steel-2);box-shadow:inset 2px 0 0 var(--gold)}
.vp-rank{grid-row:1/3;font:600 12px/1 var(--display);color:var(--muted);text-align:center}
.vp-row.mine .vp-rank{color:var(--gold-hi)}
.vp-crests{display:flex;gap:4px;grid-row:1/3}
.vp-name{font-size:13px;color:var(--text-2)}
.vp-row.mine .vp-name{color:var(--text);font-weight:600}
.vp-num{font-size:12px;color:var(--muted)}
.vp-num b{font-size:14px;color:var(--text);font-weight:600}
.vp-track{grid-column:3/5;position:relative;height:4px;background:#0a0d11;outline:1px solid var(--rule)}
.vp-track i{position:absolute;top:0;bottom:0;width:1px;background:var(--steel-1)}
.vp-fill{position:absolute;left:0;top:0;bottom:0}

/* City panel */
.city{left:16px;top:72px;width:352px}
.c-head{display:grid;grid-template-columns:56px 1fr auto;gap:16px;align-items:center;padding:16px}
.c-pop{width:56px;height:56px;display:grid;place-items:center;background:var(--pc);clip-path:polygon(10px 0,100% 0,100% calc(100% - 10px),calc(100% - 10px) 100%,0 100%,0 10px)}
.c-pop b{font:600 28px/1 var(--display);color:#0a0d11}
.c-id h2{font:600 30px/1 var(--display);letter-spacing:.06em}
.c-cap{color:var(--gold-hi);margin-right:0}
.c-sub{margin-top:8px;display:flex;align-items:center;gap:8px;font-size:12px;color:var(--text-2);white-space:nowrap}
.c-def{display:flex;flex-direction:column;gap:8px;text-align:right}
.c-def div{display:flex;gap:8px;align-items:baseline;justify-content:flex-end}
.c-def b{font-size:14px;font-weight:600}
.yields{display:grid;grid-template-columns:repeat(5,1fr);border-top:1px solid var(--rule);border-bottom:1px solid var(--rule)}
.y{display:grid;grid-template-columns:auto auto;justify-content:center;align-items:center;column-gap:8px;row-gap:8px;padding:12px 0 8px;border-left:1px solid var(--rule);position:relative}
.y:first-child{border-left:0}
.y b{font:600 18px/1 var(--sans)}
.y-l{grid-column:1/3;text-align:center;font:600 9px/1 var(--sans);letter-spacing:.14em;text-transform:uppercase;color:var(--muted)}
.y.hot{background:var(--steel-3)}
.y.hot::after{content:'';position:absolute;left:-1px;right:-1px;bottom:-1px;height:2px;background:var(--gold)}
.c-block{padding:16px 16px 0}
.c-line{display:flex;justify-content:space-between;align-items:baseline;margin-bottom:8px;font-size:12px;color:var(--text-2)}
.bar{height:3px;background:#0a0d11;outline:1px solid var(--rule)}
.bar i{display:block;height:100%}
.now{display:flex;align-items:center;gap:12px;margin:4px 0 12px}
.now-name{font:600 20px/1 var(--display);letter-spacing:.06em}
.btn{display:inline-flex;align-items:center;justify-content:center;gap:4px;height:32px;padding:0 12px;border:1px solid var(--rule-2);background:var(--steel-2);font:600 11px/1 var(--sans);letter-spacing:.12em;text-transform:uppercase;color:var(--text);white-space:nowrap;
  clip-path:polygon(6px 0,100% 0,100% calc(100% - 6px),calc(100% - 6px) 100%,0 100%,0 6px)}
.btn.buy{margin-left:auto;height:28px;border-color:var(--gold-rule);color:var(--gold-hi)}
.btn.primary{background:var(--gold);border-color:var(--gold);color:#12100a}
.btn.primary kbd{color:#3a2f16;border-color:rgba(18,16,10,.35)}
.btn.danger{color:#e58a78;border-color:transparent;background:none}
.sect{display:flex;align-items:center;gap:12px;margin:24px 16px 8px;font:600 11px/1 var(--display);letter-spacing:.24em;text-transform:uppercase;color:var(--gold)}
.sect::after{content:'';flex:1;height:1px;background:var(--rule)}
.prow{display:grid;grid-template-columns:18px 1fr 56px 32px;gap:12px;align-items:center;height:32px;padding:0 16px;color:var(--text-2);position:relative}
.prow .ic{color:var(--text-2)}
.p-name{font-size:14px;color:var(--text);display:flex;align-items:baseline;gap:8px}
.p-name em{font-style:normal;font-size:11px;color:var(--muted)}
.p-cost{display:flex;align-items:center;gap:4px;justify-content:flex-end;font-size:12px}
.p-turns{text-align:right;font-size:12px}
.p-turns small{font-size:10px;color:var(--muted);margin-left:1px}
.prow.on{background:var(--steel-3)}
.prow.on::before{content:'';position:absolute;left:0;top:0;bottom:0;width:2px;background:var(--gold)}
.prow.on .p-name,.prow.on .ic{color:var(--gold-hi)}
.focus{display:flex;align-items:center;gap:16px;padding:24px 16px 0}
.seg{display:flex;flex:1;border:1px solid var(--rule-2)}
.seg button{flex:1;height:32px;font:500 12px/1 var(--sans);color:var(--text-2);border-left:1px solid var(--rule-2)}
.seg button:first-child{border-left:0}
.seg .on{background:var(--steel-3);color:var(--gold-hi);box-shadow:inset 0 -2px 0 var(--gold)}
.c-foot{display:flex;flex-direction:column;gap:8px;margin-top:24px;padding:16px;border-top:1px solid var(--rule);font-size:12px;color:var(--text-2)}
.c-foot>span{display:flex;align-items:center}
.c-foot .lbl{width:72px;color:var(--muted)}

/* Tooltip */
.tip{left:376px;top:164px;width:272px;--c:6px;background:var(--gold)}
.tip>.in{background:#0c1015;padding:12px 16px}
.t-head{display:flex;align-items:center;gap:8px;padding-bottom:8px;margin-bottom:4px;border-bottom:1px solid var(--rule);font:600 12px/1 var(--display);letter-spacing:.18em;text-transform:uppercase;color:var(--gold-hi)}
.t-per{margin-left:auto;font:400 11px/1 var(--sans);letter-spacing:0;text-transform:none;color:var(--muted)}
.t-row{display:flex;align-items:baseline;gap:8px;height:24px;line-height:24px;font-size:13px;color:var(--text-2)}
.t-row i{flex:1;border-bottom:1px dotted var(--rule-2);transform:translateY(-6px)}
.t-row .num{min-width:24px;text-align:right;color:var(--text);font-weight:600}
.t-row.total{margin-top:4px;border-top:1px solid var(--rule);height:32px;line-height:32px;color:var(--text);font-weight:600}
.t-row.total i{border:0}
.t-row.total .num{color:var(--gold-hi);font-size:16px}
.t-note{margin-top:4px;font-size:11px;line-height:16px;color:var(--muted)}
.tip-pointer{position:absolute;left:368px;top:176px;width:9px;height:14px;background:var(--gold);clip-path:polygon(0 50%,100% 0,100% 100%)}

/* Unit command bar */
.unit{left:384px;bottom:16px;width:712px;height:104px}
.unit>.in{display:grid;grid-template-columns:228px 148px 1fr;align-items:stretch}
.acts .btn{padding:0 8px}
.acts kbd{margin-left:6px}
.u-id{display:flex;align-items:center;gap:12px;padding:0 16px}
.u-tok{width:48px;height:48px;display:grid;place-items:center;flex:none;background:var(--steel-0);border:1.5px solid var(--pc);clip-path:polygon(9px 0,calc(100% - 9px) 0,100% 9px,100% calc(100% - 9px),calc(100% - 9px) 100%,9px 100%,0 calc(100% - 9px),0 9px)}
.u-id h3{font:600 20px/1 var(--display);letter-spacing:.06em}
.u-id .c-sub{margin-top:4px}
.u-status{margin-top:8px;font:600 10px/1 var(--sans);letter-spacing:.12em;white-space:nowrap;text-transform:uppercase;color:var(--gold-hi)}
.u-stats{display:flex;flex-direction:column;justify-content:center;gap:4px;padding:0 16px;border-left:1px solid var(--rule);border-right:1px solid var(--rule)}
.st{display:grid;grid-template-columns:40px 1fr 32px;align-items:center;gap:8px;height:20px}
.st .lbl{color:var(--muted)}
.st b{font-size:13px;font-weight:600;text-align:right}
.st-note{font-size:11px;color:var(--muted)}
.pips{display:flex;gap:8px;padding-left:2px}
.pips i{width:7px;height:7px;border:1px solid var(--gold);transform:rotate(45deg)}
.pips i.on{background:var(--gold)}
.acts{display:grid;grid-template-columns:1fr 1fr 1fr;grid-template-rows:36px 32px;gap:8px;align-content:center;padding:0 16px}
.acts .btn{height:auto}
.acts .btn.primary{grid-column:1/4}
.u-hint{position:absolute;left:384px;width:712px;bottom:128px;text-align:center;font-size:12px;color:rgba(236,230,214,.7);text-shadow:0 1px 2px #000}

/* Seats */
.seats{right:16px;bottom:16px;width:320px}
.seat{display:grid;grid-template-columns:40px 1fr auto auto;gap:12px;align-items:center;height:64px;padding:0 16px;border-bottom:1px solid var(--rule);position:relative}
.seat.on{background:var(--steel-2)}
.seat.on::before{content:'';position:absolute;left:0;top:0;bottom:0;width:2px;background:var(--pc)}
.s-id{display:flex;flex-direction:column;gap:4px}
.s-id b{font:600 16px/1 var(--display);letter-spacing:.08em}
.s-id span{font-size:11px;color:var(--muted)}
.s-state{font:600 10px/1 var(--sans);letter-spacing:.16em;text-transform:uppercase;color:var(--text-2);padding:4px 8px;border:1px solid var(--rule-2)}
.seat.ready .s-state{color:var(--good);border-color:rgba(134,196,107,.45)}
.seat kbd{margin:0}
.orders{display:flex;align-items:center;gap:8px;width:100%;height:40px;padding:0 16px;font-size:12px;color:var(--text-2);border-bottom:1px solid var(--rule)}
.orders b{color:var(--text)}
.orders .chev{margin-left:auto;color:var(--muted)}
.cta{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:8px;width:calc(100% - 32px);height:64px;margin:16px 16px 8px;background:var(--gold);color:#12100a;
  clip-path:polygon(10px 0,calc(100% - 10px) 0,100% 10px,100% calc(100% - 10px),calc(100% - 10px) 100%,10px 100%,0 calc(100% - 10px),0 10px)}
.cta-l{font:600 17px/1 var(--display);letter-spacing:.08em}
.cta-s{font:600 10px/1 var(--sans);letter-spacing:.18em;text-transform:uppercase;color:#4a3c1c}
.cta kbd{color:#4a3c1c;border-color:rgba(18,16,10,.3)}
.ready-any{display:flex;align-items:center;justify-content:center;width:100%;height:32px;margin-bottom:8px;font-size:12px;color:var(--text-2)}

/* City banners on the map */
.banner{position:absolute;transform:translate(-50%,-100%);display:flex;align-items:stretch;height:24px;background:rgba(10,13,17,.94);border:1px solid var(--pc);white-space:nowrap;
  clip-path:polygon(6px 0,calc(100% - 6px) 0,100% 6px,100% 100%,0 100%,0 6px)}
.banner.fog{opacity:.85}
.b-pop{display:grid;place-items:center;min-width:24px;padding:0 4px;background:var(--pc);color:#0a0d11;font:700 12px/1 var(--sans)}
.b-name{display:flex;align-items:center;gap:6px;padding:0 10px;font:600 12px/1 var(--display);letter-spacing:.14em;text-transform:uppercase;color:var(--text)}
.b-cap{color:var(--gold-hi)}
.b-prod{display:flex;align-items:center;gap:4px;padding:0 8px;border-left:1px solid var(--rule-2);font:600 11px/1 var(--sans);color:var(--text-2)}
.b-prod .ic{color:var(--text-2)}
.map-label{position:absolute;font:500 13px/1 var(--display);letter-spacing:.5em;text-transform:uppercase;color:rgba(234,223,194,.5);white-space:nowrap}
`;

export function pages() {
  const body = `${iconSprite(ALL_ICONS)}
  ${renderMap(MAP_A, overlay())}
  <div class="map-label" style="left:392px;top:470px">The Sundering</div>
  <div class="map-label" style="left:430px;top:492px">Sea</div>
  <div class="map-label" style="left:890px;top:196px;transform:rotate(58deg);transform-origin:left;font-size:12px">Spine of Tesk</div>
  ${banners()}
  ${topbar()}
  ${cityPanel()}
  ${tooltip()}
  ${unitBar()}
  ${vpPanel()}
  ${seatPanel()}`;
  return {
    'variant-a.html': page({
      title: 'Legatum: Imperial Instrument',
      fonts: 'family=Cinzel:wght@400..700&family=IBM+Plex+Sans:ital,wght@0,400;0,500;0,600;0,700;1,400',
      css: CSS,
      body,
    }),
  };
}
