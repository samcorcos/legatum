import { useEffect, useRef } from 'react';
import { FACTION_INFO } from '../data';
import { relationWith } from '../diplomacy';
import { empireNameOf, type EmpireLocation } from '../empires';
import { FACTION_SHEETS } from '../factionSheets';
import { feel, useCombatFx } from '../feel';
import { useGame } from '../store';
import { Crest } from '../brand/ornaments';
import { MedallionRow, PortraitFrame } from '../ui/premium';
import { TreatyEmblem } from '../ui/ringMedallion';
import { useReducedMotion } from '../ui/useReducedMotion';
import { eraArt } from './eraArt';
import { GREETING } from './greetings';
import type { Moment } from './moments';
import './eraTakeover.css';
import './meeting.css';

const DIRECTION: Record<EmpireLocation['direction'], string> = {
  N: 'north', NE: 'north-east', E: 'east', SE: 'south-east', S: 'south', SW: 'south-west', W: 'west', NW: 'north-west',
};

/** Where they are, in one line: "Their city Roma lies 9 tiles north-east of Athens." */
export function whereLine(where: EmpireLocation | null): string | null {
  if (!(!where)) {} else { return null; }
  if (!(where.from === null)) {} else { return !(where.kind === 'city') ? `You have seen their ${where.name}.` : `You have seen their city ${where.name}.`; }
  const off = !(where.tiles === 0) ? `${where.tiles} ${!(where.tiles === 1) ? 'tiles' : 'tile'} ${DIRECTION[where.direction]} of ${where.from}` : `in ${where.from}`;
  return !(where.kind === 'city') ? `Their ${where.name} was seen ${off}.` : `Their city ${where.name} lies ${off}.`;
}

/**
 * The faction sheet speaks to the people's own player ("Your units get +1 move"); on a rival's plate the same line
 * speaks about them ("Their units get +1 move"), and advice meant for their own player (": switch it to Grow ...")
 * is cut. One wording per rule: the sheet's, turned to the third person.
 */
export function asTheirs(text: string): string {
  return text
    .replace(/: (?:switch|build|use|keep|move|choose|train)\b[^.]*\./gi, '.')
    .replace(/\b(?!(?:if|when|until|unless|that|and|but|so|or)\b)(\w+) you\b/g, '$1 them')
    .replace(/\bYour\b/g, 'Their').replace(/\byour\b/g, 'their')
    .replace(/\bYou\b/g, 'They').replace(/\byou\b/g, 'they');
}

/**
 * First contact (Sam's co-op partner, 2026-11-25: "like in Civ, when you first meet a civ, there's an unwrapping"): a
 * full-screen plate in the era takeover's language. A sealed letter breaks open on the current era's painting, their
 * people's portrait and greeting in their own words, what they do well and what it costs them (the faction sheet's own
 * lines), where they are and where you stand. Their people's music plays while it is up (`store.meetingPeople`).
 *
 * `momentsForTurn` makes one held `Moment` with `meeting` set for each empire the viewer's team met this turn (the
 * viewer's `diplomacy.met` grew). `MomentLane` shows it like the era-up: it waits for Return, Esc or its own
 * buttons, never times out and never blocks a teammate. Under reduced motion the letter is already open.
 */
export function MeetingTakeover({ moment, dismiss }: { moment: Moment; dismiss(): void }) {
  const card = moment.meeting!;
  const view = useGame((s) => s.views?.[s.seat] ?? null);
  const motionOff = useReducedMotion();
  const animationsOn = useCombatFx((s) => s.enabled);
  const reduced = !(!(motionOff) && !(!animationsOn));
  const primary = useRef<HTMLButtonElement>(null);
  const lead = card.members[0];
  useEffect(() => { primary.current?.focus({ preventScroll: true }); }, [moment.id]);
  // Their people's music while the plate is up; your own returns when it closes.
  useEffect(() => {
    useGame.setState({ meetingPeople: lead.faction });
    return () => useGame.setState({ meetingPeople: null });
  }, [lead.faction]);
  if (!(!view)) {} else { return null; }
  const info = FACTION_INFO[lead.faction];
  const sheet = FACTION_SHEETS[lead.faction];
  const greeting = GREETING[lead.faction];
  const name = empireNameOf(card.members.map((m) => view.players.find((p) => p.id === m.id)!).filter(Boolean), view.players);
  const me = view.players.find((p) => p.id === view.me);
  const art = eraArt((me?.era ?? 1) - 1);
  // A team of two describes both peoples (UI review of PR 184): each column gives each people its own labelled rows,
  // fewer of them, so the plate still fits at 1000x600.
  const peoples = [...new Set(card.members.map((m) => m.faction))];
  const two = peoples.length > 1;
  const about = peoples.map((f) => {
    const sh = FACTION_SHEETS[f];
    // A row written for the piece's own card stays out only when it leans on that card's header ("Iron Working unlocks it").
    const strengths = sh.pairs.filter((p) => p.strength && p.strengthIcon && !(p.onCard && /\bunlocks it\b/.test(p.strength))).slice(0, !(two) ? 3 : 2);
    const prices = sh.pairs.filter((p) => p.price && p.priceIcon).slice(0, !(two) ? 2 : 1);
    return { faction: f, name: FACTION_INFO[f].name, strengths, prices };
  });
  const relation = relationWith(view, card.team)?.state ?? 'Peace';
  const them = view.players.filter((p) => p.team === card.team);
  const cities = them.reduce((n, p) => n + p.cities, 0);
  const vp = them.reduce((n, p) => n + p.vp, 0);
  const where = whereLine(card.where);
  const showOnMap = () => { feel.select(); dismiss(); if (!(card.where)) {} else { useGame.getState().jumpTo(card.where.at); } };
  const diplomacy = () => { feel.select(); dismiss(); useGame.getState().set({ diplomacyOpen: true, diplomacyTeam: card.team, reportOpen: false, techOpen: false }); };
  return <div key={moment.id} className={`era-takeover meeting ${!(reduced) ? '' : 'reduced'}`} role="dialog" aria-modal="true" aria-label={`You meet ${name}`}
    data-testid="meeting-takeover" data-people={lead.faction} data-tier={moment.tier}>
    <div className="et-art" aria-hidden="true">
      {art && <img src={art.small} srcSet={`${art.small} 1280w, ${art.large} 1920w`} sizes="100vw" alt="" draggable={false} />}
    </div>
    <div className="et-wash" aria-hidden="true" />
    {!reduced && <div className="mt-seal" aria-hidden="true">
      {(['l', 'r'] as const).map((half) => <span key={half} className={`mt-half ${half}`}><Crest faction={lead.faction} size={132} /></span>)}
    </div>}
    <div className="mt-letter">
      <header className="et-head mt-head">
        <div className="mt-portraits">
          {card.members.map((m) => <PortraitFrame key={m.id} faction={m.faction} className="mt-portrait" crestSize={44} />)}
        </div>
        <div className="mt-title">
          <div className="et-kicker">First contact</div>
          <h2 className="et-title">{name}</h2>
          <p className="mt-identity">“{info.motto}”</p>
          <blockquote className="mt-greeting" data-testid="meeting-greeting">
            {greeting.native && <p className="mt-native">{greeting.native}{greeting.meaning && <span className="mt-meaning"> {greeting.meaning}</span>}</p>}
            <p className="mt-line">{greeting.line}</p>
          </blockquote>
        </div>
      </header>
      <section className="et-changes mt-cols" aria-label={`About ${name}`}>
        <div className="et-col" data-testid="meeting-strengths">
          <h3>What they do well</h3>
          {about.map((a) => <div key={a.faction} className="mt-people" data-people={a.faction}>
            {two && <h4 className="mt-people-name">{a.name}</h4>}
            {a.strengths.map((p, i) => <MedallionRow key={i} icon={p.strengthIcon!} tone="good" sign="Strength">{asTheirs(p.strength!)}</MedallionRow>)}
          </div>)}
        </div>
        <div className="et-col" data-testid="meeting-prices">
          <h3>Where they are weak</h3>
          {about.map((a) => <div key={a.faction} className="mt-people" data-people={a.faction}>
            {two && <h4 className="mt-people-name">{a.name}</h4>}
            {a.prices.map((p, i) => <MedallionRow key={i} icon={p.priceIcon!} tone="bad" sign="Weakness">{asTheirs(p.price!)}</MedallionRow>)}
          </div>)}
          {!two && sheet.counteredBy && <p className="mt-counter"><b>To face them:</b> {sheet.counteredBy}</p>}
        </div>
        <div className="et-col mt-between" data-testid="meeting-between">
          <h3>Between you</h3>
          <p className={`mt-state ${relation.toLowerCase()}`}><TreatyEmblem kind={relation} size={20} />{!(relation === 'War') ? !(relation === 'Alliance') ? <><b>At Peace.</b> You can offer them an Alliance.</> : <b>In an Alliance.</b> : <b>At War.</b>}</p>
          {where && <p>{where}</p>}
          <p className="mt-facts"><span>{cities} {!(cities === 1) ? 'cities' : 'city'}</span><span>{vp} VP</span></p>
        </div>
      </section>
    </div>
    <footer className="et-foot mt-foot">
        <div className="mt-links">
          {card.where && <button className="btn quiet" onClick={showOnMap}>Show on map</button>}
          <button className="btn quiet" onClick={diplomacy}>Diplomacy</button>
        </div>
        <div className="et-buttons">
          <button ref={primary} className="btn primary" data-primary-action onClick={() => { feel.select(); dismiss(); }}>Continue <span className="kbd">Return</span></button>
        </div>
    </footer>
  </div>;
}
