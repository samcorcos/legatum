// The UI-wide text guards over every player-facing string a screen can print from data (the corpus in
// test/playerText.tsx), and the markup of the screens that print the most prose.
//  - The artifact guard (user, 2026-10-11: ",." after a piece card's cost, "a few places"): the marks of a text
//    join gone wrong. Build UI text from structured data; never strip phrases out of prose with regexes.
//  - The copy lint (user, 2026-10-17: "'per complete pair' is not normal language"): banned mechanics-speak,
//    glossary synonyms, doubled words and long sentences, from test/copyRules.ts. The style guide is "Game
//    text: plain language" in the legatum-ui skill.
// A failure lists each hit as "source | reason | text".
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { FACTION_GOAL, SWARM_GOAL } from '../data';
import { artifactsIn, auditBlocks, auditCopy, copyIssuesIn, htmlBlocks } from '../test/textArtifacts';
import { companyText, coreInterfaceText, diplomacyText, factionText, guideText, interfaceText, itemText, renderedSheetText, reportText, techText, victoryText, type TextGroup } from '../test/playerText';
import { FactionSheetView } from './FactionPicker';
import { SOON_SLOTS, heldBackCatalog, scenarioCatalog } from '../scenarios/scenarios';

const report = (hits: string[]) => expect(hits, hits.join('\n')).toEqual([]);

it('the detector flags every artifact the user described and passes clean prose', () => {
  for (const bad of ['152 Production,.', 'a , b', 'x ;.', 'x :.', 'wait..', 'a ,b', 'a .b', 'a,, b', 'Cost ( )', 'Cost ()', ', and more', 'one  two', 'undefined Gold', 'NaN turns', 'null', 'A · · B', 'Walls,', 'Walls ·', 'Walls -']) {
    expect(artifactsIn(bad), bad).not.toEqual([]);
  }
  for (const good of ['Requires Masonry. The city fires like any walled city.', 'Wait… what', 'Wait...', '+3 Production, −1 Gold', '152 Production', 'Sees 3 hexes.', 'Cost (Gold)', 'Peace · 7 turns', 'Level 1.5', '—', 'Buy—', '· 7 of 10 VP', 'Hit −18 HP (82 left)+']) {
    expect(artifactsIn(good), good).toEqual([]);
  }
  // The exact markup of the bug: a card's cost amount followed by what a regex left behind.
  const amount = '<span class="amt"><span class="amt-n">152</span><span class="amt-icon"></span><span class="sr-only"> Production</span></span>';
  expect(auditBlocks('bug', htmlBlocks(`<p class="pc-text">${amount},.</p>`))).not.toEqual([]);
  expect(auditBlocks('clean', htmlBlocks(`<p class="pc-text">Requires Masonry. ${amount}.</p><div>Gold</div><div>Food</div>`))).toEqual([]);
});

const audit = (groups: TextGroup[]) => groups.flatMap((g) => [...auditBlocks(g.label, g.blocks), ...auditCopy(g.label, g.blocks)]);

it('the copy lint flags mechanics-speak, glossary synonyms, doubled words and long sentences, and passes plain prose', () => {
  for (const bad of [
    'Each city earns +1 Gold per complete pair of population.', 'Every 100 Gold held adds +1 Gold.', '50% of Production, rounded down.',
    'Military units (Scouts excepted) act.', 'Live: the VP is lost while the treasury is below it.', 'Each level adds 10% base strength.',
    'Needs population 2; costs 1 pop.', 'Within 5 hexes of the capital.', 'Spartan regiments march.', 'The the city grows.',
    'Build a War Shrine.', 'Already built in this family.', 'Nutrient Synthesis lowers growth cost.', 'The lowest axial coordinate wins.',
    'You gain gold from ruins.', 'While at war, pay an enemy.', 'Moving and firing remain mutually exclusive.', 'Your Iron stockpile holds 4.', 'Coal: 60 stored, +2 Coal per turn.',
    'Short of Iron: 2 users idle.', 'Iron: 4 in use of 2 supplied.', 'Your deposits supply 2 and 4 are in use.', 'Iron supply already counts.', 'Bring an Coal deposit inside your borders.',
    'Iron supply: needs 1 spare, you have -2', 'The Ruhr Foundry gives +6 Production and -2 Food.', 'Costs -2 Food.', 'Needs 1 Iron: you have −1.', 'Guards 4 tiles against air strikes, or strikes within 5', 'Strikes within 5.',
    'One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty twenty-one twenty-two twenty-three twenty-four twenty-five twenty-six twenty-seven twenty-eight twenty-nine thirty thirty-one.',
  ]) expect(copyIssuesIn(bad), bad).not.toEqual([]);
  for (const good of [
    'Each city earns +1 Gold a turn for every 2 population.', 'Every 100 Gold in your treasury earns +1 Gold a turn in interest.',
    'You lose the VP while your treasury is below 5,000 Gold.',
    'Within 5 tiles of your capital.', 'While at War, build a Company.', 'A unit can move or fire in a turn, not both.', 'Each deposit supplies 2 Iron, 3 with a Mine.', 'Short of Iron: 2 buildings or units are idle.', 'Bring a Coal deposit inside your borders.', '+2 Production and +1 Iron supply when built.', 'Level 1.5 · 7 of 10 VP', 'Industry: −2 Food (Workshop, Ruhr Foundry).', 'Rounds 4-5 count twice.',
    'Iron supply: needs 1 spare, you have none', 'Needs 60 Gold (you have 59)', 'Guards 4 tiles against air strikes, or strikes up to 5 tiles away', 'Upkeep: free · 2 of 2 slots left', 'Founds a city. Costs 1 population.',
  ]) expect(copyIssuesIn(good), good).toEqual([]);
});

it('every unit, building, improvement and project reads cleanly in every faction (names, descriptions, card lines, unlock sentences, refusals)', () => {
  report(audit(itemText()));
});

it('every technology blurb, quote and per-faction effect reads cleanly', () => {
  report(audit(techText()));
});

it('every faction sheet, goal, rule and unique piece reads cleanly', () => {
  for (const [name, goal] of [...Object.entries(FACTION_GOAL), ['Swarm', SWARM_GOAL]] as const) {
    // Callers add the closing full stop to a rule, so a rule that already ends in one prints "..".
    expect(goal.rule(), `${name} rule ends without a full stop`).not.toMatch(/[.!?]$/);
  }
  report(audit(factionText()));
});

it('every faction sheet as rendered (all eight peoples: piece card, goals, hall) reads cleanly', () => {
  report(audit(renderedSheetText()));
});

it('the piece card never prints leftover punctuation or repeats its stat strip', () => {
  const html = renderToStaticMarkup(<FactionSheetView faction="Celts" sheetId="s" />);
  // A pure-stats unit's only paragraph is the technology it needs: the stat strip is not said again in words.
  expect(html.match(/pc-text/g)).toHaveLength(1);
  expect(html).toContain('Requires Mathematics.');
  const glass = renderToStaticMarkup(<FactionSheetView faction="Mongols" sheetId="s" />);
  expect(glass.match(/pc-text/g)).toHaveLength(1);
  expect(htmlBlocks(glass.slice(glass.indexOf('pc-vs'))).join(' ')).toContain('Shoots or charges each turn');
});

it('the Guide, section by section, reads cleanly', () => {
  report(audit(guideText()));
});

it('city, Levy, era, tile, unit, and post-game interface strings read cleanly', () => {
  report(audit(interfaceText()));
});

it('core city breakdowns and building refusals read cleanly', async () => {
  const groups = await coreInterfaceText();
  expect(groups[0].blocks.join(' ')).toContain('Granary');
  expect(groups[1].blocks.join(' ')).toContain('Corruption:');
  report(audit(groups));
});

it('decadence, eras and Great Works and combat rules read cleanly', () => {
  report(audit(victoryText()));
});

it('diplomacy and treaty texts read cleanly', () => {
  report(audit(diplomacyText()));
});

it('mercenary company texts read cleanly (rule, contract line, roster, order, report lines)', async () => {
  report(audit(await companyText()));
});

it('a core game\'s turn report reads cleanly (every line player 0 reads over 60 AI turns)', async () => {
  report(audit(await reportText(60)));
}, 60_000);

it('every Scenario Lab card, briefing and objective reads cleanly (including scenarios held back and the soon slots)', () => {
  const blocks: string[] = [];
  for (const e of [...scenarioCatalog(), ...heldBackCatalog()]) {
    const f = e.file;
    blocks.push(f.title, f.briefing, f.objective.summary, f.lab!.question, f.lab!.blurb, f.lab!.ask ?? '', f.lab!.opponent ?? '', f.tests ?? '');
  }
  for (const s of SOON_SLOTS) blocks.push(s.title, s.question, s.blurb, `Waiting for ${s.waitingFor}.`);
  const text = blocks.filter(Boolean);
  report([...auditBlocks('scenario lab', text), ...auditCopy('scenario lab', text)]);
});

it('Germany\'s text names the Ruhr Foundry, never the Forge, except to compare the two', () => {
  const german = (g: TextGroup) => /\(Germany\) | for Germany$|^Germany (info|sheet)$|^sheet Germany$/.test(g.label);
  const comparison = /(?:\b[Aa] Forge gives \+3(?: Production)?|replaces (?:the )?Forge|Forge: [+\d]+|compared with the Forge|Against the Forge)\b/g;
  const leaks = [...itemText(), ...techText(), ...factionText(), ...renderedSheetText()].filter(german)
    .flatMap((g) => g.blocks.filter((b) => /\bForges?\b/.test(b.replace(comparison, ''))).map((b) => `${g.label} | ${b}`));
  report(leaks);
});
