// The staged reveal at the end of a game (legatum-ui "The end of a game is an event"): over the hero's era
// painting, the winner's leader rises in an engraved frame, the embossed crest is pressed onto it like a seal,
// the ribbon unfurls, "Victory" or "Defeat" engraves in, the reason follows, and the final standings count up.
// About five seconds; a click, Return, Space or Esc finishes it at once, and under reduced motion it simply
// stands finished. Base styles are the settled state: every animation runs "backwards" from hidden.
import { useEffect, useRef, useState } from 'react';
import { Crest, Star, Wordmark } from '../brand/ornaments';
import { eraArt } from '../celebrate/eraArt';
import { playSound } from '../feel';
import { playerColor } from '../theme';
import type { PlayerPublic } from '../types';
import type { TeamTrack } from '../vp';
import { Amounts } from './Amounts';
import { PortraitFrame, PortraitRoundel, Ribbon, StarDivider } from './premium';
import { useReducedMotion } from './useReducedMotion';
import { useCombatFx } from '../feel';
import { motionStill } from './motion';

export type RevealTone = 'victory' | 'lost' | 'fallen' | 'draw';

export interface RevealAction { label: string; primary?: boolean; disabled?: boolean; onClick(): void }

/** When the standings start counting, how long they count, and when the whole reveal has settled (ms). */
const COUNT_START_MS = 3000;
const COUNT_MS = 1400;
const SETTLE_MS = 5200;
/** The crest is pressed on (and the victory india sings) at this moment. */
const STAMP_MS = 1050;

/** Gold motes drifting up on a victory: fixed positions, so the scene is the same every time. */
const MOTES = Array.from({ length: 16 }, (_, i) => ({
  x: (i * 41 + 7) % 100, size: 2 + ((i * 7) % 4), delay: 1.1 + ((i * 0.31) % 2.2), rise: 180 + ((i * 47) % 240), sway: ((i * 23) % 44) - 22,
}));

const easeOut = (p: number) => 1 - (1 - p) ** 3;

export function GameReveal({
  tracks, winner, hero, titleId, title, ribbon, reason, finish = null, kicker, tone, settled, onSettle, actions,
}: {
  tracks: TeamTrack[];
  winner: TeamTrack | null;
  hero: PlayerPublic[];
  titleId: string;
  title: string;
  ribbon: string;
  reason: string | null;
  /** For a loss: where you finished and the gap ("You finished 2nd, 1 VP short."). */
  finish?: string | null;
  kicker: string;
  tone: RevealTone;
  settled: boolean;
  onSettle(): void;
  actions: RevealAction[];
}) {
  const reduced = useReducedMotion();
  const animationsOn = useCombatFx((s) => s.enabled);
  const still = motionStill(reduced, animationsOn) || settled;
  const [count, setCount] = useState(still ? 1 : 0);
  const sang = useRef(false);
  // The count-up runs on its own clock so the numbers land with the bars; settling jumps to the end.
  useEffect(() => {
    if (still) { setCount(1); if (!settled) onSettle(); return; }
    let frame = 0;
    const start = performance.now() + COUNT_START_MS;
    const tick = (now: number) => {
      const p = Math.max(0, Math.min(1, (now - start) / COUNT_MS));
      setCount(easeOut(p));
      if (p < 1) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    const done = setTimeout(onSettle, SETTLE_MS);
    return () => { cancelAnimationFrame(frame); clearTimeout(done); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [still]);
  // The victory india sounds as the crest is pressed (the turn's own victory or defeat sting has already played).
  useEffect(() => {
    if (tone !== 'victory' || sang.current) return;
    const sing = () => { if (!sang.current) { sang.current = true; playSound('fanfare.legendary'); } };
    if (still) { sing(); return; }
    const t = setTimeout(sing, STAMP_MS);
    return () => clearTimeout(t);
  }, [tone, still]);
  const size = hero.length > 2 ? 'trio' : hero.length > 1 ? 'pair' : 'solo';
  const heroEra = Math.max(1, ...hero.map((m) => m.era ?? 1));
  const art = eraArt(heroEra - 1);
  const fallen = tone === 'fallen';
  const primary = actions.find((a) => a.primary);
  return (
    <div className={`gr gr-${tone}${still ? ' still' : ''}`} data-testid="game-reveal" data-tone={tone} data-settled={still ? 'true' : 'false'}
      onClick={(e) => { if (!still && !(e.target as HTMLElement).closest('button')) onSettle(); }}>
      <div className="gr-art" aria-hidden="true">
        {art && <img src={art.small} srcSet={`${art.small} 1280w, ${art.large} 1920w`} sizes="100vw" alt="" draggable={false} />}
      </div>
      <div className="gr-wash" aria-hidden="true" />
      {tone === 'victory' && <div className="gr-glow" aria-hidden="true" />}
      {tone === 'victory' && (
        <div className="gr-motes" aria-hidden="true">
          {MOTES.map((m, i) => <span key={i} style={{ left: `${m.x}%`, width: m.size, height: m.size, animationDelay: `${m.delay}s`, ['--rise' as string]: `${m.rise}px`, ['--sway' as string]: `${m.sway}px` }} />)}
        </div>
      )}
      <div className="gr-stage">
        <div className={`go-hero gr-hero ${size}`}>
          <div className="go-portraits">
            {hero.map((m, i) => (
              <div key={m.id} className="gr-frame" style={{ animationDelay: `${300 + i * 120}ms` }}>
                <PortraitFrame faction={m.faction} fallen={fallen} crest={false} />
                <span className="gr-crest" style={{ animationDelay: `${STAMP_MS + i * 120}ms` }}>
                  <span className="gr-rays" aria-hidden="true" />
                  <Crest faction={m.faction} size={size === 'solo' ? 74 : size === 'pair' ? 56 : 44} />
                </span>
              </div>
            ))}
          </div>
          <Ribbon tone={fallen ? 'red' : winner ? 'ink' : 'gilt'} className="gr-ribbon">{ribbon}</Ribbon>
        </div>
        <div className="go-main gr-main">
          <Wordmark height={16} color="#f3ead6" className="gr-wordmark" />
          <div className="go-kicker gr-kicker"><Star size={8} /> {kicker} <Star size={8} /></div>
          <h1 id={titleId} className="gr-title">{title}</h1>
          {fallen ? (
            <div className="gr-reason">
              <p>Your empire has fallen</p>
              <p>No cities or settlers remain.</p>
            </div>
          ) : (
            <p className="gr-reason">{reason && <Amounts text={reason} />}{finish && <span className="gr-finish"> {finish}</span>}</p>
          )}
          <StarDivider className="gr-rule" />
          <div className="go-table gr-table" role="list" aria-label="Final standings">
            {tracks.map((t, i) => (
              <div key={t.team} role="listitem" style={{ animationDelay: `${COUNT_START_MS - 260 + i * 70}ms` }}
                className={`go-row gr-row ${t.isMine ? 'mine' : ''} ${t === winner ? 'win' : ''} ${t.alive ? '' : 'out'}`}>
                <span className="go-who">
                  <span className="go-faces">
                    {t.members.slice(0, 2).map((m) => (
                      <PortraitRoundel key={m.id} faction={m.faction} size={28} color={playerColor(m.id)} fallen={!m.alive} />
                    ))}
                  </span>
                  {/* Same name as the victory race: the player's own row says You. */}
                  <span className="go-name">
                    {t.isMine ? (t.members.length > 1 ? `Your team (${t.label})` : 'You') : t.label}
                    {t === winner && <Star size={9} className="go-win-star" />}
                    {!t.alive && <small>Eliminated</small>}
                  </span>
                </span>
                <span className="go-meter" aria-hidden="true"><i style={{ width: `${Math.round(t.progress * count * 100)}%` }} /></span>
                {/* The settled number is the accessible one; the count-up is decoration. */}
                <b aria-label={`${t.vp} of ${t.target} VP`}><span aria-hidden="true">{Math.round(t.vp * count)}/{t.target}</span></b>
              </div>
            ))}
          </div>
          <div className="go-actions gr-actions">
            {/* The default (focused) button is the primary, and it is the safe one: Return never leaves the game. */}
            {actions.map((a) => (
              <button key={a.label} className={`btn${a.primary ? ' primary' : ''}`} data-go-primary={a === primary ? '' : undefined}
                disabled={a.disabled} onClick={a.onClick}>{a.label}</button>
            ))}
          </div>
          {!still && <div className="gr-skip" aria-hidden="true">Click to skip</div>}
        </div>
      </div>
    </div>
  );
}
