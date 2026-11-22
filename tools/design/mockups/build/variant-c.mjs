// Direction C: Heir's Court. Art-deco / neo-classical ceremony: lapis panels, gold-leaf linework, symmetric headers.
// Type: Bodoni Moda (display) + Josefin Sans (UI; 1920s geometric, kin to the wordmark).
import {
  ALL_ICONS, CITIES, GAME, PLAYERS, TARGET, UNITS, YIELD_LABEL, YIELD_ORDER,
  at, f1, icon, iconSprite, page, R, reachEdge, reachShapes, renderMap, ringEdge, tile,
} from './shared.mjs';

const C = {
  gold: '#d4b46a', goldHi: '#f0d894', lapis: '#0f1a3a',
  food: '#8fcf72', production: '#eb9a57', goldY: '#f0cf62', science: '#6cc0ea', influence: '#c29cf0',
};
const YC = { food: C.food, production: C.production, gold: C.goldY, science: C.science, influence: C.influence };

export const MAP_C = {
  pal: {
    Ocean: '#0c2c3c', Coast: '#1b5563', Grassland: '#577f47', Plains: '#9e8d53', Forest: '#31593a',
    Hills: '#8a7049', Desert: '#c9a768', Tundra: '#9ba7a4', Mountains: '#7b7067',
  },
  deep: '#0b2837', shallow: '#1f5d69', seaMottle: 0.2,
  light: '#fff0c8', shadow: '#050914', relief: 0.36, grain: 0.18, mottle: 0.3, blend: 4.5,
  tree: { dark: '#24472e', light: '#527e46', shadow: 'rgba(5,10,8,.32)' },
  peak: { light: '#cbbfab', dark: '#5f534a', snow: '#f4f0e6', line: '#2a221c', base: 'rgba(0,0,0,.22)' },
  hill: { fill: 'rgba(255,236,200,.07)', line: '#4e3f2a' },
  dune: '#8f7442', snow: 'rgba(240,244,244,.6)', tuft: { grass: '#3f6233', plains: '#736637' },
  wave: 'rgba(212,180,106,.3)', ruin: '#f0d894',
  town: { ground: '#b9a886', roof: '#7c3f33', roofLight: '#e2cba2', shadow: 'rgba(0,0,0,.45)' },
  grid: 'rgba(5,10,25,.22)', gridWater: 'rgba(212,180,106,.05)',
  coast: '#e4c77d', coastWidth: 1.1, waveLine: '#d4b46a', waveAlpha: 0.4,
  unexplored: '#0a1128', unexploredEdge: 'rgba(212,180,106,.6)', unexploredBlur: 1.5,
  unexploredPattern: `<pattern id="unx" width="24" height="12" patternUnits="userSpaceOnUse"><path d="M0 12a12 12 0 0 1 24 0M-12 6a12 12 0 0 1 24 0M12 6a12 12 0 0 1 24 0" fill="none" stroke="rgba(212,180,106,.09)" stroke-width="1"/></pattern>`,
  fog: { desat: 0.6, tint: '#0a1128', alpha: 0.38, hatch: 'rgba(10,17,40,.55)', hatchAlpha: 0.35, edge: 'rgba(240,216,148,.2)', glyph: 0.85 },
  border: { width: 1.6, band: 5, bandAlpha: 0.32, tint: 0.07, blur: 0 },
  vignette: 0.5, vignetteColor: '#050a1c',
};

export const STAR = 'M0 -5L.95 -.95L5 0L.95 .95L0 5L-.95 .95L-5 0L-.95 -.95Z';
const star = (s = 10, cls = 'star') => `<svg class="${cls}" width="${s}" height="${s}" viewBox="-5 -5 10 10" aria-hidden="true"><path d="${STAR}" fill="currentColor"/></svg>`;
// Deco sunburst: the house star over a fan of rays.
const fan = (w = 56) => `<svg class="fan" width="${w}" height="${w / 2}" viewBox="-28 -28 56 28" aria-hidden="true">
  ${Array.from({ length: 9 }, (_, i) => { const a = Math.PI * (i + 1) / 10; return `<line x1="${f1(-Math.cos(a) * 9)}" y1="${f1(-Math.sin(a) * 9)}" x2="${f1(-Math.cos(a) * (i % 2 ? 22 : 27))}" y2="${f1(-Math.sin(a) * (i % 2 ? 22 : 27))}" stroke="currentColor" stroke-width=".8"/>`; }).join('')}
  <path d="M-7 0a7 7 0 0 1 14 0" fill="none" stroke="currentColor" stroke-width=".8"/>
  <path d="${STAR}" transform="translate(0 -1) scale(.8)" fill="currentColor"/></svg>`;
const rule = (w = 64) => `<svg class="rule" width="${w}" height="8" viewBox="0 0 ${w} 8" aria-hidden="true"><path d="M0 4H${w - 10}" stroke="currentColor" stroke-width="1"/><path d="M${w - 8} 4l4 -4l4 4l-4 4z" fill="currentColor"/></svg>`;
const ruleR = (w = 64) => `<svg class="rule" width="${w}" height="8" viewBox="0 0 ${w} 8" aria-hidden="true"><path d="M10 4H${w}" stroke="currentColor" stroke-width="1"/><path d="M0 4l4 -4l4 4l-4 4z" fill="currentColor"/></svg>`;
const heading = (t, w = 48) => `<div class="deco-h">${rule(w)}<span>${t}</span>${ruleR(w)}</div>`;
const crest = (faction, color, size) =>
  `<span class="crest" style="--pc:${color};width:${size}px;height:${size}px">${icon('faction-' + faction.toLowerCase(), Math.round(size * 0.62), 'ic', `color:${color}`)}</span>`;
const ROMAN = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII'];

function overlay() {
  const pc = (id) => PLAYERS[id].color;
  const s = at(UNITS.find((u) => u.selected).pos);
  const mid = at([25, 13]);
  const t = at(TARGET);
  let o = '';
  o += `<path d="${reachShapes()}" fill="rgba(240,216,148,.12)"/>`;
  o += `<path d="${reachEdge()}" fill="none" stroke="${C.goldHi}" stroke-width="1.2" opacity=".9"/>`;
  o += `<path d="${ringEdge(TARGET, 1)}" fill="rgba(74,168,255,.1)" stroke="${pc(0)}" stroke-width="1.4" stroke-dasharray="1 3" stroke-linecap="round"/>`;
  const d = `M${f1(s.x)} ${f1(s.y)}L${f1(mid.x)} ${f1(mid.y)}L${f1(t.x)} ${f1(t.y + 12)}`;
  o += `<path d="${d}" fill="none" stroke="#050914" stroke-width="5" stroke-linejoin="round" opacity=".5"/>`;
  o += `<path d="${d}" fill="none" stroke="${C.goldHi}" stroke-width="2" stroke-linejoin="round"/>`;
  o += `<g transform="translate(${f1(t.x)} ${f1(t.y)})"><path d="M0 -13L13 0L0 13L-13 0Z" fill="${C.lapis}" stroke="${C.goldHi}" stroke-width="1.2"/><path d="M0 -9L9 0L0 9L-9 0Z" fill="none" stroke="${C.gold}" stroke-width=".6"/><text y="4" text-anchor="middle" font-family="Bodoni Moda" font-weight="600" font-size="12" fill="${C.goldHi}">I</text></g>`;
  // Selection: a gold ring with four diamond studs.
  o += `<g transform="translate(${f1(s.x)} ${f1(s.y)})" fill="none" stroke="${C.goldHi}"><circle r="24" stroke-width="1.4"/>${[0, 90, 180, 270].map((a) => `<path d="M0 -28l3 3l-3 3l-3 -3z" transform="rotate(${a})" fill="${C.goldHi}" stroke="none"/>`).join('')}</g>`;
  // Tokens: medallions.
  for (const u of UNITS) {
    const p = at(u.pos);
    const col = pc(u.owner);
    o += `<g transform="translate(${f1(p.x)} ${f1(p.y - 1)})">
      <circle r="16" cy="2" fill="rgba(0,0,0,.4)"/>
      <circle r="16" fill="${C.lapis}" stroke="${C.gold}" stroke-width="1"/>
      <circle r="13.2" fill="none" stroke="${col}" stroke-width="2.4"/>
      <use href="#i-unit-${u.kind.toLowerCase()}" x="-9" y="-9" width="18" height="18" style="color:#f3ecda"/>
      ${u.hp < 100 ? `<path d="M-13.2 0A13.2 13.2 0 0 1 13.2 0" transform="rotate(${f1(-90 + (180 * u.hp) / 100)})" fill="none" stroke="#0a1128" stroke-width="2.6" opacity=".8"/>` : ''}
      ${u.fortified ? `<use href="#i-fortify" x="8" y="-20" width="12" height="12" style="color:${C.goldHi}"/>` : ''}
    </g>`;
  }
  return o;
}

function banners() {
  return CITIES.map((ci) => {
    const p = at(ci.pos);
    const col = PLAYERS[ci.owner].color;
    const fog = tile(...ci.pos).vis === 'explored';
    return `<div class="plaque ${fog ? 'fog' : ''}" style="left:${f1(p.x)}px;top:${f1(p.y - R - 10)}px;--pc:${col}">
      <span class="q-pop num">${ci.pop}</span>
      <span class="q-name">${ci.capital ? star(8, 'q-cap') : ''}${ci.name}</span>
      <span class="q-turns num">${ci.prod ? ci.turns : '–'}</span>
    </div>`;
  }).join('');
}

function topbar() {
  const g = GAME;
  const pct = (g.research.progress / g.research.cost) * 100;
  return `<header class="top">
    <div class="tb-side">
      <svg class="logo" viewBox="0 0 236 50" width="116" height="25" aria-label="Legatum"><use href="#i-logo-legatum"/></svg>
      <div class="tb-cell">${icon('yield-gold', 16, 'ic', `color:${C.goldY}`)}<b class="num">${g.gold}</b><span class="pos num">+${g.goldPt}</span></div>
      <div class="tb-cell">${icon('yield-science', 16, 'ic', `color:${C.science}`)}<b class="num">+${g.sciencePt}</b></div>
    </div>
    <div class="turn">
      <span class="t-l">Turn</span><b class="num">${g.turn}</b><span class="t-l">of ${g.limit}</span>
    </div>
    <div class="tb-side right">
      <div class="tb-cell research"><span class="t-l">Research</span><span class="r-name">${g.research.name}</span><span class="r-bar"><i style="width:${pct}%"></i></span><span class="num r-t">${g.research.turns} turns</span></div>
      <button class="tb-btn">${icon('vp', 16)}</button>
      <button class="tb-btn">${icon('milestone-six-techs', 16)}</button>
      <button class="tb-btn menu"><span></span><span></span><span></span></button>
    </div>
  </header>`;
}

function vpPanel() {
  const rows = GAME.vp.map((t, i) => {
    const w = (t.vp / t.target) * 100;
    const fill = t.members.length > 1
      ? `linear-gradient(90deg, ${PLAYERS[t.members[0]].color} 0 50%, ${PLAYERS[t.members[1]].color} 50% 100%)`
      : PLAYERS[t.members[0]].color;
    const ticks = Array.from({ length: t.target - 1 }, (_, k) => `<i style="left:${((k + 1) / t.target) * 100}%"></i>`).join('');
    return `<div class="vp-row ${t.mine ? 'mine' : ''}">
      <span class="v-rank">${ROMAN[i]}</span>
      <span class="v-crest">${t.members.map((m) => crest(PLAYERS[m].faction, PLAYERS[m].color, 26)).join('')}</span>
      <span class="v-name">${t.label}</span>
      <span class="v-num num"><b>${t.vp}</b> / ${t.target}</span>
      <span class="v-track">${ticks}<span class="v-fill" style="width:${w}%;background:${fill}"></span></span>
    </div>`;
  }).join('');
  return `<section class="deco vp"><div class="in">
    <header class="ph">${fan(48)}${heading('Victory Race', 40)}</header>
    <div class="vp-rows">${rows}</div>
    <footer class="pf"><button class="link">Titles &amp; Milestones</button></footer>
  </div></section>`;
}

function cityPanel() {
  const c = GAME.city;
  const col = PLAYERS[0].color;
  const yields = YIELD_ORDER.map((k) => `<div class="y ${k === 'production' ? 'hot' : ''}">${icon('yield-' + k, 18, 'ic', `color:${YC[k]}`)}<b class="num">${c.yields[k]}</b><span class="y-l">${YIELD_LABEL[k]}</span></div>`).join('');
  const row = (o) => `<div class="prow ${o.active ? 'on' : ''}">${icon(o.icon, 18)}<span class="p-name">${o.key}${o.note ? `<em>${o.note}</em>` : ''}</span><span class="p-cost num">${o.cost}</span><span class="p-turns num">${o.turns}</span></div>`;
  return `<section class="deco city" style="--pc:${col}"><div class="in">
    <header class="c-head">
      <div class="c-kicker">${rule(28)}<span>Capital of the Japan</span>${ruleR(28)}</div>
      <h2>${c.name}</h2>
      <div class="c-meta"><span><span class="t-l">Strength</span><b class="num">${c.str}</b></span><span class="c-pop"><b class="num">${c.pop}</b></span><span><span class="t-l">Health</span><b class="num">${c.hp}</b></span></div>
    </header>
    <div class="yields">${yields}</div>
    <div class="c-block">
      <div class="c-line"><span class="t-l">Growth</span><span class="num">${c.food} / ${c.growAt} · ${c.growTurns} turns</span></div>
      <div class="bar"><i style="width:${(c.food / c.growAt) * 100}%;background:${C.food}"></i></div>
    </div>
    <div class="c-block">
      <div class="c-line"><span class="t-l">Producing</span><span class="num">${c.prodStored} / ${c.prodCost} · ${c.prodTurns} turns</span></div>
      <div class="now">${icon('unit-settler', 24, 'ic', `color:${C.goldHi}`)}<span class="now-name">${c.producing}</span><button class="btn buy">Buy ${icon('yield-gold', 13, 'ic', `color:${C.goldY}`)}<span class="num">${c.buy}</span></button></div>
      <div class="bar"><i style="width:${(c.prodStored / c.prodCost) * 100}%;background:${C.production}"></i></div>
    </div>
    <div class="list-h"><span>Units</span><span class="lh-r">${icon('yield-production', 12, 'ic', `color:${C.production}`)}<span>Turns</span></span></div>
    ${c.options.filter((o) => o.kind === 'unit').map(row).join('')}
    <div class="list-h"><span>Buildings</span></div>
    ${c.options.filter((o) => o.kind === 'building').map(row).join('')}
    <div class="focus"><div class="seg"><button class="on">Balanced</button><button>Growth</button><button>Production</button></div></div>
    <footer class="c-foot"><span>Monument · Granary</span><span class="num">${c.tiles} tiles · ${icon('yield-influence', 12, 'ic', `color:${C.influence}`)} ${c.influence} / ${c.nextTile}</span></footer>
  </div></section>`;
}

function tooltip() {
  const b = GAME.city.breakdown;
  return `<div class="tip"><div class="in">
    <div class="t-head">${icon('yield-production', 14, 'ic', `color:${C.production}`)}<span>${b.title}</span></div>
    ${b.rows.map(([l, v]) => `<div class="t-row"><span>${l}</span><i></i><span class="num ${v < 0 ? 'neg' : ''}">${v > 0 ? '+' : '−'}${Math.abs(v)}</span></div>`).join('')}
    <div class="t-row total"><span>Per turn</span><i></i><span class="num">${b.total}</span></div>
    <p class="t-note">${b.note}</p>
  </div></div>`;
}

function unitBar() {
  const u = GAME.unit;
  const col = PLAYERS[0].color;
  return `<section class="deco unit"><div class="in">
    <div class="u-id">
      <span class="u-med" style="--pc:${col}">${icon('unit-settler', 26, 'ic', 'color:#f3ecda')}</span>
      <div><h3>${u.kind}</h3><p>Ada · Civilian</p><p class="u-status">${u.status}</p></div>
    </div>
    <div class="u-stats">
      <div><span class="t-l">Health</span><span class="bar"><i style="width:100%;background:${C.food}"></i></span><b class="num">${u.hp}</b></div>
      <div><span class="t-l">Moves</span><span class="pips"><i class="on"></i><i class="on"></i></span><b class="num">${u.moves} / ${u.maxMoves}</b></div>
      <div><span class="t-l">Strength</span><span class="muted">—</span><b class="num">${u.strength}</b></div>
    </div>
    <div class="acts">
      <button class="btn primary">Found City<kbd>B</kbd></button>
      <button class="btn">Hold<kbd>F</kbd></button>
      <button class="btn">Skip<kbd>Space</kbd></button>
      <button class="btn quiet">Disband</button>
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
  return `<section class="deco seats"><div class="in">
    <header class="ph small">${heading('The Council', 36)}</header>
    ${seats}
    <button class="orders"><span>Pending orders</span><b class="num">3</b><span class="chev">▾</span></button>
    <div class="cta-wrap"><button class="cta"><span class="cta-l">${GAME.needing} Units Need Orders</span><span class="cta-s">Next unit<kbd>Space</kbd></span></button>
    <button class="ready-any">Ready anyway<kbd>Enter</kbd></button></div>
  </div></section>`;
}

export const TOKENS_C = `
  --lapis-0:#0a1128; --lapis-1:#0f1a3a; --lapis-2:#15234c; --lapis-3:#1c2d60;
  --gold:#d4b46a; --gold-hi:#f0d894; --gold-dim:rgba(212,180,106,.4); --gold-faint:rgba(212,180,106,.16);
  --ivory:#f3ecda; --ivory-2:#c9c6c4; --muted:#8e93ad; --good:#8fcf72; --bad:#ef7f6a;
  --display:'Bodoni Moda', 'Didot', serif; --sans:'Josefin Sans', 'Futura', sans-serif;
  --n:10px;`;

const CSS = `
:root{${TOKENS_C}}
*{box-sizing:border-box;margin:0;padding:0}
body{width:1440px;height:900px;overflow:hidden;background:#0a1128;color:var(--ivory);font:400 14px/1.35 var(--sans);font-feature-settings:"lnum" 1,"tnum" 1;-webkit-font-smoothing:antialiased;position:relative}
button{font:inherit;color:inherit;background:none;border:0;cursor:pointer}
.map{position:absolute;inset:0}
.muted{color:var(--muted)} .pos{color:var(--good)} .neg{color:var(--bad)}
.ic{display:inline-block;vertical-align:middle;flex:none}
.t-l{font:600 10px/1 var(--sans);letter-spacing:.22em;text-transform:uppercase;color:var(--gold)}
kbd{display:inline-block;margin-left:8px;padding:3px 4px 1px;min-width:16px;font:600 10px/1 var(--sans);letter-spacing:.06em;color:var(--ivory-2);border:1px solid var(--gold-dim);text-align:center}
.star{display:inline-block;color:var(--gold-hi);flex:none}
.rule{display:block;color:var(--gold);flex:none}
.fan{display:block;margin:0 auto 4px;color:var(--gold)}
.deco-h{display:flex;align-items:center;justify-content:center;gap:12px}
.deco-h span{font:600 12px/1 var(--sans);letter-spacing:.32em;text-transform:uppercase;color:var(--gold-hi);padding-top:2px}
.link{font:600 10px/1 var(--sans);letter-spacing:.22em;text-transform:uppercase;color:var(--gold);padding-bottom:4px;border-bottom:1px solid var(--gold-dim)}

/* Deco panel: notched corners, gold outer line, inset gold hairline. */
.deco{position:absolute;background:var(--gold-dim);padding:1px;
  clip-path:polygon(var(--n) 0,calc(100% - var(--n)) 0,calc(100% - var(--n)) var(--n),100% var(--n),100% calc(100% - var(--n)),calc(100% - var(--n)) calc(100% - var(--n)),calc(100% - var(--n)) 100%,var(--n) 100%,var(--n) calc(100% - var(--n)),0 calc(100% - var(--n)),0 var(--n),var(--n) var(--n))}
.deco>.in{position:relative;height:100%;background:var(--lapis-1);
  clip-path:polygon(var(--n) 0,calc(100% - var(--n)) 0,calc(100% - var(--n)) var(--n),100% var(--n),100% calc(100% - var(--n)),calc(100% - var(--n)) calc(100% - var(--n)),calc(100% - var(--n)) 100%,var(--n) 100%,var(--n) calc(100% - var(--n)),0 calc(100% - var(--n)),0 var(--n),var(--n) var(--n))}
.deco>.in::before{content:'';position:absolute;inset:4px;border:1px solid var(--gold-faint);pointer-events:none}
.ph{padding:16px 16px 12px;text-align:center}
.ph.small{padding:16px 16px 12px}
.pf{padding:8px 16px 20px;text-align:center}
.crest{display:inline-grid;place-items:center;flex:none;background:var(--lapis-0);border:1px solid var(--gold-dim);border-radius:50%;box-shadow:inset 0 0 0 2px var(--lapis-0),inset 0 0 0 3px color-mix(in srgb,var(--pc) 60%,transparent)}

/* Top bar: symmetric, with a stepped turn plaque. */
.top{position:absolute;left:0;right:0;top:0;height:52px;background:var(--lapis-0);border-bottom:1px solid var(--gold-dim);display:flex;align-items:center;justify-content:space-between;z-index:5}
.top::after{content:'';position:absolute;left:0;right:0;bottom:-4px;height:1px;background:var(--gold-faint)}
.tb-side{display:flex;align-items:center;height:100%;width:560px}
.tb-side.right{justify-content:flex-end}
.logo{color:var(--gold-hi);margin:0 24px}
.tb-cell{display:flex;align-items:center;gap:8px;height:100%;padding:0 20px;border-left:1px solid var(--gold-faint)}
.right .tb-cell{border-left:0;border-right:1px solid var(--gold-faint)}
.tb-cell b{font:600 17px/1 var(--sans);padding-top:3px}
.tb-cell .pos{font-size:13px;padding-top:3px}
.research{gap:12px}
.r-name{font:italic 500 17px/1 var(--display)}
.r-bar{width:80px;height:2px;background:var(--lapis-3)}
.r-bar i{display:block;height:100%;background:${C.science}}
.r-t{font:600 13px/1 var(--sans);color:var(--ivory-2);padding-top:2px}
.tb-btn{height:100%;width:52px;display:grid;place-items:center;border-right:1px solid var(--gold-faint);color:var(--gold)}
.menu{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:4px;border-right:0}
.menu span{display:block;width:16px;height:1px;background:var(--gold)}
.turn{position:absolute;left:50%;top:0;transform:translateX(-50%);width:208px;height:64px;display:flex;align-items:center;justify-content:center;gap:12px;background:var(--lapis-0);
  clip-path:polygon(0 0,100% 0,100% 44px,calc(100% - 16px) 44px,calc(100% - 16px) 54px,calc(100% - 32px) 54px,calc(100% - 32px) 64px,32px 64px,32px 54px,16px 54px,16px 44px,0 44px)}
.turn::before{content:'';position:absolute;inset:0;background:var(--gold-dim);
  clip-path:polygon(0 0,100% 0,100% 44px,calc(100% - 16px) 44px,calc(100% - 16px) 54px,calc(100% - 32px) 54px,calc(100% - 32px) 64px,32px 64px,32px 54px,16px 54px,16px 44px,0 44px,0 43px,17px 43px,17px 53px,33px 53px,33px 63px,calc(100% - 33px) 63px,calc(100% - 33px) 53px,calc(100% - 17px) 53px,calc(100% - 17px) 43px,100% 43px,100% 0)}
.turn b{font:500 32px/1 var(--display);color:var(--gold-hi);margin-top:-4px}
.turn .t-l{padding-top:2px}

/* Victory race */
.vp{right:16px;top:76px;width:320px}
.vp-rows{padding:4px 12px 8px}
.vp-row{display:grid;grid-template-columns:28px auto 1fr auto;grid-template-rows:22px 6px;column-gap:10px;row-gap:4px;align-items:center;padding:6px 8px}
.vp-row.mine{background:var(--lapis-2);box-shadow:inset 0 1px 0 var(--gold-dim),inset 0 -1px 0 var(--gold-dim)}
.v-rank{grid-row:1/3;text-align:center;font:500 15px/1 var(--display);color:var(--muted)}
.vp-row.mine .v-rank{color:var(--gold-hi)}
.v-crest{grid-row:1/3;display:flex}
.v-crest .crest+.crest{margin-left:-6px}
.v-name{font:400 14px/1 var(--sans);padding-top:3px;color:var(--ivory-2)}
.vp-row.mine .v-name{color:var(--ivory);font-weight:600}
.v-num{font:400 12px/1 var(--sans);color:var(--muted);padding-top:3px}
.v-num b{font:500 17px/1 var(--display);color:var(--ivory)}
.v-track{grid-column:3/5;position:relative;height:3px;background:var(--lapis-0)}
.v-track i{position:absolute;top:0;bottom:0;width:1px;background:var(--lapis-1)}
.v-fill{position:absolute;left:0;top:0;bottom:0}

/* City panel */
.city{left:16px;top:76px;width:352px}
.c-head{text-align:center;padding:24px 16px 16px}
.c-kicker{display:flex;align-items:center;justify-content:center;gap:12px}
.c-kicker span{font:600 10px/1 var(--sans);letter-spacing:.28em;text-transform:uppercase;color:var(--gold);padding-top:2px}
.c-head h2{margin-top:12px;font:500 46px/1 var(--display);letter-spacing:.01em;color:var(--ivory)}
.c-meta{display:grid;grid-template-columns:1fr auto 1fr;align-items:center;gap:16px;margin-top:12px}
.c-meta>span{display:flex;align-items:center;gap:8px}
.c-meta>span:first-child{justify-content:flex-end}
.c-meta b{font:600 15px/1 var(--sans);padding-top:3px}
.c-pop{width:36px;height:36px;display:grid !important;place-items:center;background:var(--pc);transform:rotate(45deg);box-shadow:0 0 0 2px var(--lapis-1),0 0 0 3px var(--gold)}
.c-pop b{transform:rotate(-45deg);font:600 18px/1 var(--display) !important;color:#08101f;padding:0 !important}
.yields{display:grid;grid-template-columns:repeat(5,1fr);margin:0 12px;border-top:1px solid var(--gold-dim);border-bottom:1px solid var(--gold-dim)}
.y{display:flex;flex-direction:column;align-items:center;gap:4px;padding:10px 0 8px;position:relative}
.y+.y::before{content:'';position:absolute;left:0;top:12px;bottom:12px;width:1px;background:var(--gold-faint)}
.y b{font:500 22px/1 var(--display)}
.y-l{font:600 9px/1 var(--sans);letter-spacing:.06em;text-transform:uppercase;color:var(--muted)}
.y.hot{background:var(--lapis-3)}
.y.hot::after{content:'';position:absolute;left:50%;bottom:-4px;width:7px;height:7px;background:var(--gold-hi);transform:translateX(-50%) rotate(45deg)}
.c-block{padding:16px 20px 0}
.c-line{display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;font-size:12px;color:var(--ivory-2)}
.bar{display:block;height:3px;background:var(--lapis-0)}
.bar i{display:block;height:100%}
.now{display:flex;align-items:center;gap:10px;margin-bottom:10px}
.now-name{font:500 24px/1 var(--display)}
.btn{display:inline-flex;align-items:center;justify-content:center;gap:4px;height:32px;padding:3px 12px 0;border:1px solid var(--gold-dim);background:var(--lapis-2);font:600 11px/1 var(--sans);letter-spacing:.16em;text-transform:uppercase;color:var(--ivory);white-space:nowrap}
.btn.buy{margin-left:auto;height:28px;color:var(--gold-hi)}
.btn.primary{background:var(--gold);border-color:var(--gold-hi);color:#0a1128;box-shadow:inset 0 0 0 2px var(--gold),inset 0 0 0 3px rgba(10,17,40,.35)}
.btn.primary kbd{color:#0a1128;border-color:rgba(10,17,40,.35)}
.btn.quiet{border-color:transparent;background:none;color:#ef9d8a}
.list-h{display:flex;justify-content:space-between;align-items:center;margin:20px 20px 4px;padding-bottom:6px;border-bottom:1px solid var(--gold-faint);font:600 10px/1 var(--sans);letter-spacing:.28em;text-transform:uppercase;color:var(--gold)}
.lh-r{display:flex;align-items:center;gap:16px;letter-spacing:.1em;color:var(--muted)}
.prow{display:grid;grid-template-columns:18px 1fr 40px 32px;gap:12px;align-items:center;height:30px;margin:0 12px;padding:0 8px;color:var(--ivory-2)}
.prow .ic{color:var(--ivory-2)}
.p-name{font:400 15px/1 var(--sans);padding-top:3px;color:var(--ivory);display:flex;align-items:baseline;gap:8px}
.p-name em{font:italic 400 12px/1 var(--display);color:var(--muted)}
.p-cost,.p-turns{text-align:right;font-size:13px;padding-top:3px}
.prow.on{background:var(--lapis-3);box-shadow:inset 0 1px 0 var(--gold-dim),inset 0 -1px 0 var(--gold-dim)}
.prow.on .p-name{color:var(--gold-hi);font-weight:600}
.prow.on .ic{color:var(--gold-hi)}
.focus{padding:20px 20px 0}
.seg{display:flex;border:1px solid var(--gold-dim)}
.seg button{flex:1;height:32px;padding-top:3px;font:600 10px/1 var(--sans);letter-spacing:.2em;text-transform:uppercase;color:var(--ivory-2)}
.seg button+button{border-left:1px solid var(--gold-faint)}
.seg .on{background:var(--gold);color:#0a1128}
.c-foot{display:flex;justify-content:space-between;margin-top:16px;padding:14px 20px 20px;border-top:1px solid var(--gold-faint);font-size:12px;color:var(--ivory-2)}

/* Tooltip */
.tip{position:absolute;left:380px;top:156px;width:272px;background:var(--gold);padding:1px}
.tip>.in{background:var(--lapis-0);padding:12px 16px}
.tip::before{content:'';position:absolute;left:-6px;top:26px;width:11px;height:11px;background:var(--gold);transform:rotate(45deg);z-index:-1}
.t-head{display:flex;align-items:center;justify-content:center;gap:8px;padding-bottom:10px;margin-bottom:4px;border-bottom:1px solid var(--gold-dim);font:600 11px/1 var(--sans);letter-spacing:.28em;text-transform:uppercase;color:var(--gold-hi)}
.t-row{display:flex;align-items:baseline;gap:8px;height:24px;line-height:26px;font-size:13px;color:var(--ivory-2)}
.t-row i{flex:1;border-bottom:1px dotted rgba(212,180,106,.3);transform:translateY(-6px)}
.t-row .num{min-width:24px;text-align:right;color:var(--ivory);font-weight:600}
.t-row.total{margin-top:4px;border-top:1px solid var(--gold-dim);height:32px;line-height:34px;color:var(--gold);font:600 10px/34px var(--sans);letter-spacing:.22em;text-transform:uppercase}
.t-row.total i{border:0}
.t-row.total .num{font:500 20px/32px var(--display);color:var(--gold-hi);letter-spacing:0}
.t-note{margin-top:4px;font:italic 400 12px/16px var(--display);color:var(--muted)}

/* Unit bar */
.unit{left:384px;bottom:16px;width:712px;height:104px}
.unit>.in{display:grid;grid-template-columns:216px 176px 1fr;align-items:center}
.u-id{display:flex;align-items:center;gap:12px;padding:0 12px 0 20px}
.u-med{width:52px;height:52px;display:grid;place-items:center;flex:none;border-radius:50%;background:var(--lapis-0);border:1px solid var(--gold);box-shadow:inset 0 0 0 3px var(--lapis-0),inset 0 0 0 5px var(--pc)}
.u-id h3{font:500 26px/1 var(--display)}
.u-id p{margin-top:4px;font-size:12px;color:var(--ivory-2)}
.u-status{font:600 9px/1 var(--sans) !important;letter-spacing:.22em;text-transform:uppercase;color:var(--gold-hi) !important}
.u-stats{display:flex;flex-direction:column;gap:6px;padding:0 16px;height:72px;justify-content:center;border-left:1px solid var(--gold-faint);border-right:1px solid var(--gold-faint)}
.u-stats div{display:grid;grid-template-columns:64px 1fr auto;gap:8px;align-items:center;height:16px}
.u-stats .t-l{color:var(--muted);font-size:9px}
.u-stats b{font-size:13px;padding-top:3px}
.pips{display:flex;gap:6px}
.pips i{width:7px;height:7px;border:1px solid var(--gold);transform:rotate(45deg)}
.pips i.on{background:var(--gold)}
.acts{display:grid;grid-template-columns:1fr 1fr 1fr;grid-template-rows:36px 30px;gap:8px;padding:0 16px}
.acts .btn{height:auto;padding:3px 8px 0}
.acts kbd{margin-left:6px}
.acts .primary{grid-column:1/4}
.u-hint{position:absolute;left:384px;width:712px;bottom:128px;text-align:center;font:italic 400 14px/1 var(--display);color:rgba(243,236,218,.8);text-shadow:0 1px 2px #000}

/* Seats */
.seats{right:16px;bottom:16px;width:320px}
.seat{display:grid;grid-template-columns:40px 1fr auto auto;gap:12px;align-items:center;height:60px;margin:0 12px;padding:0 8px;border-top:1px solid var(--gold-faint)}
.seat.on{background:var(--lapis-2);box-shadow:inset 2px 0 0 var(--pc)}
.s-id{display:flex;flex-direction:column;gap:4px}
.s-id b{font:500 20px/1 var(--display)}
.s-id span{font-size:11px;color:var(--muted)}
.s-state{font:600 9px/1 var(--sans);letter-spacing:.2em;text-transform:uppercase;color:var(--ivory-2);padding:4px 6px 2px;border:1px solid var(--gold-faint)}
.seat.ready .s-state{color:var(--good);border-color:rgba(143,207,114,.4)}
.seat kbd{margin:0}
.orders{display:flex;align-items:center;gap:8px;width:calc(100% - 24px);height:40px;margin:0 12px;padding:3px 8px 0;font-size:12px;color:var(--ivory-2);border-top:1px solid var(--gold-faint);border-bottom:1px solid var(--gold-faint)}
.orders b{color:var(--ivory)}
.orders .chev{margin-left:auto;color:var(--gold)}
.cta-wrap{padding:16px 20px 12px}
.cta{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:6px;width:100%;height:64px;background:var(--gold);color:#0a1128;box-shadow:inset 0 0 0 3px var(--gold),inset 0 0 0 4px rgba(10,17,40,.4);
  clip-path:polygon(8px 0,calc(100% - 8px) 0,calc(100% - 8px) 8px,100% 8px,100% calc(100% - 8px),calc(100% - 8px) calc(100% - 8px),calc(100% - 8px) 100%,8px 100%,8px calc(100% - 8px),0 calc(100% - 8px),0 8px,8px 8px)}
.cta-l{font:500 21px/1 var(--display)}
.cta-s{font:600 10px/1 var(--sans);letter-spacing:.22em;text-transform:uppercase;color:#3b3420;display:flex;align-items:center}
.cta kbd{color:#3b3420;border-color:rgba(10,17,40,.3)}
.ready-any{display:flex;align-items:center;justify-content:center;width:100%;height:28px;margin-top:4px;font-size:12px;color:var(--ivory-2)}

/* City plaques on the map */
.plaque{position:absolute;transform:translate(-50%,-100%);display:flex;align-items:center;height:26px;padding:0 10px 0 4px;background:rgba(10,17,40,.94);border-top:1px solid var(--gold);border-bottom:1px solid var(--gold);white-space:nowrap}
.plaque::before,.plaque::after{content:'';position:absolute;top:3px;bottom:3px;width:1px;background:var(--gold)}
.plaque::before{left:-4px} .plaque::after{right:-4px}
.plaque.fog{opacity:.85}
.q-pop{display:grid;place-items:center;width:18px;height:18px;margin-right:4px;background:var(--pc);color:#08101f;font:700 11px/1 var(--sans);padding-top:2px;transform:rotate(45deg) scale(.85)}
.q-pop{transform:none;clip-path:polygon(50% 0,100% 50%,50% 100%,0 50%);width:22px;height:22px}
.q-name{display:flex;align-items:center;gap:6px;padding:2px 8px 0;font:600 11px/1 var(--sans);letter-spacing:.24em;text-transform:uppercase;color:var(--ivory)}
.q-cap{color:var(--gold-hi)}
.q-turns{padding:2px 0 0 8px;border-left:1px solid var(--gold-dim);font:500 13px/1 var(--display);color:var(--gold-hi)}
.map-label{position:absolute;font:italic 400 18px/1 var(--display);letter-spacing:.24em;color:rgba(240,216,148,.62);white-space:nowrap}
.map-label.land{font:600 11px/1 var(--sans);letter-spacing:.6em;text-transform:uppercase;color:rgba(243,236,218,.5)}
`;

export function pages() {
  const body = `${iconSprite(ALL_ICONS)}
  ${renderMap(MAP_C, overlay())}
  <div class="map-label" style="left:398px;top:474px">The Sundering Sea</div>
  <div class="map-label land" style="left:884px;top:196px;transform:rotate(58deg);transform-origin:left">Spine of Tesk</div>
  <div class="map-label land" style="left:896px;top:656px">Verdance</div>
  ${banners()}
  ${topbar()}
  ${cityPanel()}
  ${tooltip()}
  ${unitBar()}
  ${vpPanel()}
  ${seatPanel()}`;
  return {
    'variant-c.html': page({
      title: "Legatum: Heir's Court",
      fonts: 'family=Bodoni+Moda:ital,opsz,wght@0,6..96,400..700;1,6..96,400..600&family=Josefin+Sans:wght@400;600;700',
      css: CSS,
      body,
    }),
  };
}
