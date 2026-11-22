// Lobby / staging room in direction C (Heir's Court): seats table with crests at full size, game settings, map preview.
import { ALL_ICONS, PLAYERS, f1, icon, iconSprite, page, renderMap } from './shared.mjs';
import { MAP_C, STAR, TOKENS_C } from './variant-c.mjs';

const star = (s = 10) => `<svg class="star" width="${s}" height="${s}" viewBox="-5 -5 10 10" aria-hidden="true"><path d="${STAR}" fill="currentColor"/></svg>`;
const rule = (w, flip) => `<svg class="rule" width="${w}" height="8" viewBox="0 0 ${w} 8" aria-hidden="true">${flip ? `<path d="M10 4H${w}" stroke="currentColor"/><path d="M0 4l4 -4l4 4l-4 4z" fill="currentColor"/>` : `<path d="M0 4H${w - 10}" stroke="currentColor"/><path d="M${w - 8} 4l4 -4l4 4l-4 4z" fill="currentColor"/>`}</svg>`;
const heading = (t, w = 56) => `<div class="deco-h">${rule(w)}<span>${t}</span>${rule(w, true)}</div>`;

const SEATS = [
  { p: 0, name: 'Ada', tag: 'Host', type: 'Human', team: 1, diff: '', ready: true },
  { p: 1, name: 'Bo', tag: 'Player', type: 'Human', team: 1, diff: '', ready: false },
  { p: 2, name: 'Hollow Crown', type: 'AI', team: 2, diff: 'Governor' },
  { p: 3, name: 'Saint Veyra', type: 'AI', team: 3, diff: 'Governor' },
  { p: 4, name: 'The Meridian', type: 'AI', team: 4, diff: 'Consul' },
  { p: 5, name: 'Ashfall Compact', type: 'AI', team: 5, diff: 'Governor' },
  { p: 6, name: 'Orrery of Tesk', type: 'AI', team: 6, diff: 'Consul' },
  { p: 7, name: '', type: 'Closed', team: null, diff: '' },
];

const CSS = `
:root{${TOKENS_C}}
*{box-sizing:border-box;margin:0;padding:0}
body{width:1440px;height:900px;overflow:hidden;background:#0a1128;color:var(--ivory);font:400 14px/1.35 var(--sans);font-feature-settings:"lnum" 1,"tnum" 1;-webkit-font-smoothing:antialiased;position:relative}
button{font:inherit;color:inherit;background:none;border:0;cursor:pointer}
.ic{display:inline-block;vertical-align:middle;flex:none}
.star{display:inline-block;color:var(--gold-hi)}
.rule{display:block;color:var(--gold);flex:none}
.deco-h{display:flex;align-items:center;justify-content:center;gap:12px}
.deco-h span{font:600 12px/1 var(--sans);letter-spacing:.32em;text-transform:uppercase;color:var(--gold-hi);padding-top:2px}
.t-l{font:600 10px/1 var(--sans);letter-spacing:.22em;text-transform:uppercase;color:var(--gold)}
.bg{position:absolute;inset:0;background:radial-gradient(ellipse 70% 60% at 50% 40%, #13214a 0%, #0a1128 70%)}
.bg::after{content:'';position:absolute;inset:0;background:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='48' height='24'%3E%3Cpath d='M0 24a24 24 0 0 1 48 0M-24 12a24 24 0 0 1 48 0M24 12a24 24 0 0 1 48 0' fill='none' stroke='rgba(212,180,106,.05)'/%3E%3C/svg%3E")}
.top{position:absolute;left:0;right:0;top:0;height:64px;display:flex;align-items:center;justify-content:space-between;padding:0 32px;border-bottom:1px solid var(--gold-dim)}
.top::after{content:'';position:absolute;left:0;right:0;bottom:-4px;height:1px;background:var(--gold-faint)}
.logo{color:var(--gold-hi)}
.title{position:absolute;left:50%;transform:translateX(-50%);text-align:center}
.title h1{font:500 28px/1 var(--display)}
.title p{margin-top:4px;font:600 10px/1 var(--sans);letter-spacing:.3em;text-transform:uppercase;color:var(--gold)}
.leave{font:600 11px/1 var(--sans);letter-spacing:.24em;text-transform:uppercase;color:var(--ivory-2);padding:10px 16px 8px;border:1px solid var(--gold-faint)}

.deco{position:absolute;background:var(--gold-dim);padding:1px;
  clip-path:polygon(var(--n) 0,calc(100% - var(--n)) 0,calc(100% - var(--n)) var(--n),100% var(--n),100% calc(100% - var(--n)),calc(100% - var(--n)) calc(100% - var(--n)),calc(100% - var(--n)) 100%,var(--n) 100%,var(--n) calc(100% - var(--n)),0 calc(100% - var(--n)),0 var(--n),var(--n) var(--n))}
.deco>.in{position:relative;height:100%;background:var(--lapis-1);
  clip-path:polygon(var(--n) 0,calc(100% - var(--n)) 0,calc(100% - var(--n)) var(--n),100% var(--n),100% calc(100% - var(--n)),calc(100% - var(--n)) calc(100% - var(--n)),calc(100% - var(--n)) 100%,var(--n) 100%,var(--n) calc(100% - var(--n)),0 calc(100% - var(--n)),0 var(--n),var(--n) var(--n))}
.deco>.in::before{content:'';position:absolute;inset:4px;border:1px solid var(--gold-faint);pointer-events:none}
.ph{padding:20px 16px 16px}

.seats{left:32px;top:96px;width:960px;height:772px}
.thead,.row{display:grid;grid-template-columns:32px 64px 1fr 120px 168px 48px 120px 96px;align-items:center;column-gap:8px;padding:0 24px}
.thead{height:32px;border-bottom:1px solid var(--gold-dim);margin:0 12px}
.thead span{font:600 9px/1 var(--sans);letter-spacing:.22em;text-transform:uppercase;color:var(--muted)}
.row{height:76px;margin:0 12px;border-bottom:1px solid var(--gold-faint)}
.row.me{background:var(--lapis-2);box-shadow:inset 2px 0 0 var(--pc)}
.row.closed{opacity:.5}
.n{font:500 18px/1 var(--display);color:var(--muted);text-align:center}
.medal{position:relative;width:60px;height:60px;display:grid;place-items:center;border-radius:50%;background:#0a1128;border:1px solid var(--gold);box-shadow:inset 0 0 0 3px #0a1128,inset 0 0 0 4px var(--gold-dim)}
.medal::after{content:'';position:absolute;inset:7px;border-radius:50%;border:2px solid var(--pc);opacity:.8}
.medal.empty{border-style:dashed;border-color:var(--gold-dim)}
.medal.empty::after{display:none}
.who b{display:block;font:500 20px/1.1 var(--display);white-space:nowrap}
.who span{display:block;margin-top:4px;font:600 9px/1 var(--sans);letter-spacing:.22em;text-transform:uppercase;color:var(--gold)}
.who span.ai{color:var(--muted)}
.sel{display:flex;align-items:center;justify-content:space-between;height:36px;padding:3px 12px 0;border:1px solid var(--gold-faint);background:var(--lapis-0);font:400 14px/1 var(--sans);color:var(--ivory)}
.sel::after{content:'';width:6px;height:6px;border-right:1px solid var(--gold);border-bottom:1px solid var(--gold);transform:translateY(-3px) rotate(45deg)}
.sel .fx{display:flex;align-items:center;gap:8px}
.sel.dis{color:var(--muted);border-color:transparent;background:none}
.sel.dis::after{display:none}
.teamchip{display:inline-grid;place-items:center;width:36px;height:36px;padding-top:3px;font:500 18px/1 var(--display);border:1px solid var(--gold-faint)}
.state{font:600 10px/1 var(--sans);letter-spacing:.2em;text-transform:uppercase;text-align:center;padding:6px 0 4px;border:1px solid var(--gold-faint);color:var(--ivory-2)}
.state.ready{color:var(--good);border-color:rgba(143,207,114,.45)}
.state.ai{border-color:transparent;color:var(--muted)}
.add{display:flex;justify-content:center;gap:24px;padding:20px 0 0}
.add button{font:600 10px/1 var(--sans);letter-spacing:.24em;text-transform:uppercase;color:var(--gold);padding-bottom:4px;border-bottom:1px solid var(--gold-dim)}

.side{right:32px;top:96px;width:400px;height:772px}
.preview{position:relative;margin:0 24px;height:228px;overflow:hidden;border:1px solid var(--gold-dim)}
.preview .map-wrap{position:absolute;left:0;top:0;width:1440px;height:900px;transform:scale(.46);transform-origin:-330px -230px}
.preview-cap{display:flex;justify-content:space-between;margin:8px 24px 0;font:italic 400 14px/1 var(--display);color:var(--ivory-2)}
.settings{margin:20px 24px 0}
.set{display:grid;grid-template-columns:112px 1fr;align-items:center;height:48px;border-bottom:1px solid var(--gold-faint)}
.set .v{display:flex;align-items:center;justify-content:space-between;font:400 15px/1 var(--sans);padding-top:3px}
.set .v em{font:italic 400 14px/1 var(--display);color:var(--muted)}
.seed{font:500 14px/1 var(--sans);letter-spacing:.08em;color:var(--gold-hi)}
.actions{position:absolute;left:24px;right:24px;bottom:24px;display:flex;flex-direction:column;gap:12px}
.start{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:8px;height:72px;background:var(--gold);color:#0a1128;box-shadow:inset 0 0 0 3px var(--gold),inset 0 0 0 4px rgba(10,17,40,.4);
  clip-path:polygon(10px 0,calc(100% - 10px) 0,calc(100% - 10px) 10px,100% 10px,100% calc(100% - 10px),calc(100% - 10px) calc(100% - 10px),calc(100% - 10px) 100%,10px 100%,10px calc(100% - 10px),0 calc(100% - 10px),0 10px,10px 10px)}
.start b{font:500 24px/1 var(--display)}
.start span{font:600 10px/1 var(--sans);letter-spacing:.2em;text-transform:uppercase;color:#3b3420}
.ready-row{display:flex;gap:12px}
.ready-row button{flex:1;height:40px;padding-top:3px;font:600 11px/1 var(--sans);letter-spacing:.22em;text-transform:uppercase;border:1px solid var(--gold-dim);color:var(--ivory)}
.ready-row .on{background:var(--lapis-3);color:var(--good);border-color:rgba(143,207,114,.45)}
`;

function seatRows() {
  return SEATS.map((s, i) => {
    const p = PLAYERS[s.p];
    const closed = s.type === 'Closed';
    const col = closed ? 'transparent' : p.color;
    const medal = closed
      ? `<span class="medal empty"></span>`
      : `<span class="medal" style="--pc:${col}">${icon('faction-' + p.faction.toLowerCase(), 38, 'ic', `color:${col}`)}</span>`;
    const who = closed
      ? `<div class="who"><b class="muted" style="color:var(--muted)">Closed</b></div>`
      : `<div class="who"><b>${s.name}</b><span class="${s.type === 'AI' ? 'ai' : ''}">${s.type === 'AI' ? 'AI empire' : s.tag}</span></div>`;
    return `<div class="row ${i === 0 ? 'me' : ''} ${closed ? 'closed' : ''}" style="--pc:${col}">
      <span class="n">${['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII'][i]}</span>
      ${medal}
      ${who}
      <span class="sel ${i === 0 ? 'dis' : ''}">${s.type}</span>
      <span class="sel ${closed ? 'dis' : ''}"><span class="fx">${closed ? '—' : `The ${p.faction}`}</span></span>
      <span>${s.team ? `<span class="teamchip">${s.team}</span>` : ''}</span>
      <span class="sel ${s.type !== 'AI' ? 'dis' : ''}">${s.diff || '—'}</span>
      <span class="state ${s.ready ? 'ready' : ''} ${s.type !== 'Human' ? 'ai' : ''}">${s.type === 'Human' ? (s.ready ? 'Ready' : 'Choosing') : closed ? '' : 'AI'}</span>
    </div>`;
  }).join('');
}

export function pages() {
  const body = `${iconSprite(ALL_ICONS)}
  <div class="bg"></div>
  <header class="top">
    <svg class="logo" viewBox="0 0 236 50" width="112" height="24" aria-label="Legatum"><use href="#i-logo-legatum"/></svg>
    <div class="title"><h1>New Game</h1><p>Multiplayer · Hosted on this computer</p></div>
    <button class="leave">Leave lobby</button>
  </header>
  <section class="deco seats"><div class="in">
    <header class="ph">${heading('Players', 64)}</header>
    <div class="thead"><span></span><span>Crest</span><span>Player</span><span>Seat</span><span>Faction</span><span>Team</span><span>Difficulty</span><span>Status</span></div>
    ${seatRows()}
    <div class="add"><button>Open all seats</button><button>Balance teams</button><button>Randomize factions</button></div>
  </div></section>
  <section class="deco side"><div class="in">
    <header class="ph">${heading('The World', 56)}</header>
    <div class="preview"><div class="map-wrap">${renderMap({ ...MAP_C, vignette: 0.3 }, '')}</div></div>
    <div class="preview-cap"><span>Continents · temperate</span><span>48 × 32</span></div>
    <div class="settings">
      <div class="set"><span class="t-l">Map size</span><span class="v">Standard <em>48 × 32 · 8 empires</em></span></div>
      <div class="set"><span class="t-l">Seed</span><span class="v"><span class="seed">ORRERY-7731</span><em>Reroll</em></span></div>
      <div class="set"><span class="t-l">Game length</span><span class="v">Standard <em>150 turns</em></span></div>
      <div class="set"><span class="t-l">Victory</span><span class="v">10 VP <em>17 for a two-empire team</em></span></div>
      <div class="set"><span class="t-l">Turns</span><span class="v">Simultaneous <em>no timer</em></span></div>
    </div>
    <div class="actions">
      <div class="ready-row"><button class="on">${star(8)} Ready</button><button>Invite players</button></div>
      <button class="start"><b>Begin the Game</b><span>Waiting for 1 player to ready</span></button>
    </div>
  </div></section>`;
  return {
    'lobby-c.html': page({
      title: "Legatum: Lobby (Heir's Court)",
      fonts: 'family=Bodoni+Moda:ital,opsz,wght@0,6..96,400..700;1,6..96,400..600&family=Josefin+Sans:wght@400;600;700',
      css: CSS,
      body,
    }),
  };
}
