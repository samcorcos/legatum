// Title screen in direction C (Heir's Court): the wordmark as hero over the world, standard main menu.
import { ALL_ICONS, f1, iconSprite, page, renderMap } from './shared.mjs';
import { MAP_C, STAR, TOKENS_C } from './variant-c.mjs';

const fan = (w, rays = 13) => `<svg class="fan" width="${w}" height="${w / 2}" viewBox="-40 -40 80 40" aria-hidden="true">
  ${Array.from({ length: rays }, (_, i) => { const a = Math.PI * (i + 1) / (rays + 1); const r2 = i % 2 ? 30 : 38; return `<line x1="${f1(-Math.cos(a) * 13)}" y1="${f1(-Math.sin(a) * 13)}" x2="${f1(-Math.cos(a) * r2)}" y2="${f1(-Math.sin(a) * r2)}" stroke="currentColor" stroke-width=".7"/>`; }).join('')}
  <path d="M-10 0a10 10 0 0 1 20 0" fill="none" stroke="currentColor" stroke-width=".7"/>
  <path d="${STAR}" transform="translate(0 -1.5) scale(1.1)" fill="currentColor"/></svg>`;


const CSS = `
:root{${TOKENS_C}}
*{box-sizing:border-box;margin:0;padding:0}
body{width:1440px;height:900px;overflow:hidden;background:#0a1128;color:var(--ivory);font:400 14px/1.35 var(--sans);font-feature-settings:"lnum" 1,"tnum" 1;-webkit-font-smoothing:antialiased;position:relative}
button{font:inherit;color:inherit;background:none;border:0;cursor:pointer}
.ic{display:inline-block;vertical-align:middle;flex:none}
.world{position:absolute;inset:0;transform:scale(1.22) translate(40px,10px);transform-origin:60% 45%;filter:saturate(.85) brightness(.62)}
.veil{position:absolute;inset:0;background:
  radial-gradient(ellipse 46% 60% at 50% 42%, rgba(10,17,40,.86) 0%, rgba(10,17,40,.7) 45%, rgba(10,17,40,.25) 100%),
  linear-gradient(180deg, rgba(10,17,40,.55) 0%, rgba(10,17,40,0) 30%, rgba(10,17,40,0) 62%, rgba(10,17,40,.95) 100%)}
.frame{position:absolute;inset:24px;border:1px solid var(--gold-dim);pointer-events:none}
.frame::before{content:'';position:absolute;inset:5px;border:1px solid var(--gold-faint)}
.corner{position:absolute;width:24px;height:24px;background:#0a1128;border:1px solid var(--gold-dim)}
.corner::after{content:'';position:absolute;left:50%;top:50%;width:6px;height:6px;background:var(--gold);transform:translate(-50%,-50%) rotate(45deg)}
.c-tl{left:12px;top:12px}.c-tr{right:12px;top:12px}.c-bl{left:12px;bottom:12px}.c-br{right:12px;bottom:12px}
.hero{position:absolute;left:0;right:0;top:88px;display:flex;flex-direction:column;align-items:center;text-align:center}
.fan{display:block;color:var(--gold)}
.mark{color:var(--gold-hi);margin-top:20px}
.kicker{display:flex;align-items:center;gap:16px;margin-top:28px;font:600 12px/1 var(--sans);letter-spacing:.5em;text-transform:uppercase;color:var(--gold)}
.kicker i{display:block;width:72px;height:1px;background:var(--gold-dim)}
.tag{margin-top:16px;font:italic 400 22px/1.3 var(--display);color:var(--ivory-2)}
.menu{position:absolute;left:50%;top:436px;width:360px;transform:translateX(-50%);display:flex;flex-direction:column;align-items:stretch;gap:8px}
.continue{display:flex;flex-direction:column;align-items:center;gap:8px;height:80px;justify-content:center;background:var(--gold);color:#0a1128;box-shadow:inset 0 0 0 3px var(--gold),inset 0 0 0 4px rgba(10,17,40,.4);
  clip-path:polygon(10px 0,calc(100% - 10px) 0,calc(100% - 10px) 10px,100% 10px,100% calc(100% - 10px),calc(100% - 10px) calc(100% - 10px),calc(100% - 10px) 100%,10px 100%,10px calc(100% - 10px),0 calc(100% - 10px),0 10px,10px 10px)}
.continue b{font:500 26px/1 var(--display)}
.continue span{font:600 10px/1 var(--sans);letter-spacing:.2em;text-transform:uppercase;color:#3b3420}
.item{display:flex;align-items:center;justify-content:center;gap:16px;height:44px;padding-top:3px;font:600 13px/1 var(--sans);letter-spacing:.36em;text-transform:uppercase;color:var(--ivory-2)}
.item.hover{color:var(--gold-hi)}
.item.hover::before,.item.hover::after{content:'';width:48px;height:1px;background:linear-gradient(90deg,transparent,var(--gold))}
.item.hover::after{background:linear-gradient(270deg,transparent,var(--gold))}
.sep{height:1px;margin:8px 96px;background:var(--gold-faint)}
.sub{display:flex;align-items:center;justify-content:center;gap:16px;height:32px;margin-top:-8px}
.sub button{padding-top:3px;font:600 11px/1 var(--sans);letter-spacing:.24em;text-transform:uppercase;color:var(--ivory)}
.sub i{width:5px;height:5px;background:var(--gold-dim);transform:rotate(45deg)}
.foot{position:absolute;left:80px;right:80px;bottom:44px;display:flex;justify-content:space-between;font:600 10px/1 var(--sans);letter-spacing:.24em;text-transform:uppercase;color:var(--muted)}
`;

export function pages() {
  const body = `${iconSprite(ALL_ICONS)}
  <div class="world">${renderMap({ ...MAP_C, vignette: 0 }, '')}</div>
  <div class="veil"></div>
  <div class="frame"><span class="corner c-tl"></span><span class="corner c-tr"></span><span class="corner c-bl"></span><span class="corner c-br"></span></div>
  <div class="hero">
    ${fan(120)}
    <svg class="mark" viewBox="0 0 236 50" width="560" height="119" aria-label="Legatum"><use href="#i-logo-legatum"/></svg>
    <div class="kicker"><i></i>The Heirs of the Astrarchy<i></i></div>
    <p class="tag">The star empire has fallen. Its heirs remain.</p>
  </div>
  <nav class="menu">
    <button class="continue"><b>Continue</b><span>Turn 27 of 150 · autosave</span></button>
    <button class="item">Single Player</button>
    <button class="item hover">Multiplayer</button>
    <div class="sub"><button>Host</button><i></i><button>Join</button><i></i><button>Hot Seat</button></div>
    <button class="item">Load Game</button>
    <div class="sep"></div>
    <button class="item">Settings</button>
    <button class="item">Quit</button>
  </nav>
  <div class="foot"><span>Build 0.4</span><span>© 2026</span></div>`;
  return {
    'title-c.html': page({
      title: 'Legatum: Title (Heir\'s Court)',
      fonts: 'family=Bodoni+Moda:ital,opsz,wght@0,6..96,400..700;1,6..96,400..600&family=Josefin+Sans:wght@400;600;700',
      css: CSS,
      body,
    }),
  };
}
