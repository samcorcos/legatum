// The copy lint's rules, in one data file so the list grows in one place (user, 2026-10-17: "'per complete
// pair' is not normal language... is there a rule or something we can add to make sure we use normal
// language"). The style guide these rules enforce is "Game text: plain language" in
// .claude/skills/legatum-ui/SKILL.md. The detector is `copyIssuesIn` in ./textArtifacts.ts; the tests are
// ui/textArtifacts.test.tsx (every string the game prints from data, rendered) and ui/copySource.test.ts
// (string literals in the app and in crates/core that reach the player).
//
// To add a rule: append a row. `pattern` runs on one rendered text block. `source: true` also runs it over
// string literals in the source files, so only mark phrases that never appear in code identifiers.

export interface CopyRule {
  pattern: RegExp;
  /** Why it is banned, in one line. */
  reason: string;
  /** What to write instead. */
  instead: string;
  /** Also scan string literals in app and core source (only for multi-word phrases code never contains). */
  source?: boolean;
}

/** Mechanics-speak and code terms: describe what the player gets, not the integer maths behind it. */
export const BANNED: CopyRule[] = [
  { pattern: /\bNearest you know\b/i, reason: 'reads oddly (plain-language review, 2026-11-22)', instead: '"Nearest known: Sarai (Mongols), 5 tiles from your city Ajmer."', source: true },
  { pattern: /\bPeace (?:ends|would end|ended) (?:all|every) (?:wars?|conflicts?)\b/i, reason: 'Peace ends only the war between its two named teams', instead: 'name that war and any wars that continue', source: true },
  { pattern: /\bPeace (?:covers|includes|binds) (?:their |your )?allies\b/i, reason: 'Alliances stay, but allies are not included in a Peace', instead: 'Peace is between the two named teams', source: true },
  { pattern: /\bWar Shrine\b/i, reason: 'Sparta’s building is called the Agoge', instead: 'Agoge', source: true },
  { pattern: /\bin this family\b/i, reason: 'building refusal does not name what stands', instead: 'name the standing building or upgrade', source: true },
  { pattern: /\bNutrient Synthesis\b/i, reason: 'removed technology', instead: 'describe the growth cost without it', source: true },
  { pattern: /\baxial\b/i, reason: 'internal map coordinate term', instead: 'describe the player-visible tile rule', source: true },
  { pattern: /takes effect a turn later/i, reason: 'a surprise war at Peace starts before attacks in the same turn', instead: 'can declare War and attack in the same turn', source: true },
  { pattern: /doubles the experience/i, reason: 'a kill doubles only the enemy-strength term of fight XP', instead: 'a kill earns extra experience', source: true },
  { pattern: /becomes a Cataphract/i, reason: 'the Keshig already occupies the Cataphract slot', instead: 'upgrades to a Knight', source: true },
  { pattern: /missing prerequisites first/i, reason: 'technologies have no prerequisites within an era', instead: 'choose any current-era technology', source: true },
  { pattern: /\bmake units desert\b|\bwithout battle\b|\brestless\b/i, reason: 'Sparta’s desertion rule was removed on 2026-11-07', instead: 'name the +50% non-military cost as Sparta’s only drawback', source: true },
  { pattern: /\bvalid JSON\b/i, reason: 'a parser detail in a load failure (UI review, 2026-11-10)', instead: 'This file is not a Legatum save. Nothing was changed; choose another file or start a new game.' },
  { pattern: /^Error: /, reason: 'a stringified exception shown as player text (UI review, 2026-11-10)', instead: 'run the failure through loadErrorMessage' },
  { pattern: /\bunknown owner\b|\boff the map\b/i, reason: 'engine validation detail in a load failure (UI review, 2026-11-10)', instead: 'This save file is damaged and can\'t be loaded. Your save file is kept; ...' },
  { pattern: /\bat the defen[cs]e ratio\b|\bscaled by the ratio\b/i, reason: 'the pooled city overflow rule was replaced by a per-hit carry in rules 27 (UI review, 2026-11-11)', instead: '"The hit that empties the city\'s HP carries on to the garrison with the part of its damage the city did not absorb. Later hits strike the garrison in full."', source: true },
  { pattern: /\bPlayerId\b/, reason: 'engine identifier', instead: 'seat number' },
  { pattern: /first capture of it/i, reason: 'the original-capital rule said it as a count of captures (UI review, 2026-11-05)', instead: '"Hold it to score 1 VP."', source: true },
  { pattern: /\bbinds? (allies|you)\b/i, reason: 'unclear diplomacy effect', instead: 'changes how wars work' },
  { pattern: /(?:^|\s)Live ·/, reason: 'internal VP label', instead: 'Held only while you lead' },
  { pattern: /\bsuperseded\b/i, reason: 'engine report term', instead: 'dropped' },
  { pattern: /\boutrun units\b|\bcan.t attack a mounted unit\b/i, reason: 'Outrider is an escape when the mounted unit moves', instead: "Outrider: before the World War era, a mounted unit can ride away from a melee attacker that didn't start the turn beside it.", source: true },
  { pattern: /\(at least 1\)/i, reason: 'a fee minimum belongs in plain words', instead: 'with a minimum of 1 Gold', source: true },
  { pattern: /\(at least \d+ in total\)/i, reason: 'a fee minimum in brackets (train-1103 review: "for each city it owns (at least 1 in total)")', instead: '"for every city it owns, and never less than 1 Gold"', source: true },
  { pattern: /\bcurrent count plus one new\b/i, reason: 'code shorthand for the Hold card rule (train-1103 review)', instead: '"reach the count, with at least one you added since you entered the era or last scored"', source: true },
  { pattern: /\bsince entry or last point\b/i, reason: 'a counter label read as a code comment (train-1103 review: "+0 since entry or last point")', instead: '"added since you entered or last scored"', source: true },
  { pattern: /\bkeep the condition or item\b/i, reason: 'a ledger header that never says what is lost (train-1103 review)', instead: '"you lose these if you lose the city, tile or condition"', source: true },
  { pattern: /\bmay gain the wonder.s effect\b/i, reason: 'hedged: the city does get the effect (train-1103 review)', instead: '"Its city gets +1 Production."', source: true },
  { pattern: /\bseven AI empires\b|\bseven rivals\b/i, reason: 'Standard now starts with six empires (UI review, 2026-11-03)', instead: 'derive the count from MAP_SIZES.Standard.default', source: true },
  { pattern: /\(theirs \w|\btheirs (?:rules|link)\b|\bTheirs is \S+, yours is\b|\byours (?:rules|link)\b/i, reason: 'a version mismatch written as a comparison table (UI review, 2026-11-21)', instead: '"This game is on version X and you have Y. Update the older one to play together."', source: true },
  { pattern: /Reconnect to the server/i, reason: 'players do not know there is a server (UI review, 2026-11-21)', instead: '"You can leave once you are connected again."', source: true },
  { pattern: /\bnone is legal\b/i, reason: 'mechanics-speak (UI review, 2026-11-03)', instead: 'say why no choice can be made', source: true },
  { pattern: /is no betrayal and has no/i, reason: 'wordy decline rule', instead: 'Declining costs nothing', source: true },
  { pattern: /\bPurple-Born\b|\bPorphyrogennetos\b/i, reason: 'the Heir has one player-facing name', instead: 'Heir to the Throne or the Heir', source: true },
  { pattern: /\bSleep \(F\)|\bSleep\b.{0,60}\b(?:works for military and civilian|keeps? a unit|is the primary on a unit|takes? a unit out of the order cycle)\b/i, reason: 'units use Fortify (military) or Wait (civilians) for this order', instead: 'Fortify or Wait' },
  { pattern: /\b(Unlocks? (an? )?(Embassy|Academy)|Build an? (Embassy|Academy))\b/i, reason: 'removed buildings must not appear as live unlocks or build choices', instead: 'describe the remaining technology effect and Great Library route', source: true },
  { pattern: /\bgame menu\b/i, reason: 'the menu is called Menu (UI review, 2026-11-22)', instead: '"Menu → Save to file"', source: true },
  { pattern: /takes its place once/i, reason: 'a replaced game is replaced, not "given a place" (plain-language review, 2026-11-22)', instead: '"Starting a new game replaces it."', source: true },
  { pattern: /\bcomplete pairs?\b/i, reason: 'integer maths read aloud', instead: '"+1 Gold for every 2 population"', source: true },
  { pattern: /\bGold held\b|\bheld adds\b/i, reason: '"held" for Gold is bookkeeping', instead: '"in your treasury"', source: true },
  { pattern: /\brounded (down|up)\b|\brounds (down|up)\b|\bround(ing)? down\b/i, reason: 'rounding is implementation detail', instead: 'state the number the player gets, or "about"', source: true },
  { pattern: /\b(floor|ceil|min|max|abs|trunc)\(/i, reason: 'a code expression', instead: 'the rule in words' },
  { pattern: /\baccru(e|es|ed|ing)\b/i, reason: 'accounting jargon', instead: '"earns", "builds up"', source: true },
  { pattern: /\bper-turn tick\b|\bticks? (over|down|up)\b/i, reason: 'engine-loop jargon', instead: '"each turn"', source: true },
  { pattern: /\bper turn per\b/i, reason: 'stacked rates', instead: 'one rate per sentence: "each city earns +1 Gold a turn for every 2 population"', source: true },
  { pattern: /\b(live|permanent) VP source\b|\bVP source\b/i, reason: 'the code\'s word for it', instead: '"a way to score", or say whether the VP can be lost' },
  { pattern: /\bLive:/, reason: 'a type tag, not a sentence', instead: '"You lose the VP while ..."', source: true },
  { pattern: /(?:^|[.!?]\s)Live\. [A-Z]/, reason: '"Live." is the code\'s tag for a VP that can be lost, read as a sentence (8.9.14 UI review, 2026-10-28: "Live. You lose the VP while...")', instead: 'start with what the player loses: "You lose the VP while ..."', source: true },
  { pattern: /\b1 (orders|alerts|issues)\b/i, reason: 'a count of one with a plural noun (8.9.14 UI review, 2026-10-28: "Turn details · 1 orders")', instead: 'the counted() helper in app/src/plural.ts: "1 order"', source: true },
  { pattern: /\b(?:you have|you hold|you own|spare)[:\s]+[−-]\d/i, reason: 'a negative amount of something the player has (8.9.16 UI review, 2026-10-29: "Iron supply: needs 1 spare, you have -2")', instead: '"you have none" for zero or less spare supply' },
  { pattern: /\bwithin \d+\s*(?:$|[.,;:)!?])/, reason: 'a distance with no unit (8.9.16 UI review, 2026-10-29: "or strikes within 5")', instead: '"strikes up to 5 tiles away", "within 5 tiles"' },
  { pattern: /\b(Science|Gold|Military|Balanced) will choose\b/, reason: 'the automation focus named as the one who chooses (UI review, 2026-10-31: "Science will choose when it empties")', instead: '"When it empties, the city picks a Science build."', source: true },
  { pattern: /\bbest available choice\b/i, reason: 'a stock phrase that hides why automation picked an item (UI review, 2026-10-31: "Horseman: Science focus, best available choice")', instead: 'the actual focus or civilian build value', source: true },
  { pattern: /\bSiege: No Siege\b/, reason: 'a reason line that repeats its own label (UI review, 2026-10-31)', instead: 'show the era gate where the price would be: "Mongols can buy Siege from the Medieval era."' },
  { pattern: /\bHold it or rebase it\b/, reason: 'names two buttons that may be missing or disabled (UI review, 2026-11-05: Rebase was unavailable on the same card)', instead: 'name only the buttons on the card: "Wait for a target, or skip this turn."', source: true },
  { pattern: /\bmultiplier\b/i, reason: 'maths jargon', instead: '"×2", "doubled", "half"', source: true },
  { pattern: /\bbase (strength|cost|price|melee|ranged|Production|Gold|Science|Food)\b/i, reason: '"base" is a tuning figure the player never sees', instead: 'the real number, or "before bonuses"', source: true },
  { pattern: /\badjusted (Production|price|cost)\b/i, reason: 'bookkeeping', instead: '"its Production cost"', source: true },
  { pattern: /\bexcepted\b/i, reason: 'legalese', instead: '"except Scouts", "not counting Scouts"', source: true },
  { pattern: /\bpop\b/i, reason: 'abbreviation', instead: '"population"' },
  { pattern: /\bat Standard\b|\bon Quick\b|\bStandard costs\b|\bat Standard speed\b/, reason: 'speed modes were removed (Standard is the only speed)', instead: 'the plain number', source: true },
  { pattern: /\bbuilding famil(y|ies)\b|\b[A-Z][a-z]+-family\b/, reason: 'a code grouping (user, 2026-10-17: "what is a new building family?")', instead: '"the Market and its upgrades"', source: true },
  { pattern: /\bmutually exclusive\b/i, reason: 'logic jargon', instead: '"A unit can move or fire in a turn, not both"', source: true },
  { pattern: /\bprovided (that|they|it)\b/i, reason: 'legalese', instead: '"if"', source: true },
  { pattern: /\b(retain|retains|retained)\b/i, reason: 'formal', instead: '"keep", "keeps"', source: true },
  { pattern: /\bi\.e\.|\be\.g\./i, reason: 'abbreviation', instead: '"that is", "for example", or a second sentence' },
  { pattern: /Σ/, reason: 'a maths symbol', instead: '"added up"' },
  { pattern: /\bmax(imum)? \d|\(max\b|\bmin(imum)? \d|\(min\b/i, reason: 'abbreviation or maths', instead: '"up to 5", "at least 3"' },
  { pattern: /\bresp\.|\bw\/o?\b/i, reason: 'abbreviation', instead: 'the words' },
  { pattern: /\bhalving\b|\bera.s siege\b/i, reason: 'the Walls and Engineer rule in mechanics-speak (UI review, 2026-10-18)', instead: '"Walls halve this attack", "cancels Walls but not a Castle"', source: true },
  { pattern: /\bWalls: [^.]{0,60}\bfires?\b/, reason: 'the city\'s shots are "City fire"; Walls only ever means the building (8.9.6 UI review, 2026-10-19)', instead: '"City fire: Athens fires first, about -40 HP"', source: true },
  { pattern: /\bmedian score\b/i, reason: 'a statistics word on a player card (8.9.6 UI review, 2026-10-19: "Hard AI: median score N")', instead: '"a typical score of N", "most tries took N turns"', source: true },
  { pattern: /\bstockpil(e|es|ed|ing)\b|\bstored (Iron|Niter|Coal|Oil|Silicon)\b|\b(Iron|Niter|Coal|Oil|Silicon) (per|a|each) turn\b/i, reason: 'Strategic resources are supplied, not stored: there is no stockpile and no income (strategic supply, 2026-10-19)', instead: '"supplies 2 Iron", "uses 1 Iron", "Iron 5 / 6"', source: true },
  { pattern: /\b(\d+|newest|idle|supply|every) users?\b|\busers? (idle|work)/i, reason: '"users" is the code\'s word for the buildings and units that need a strategic resource (strategic supply UI review, 2026-10-20)', instead: '"buildings or units": "2 buildings or units are idle"', source: true },
  { pattern: /\bin use of\b/i, reason: 'demand is not "in use": what is wanted can exceed what is supplied (strategic supply UI review, 2026-10-20: "4 of 2 Iron in use")', instead: '"2 of 2 Iron at work", "needs 4, supplies 2"', source: true },
  { pattern: /\bsupply (\d+|\$\{[^}]*\}) and (\d+|\$\{[^}]*\}) are\b/i, reason: 'the old refusal wording, a sum read aloud (strategic supply UI review, 2026-10-20)', instead: 'the core\'s sentence: "all 2 of your Iron are in use"', source: true },
  { pattern: /\balready counts\b/i, reason: 'says nothing about what a Mine adds (strategic supply UI review, 2026-10-20: "Iron supply already counts" hid the +1 supply)', instead: '"+2 Production and +1 Iron supply when built"', source: true },
  { pattern: /\bera rule\b/i, reason: 'internal wording for a resource requirement', instead: 'say which units or buildings needed the resource', source: true },
  { pattern: /\bSleep stops\b/i, reason: 'Sleep is a retired player-facing order name', instead: '"Wait stops it asking each turn."', source: true },
  { pattern: /\bCounted above\b/i, reason: 'does not say which forecast number includes this damage', instead: 'name the unit HP loss above', source: true },
  { pattern: /\bMine on one for \+\d+ (Niter|Oil|Silicon)\b/, reason: 'Niter, Oil and Silicon take their own works, not a Mine (era resources UI review, 2026-10-22)', instead: '"build an Oil Well on one for +1 Oil"', source: true },
  { pattern: /(Niter|Oil|Silicon)[^.]{0,60}\bwith a Mine\b/, reason: 'Niter, Oil and Silicon take their own works, not a Mine (era resources UI review, 2026-10-22)', instead: '"3 Oil with an Oil Well"', source: true },
  { pattern: /\ban (Coal|Stone|Copper|Niter|Silicon)\b/, reason: 'wrong article', instead: '"a Coal deposit"', source: true },
  { pattern: /\ban (?:Uni(?:versity|versal|on|form|que|ts?\b|fied|ty)|Eu)/, reason: 'wrong article: these vowel letters sound like "you" (2026-10-31 UI review: "an University")', instead: '"a University", "a European"', source: true },
  { pattern: /\ba (?!Uni|Eu|One\b)[AEIOU][a-z]/, reason: 'wrong article before a generated unit or building name (8.9.7 UI review, 2026-10-21: "a Infantry")', instead: 'an a/an helper: "an Infantry"' },
  { pattern: /\bthe user[’']s\b|\b\d+\.\d+\.\d+\b/i, reason: 'developer voice or a build number in scenario text (8.9.7 UI review, 2026-10-21: "the user\'s 8.9.6 Switzerland game")', instead: '"In a real game, the AI India led most of the way"' },
  { pattern: /\byour [A-Z][a-z]+ game\b|\btests whether\b|\bthe designer\b|\bshould be easy for a human\b/, reason: 'scenario text that speaks to the developer or one owner, not the player (pre-launch UX review, 2026-11-21: "Turn 133 of your Aztecs game", "it tests whether Hard sees and uses the chance")', instead: '"Turn 133 of a recorded game."; say what the situation is and what the player must do' },
  { pattern: /\bscripted (opponent|rusher)\b|\bWhat Claude\b|\bon the host\b/i, reason: 'developer voice or storage detail in player text (Scenario Lab review, 2026-10-18)', instead: '"follows a fixed rush plan", "What we expected", "Your run is saved"' },
  { pattern: /\bper (?:Granary|Workshop|Library|Market) family\b/, reason: 'the code\'s grouping on a technology card (8.9.23 UI review: "+1 Food per Granary family")', instead: '"+1 Food for each Granary and Mill"' },
  { pattern: /\bring-2\b/, reason: 'a distance in rings (8.9.23 UI review: "claims ring-2 tile")', instead: '"its best tile 2 tiles out"' },
  { pattern: /\+1 VP \(permanent\)/, reason: '"permanent" in brackets for a point that is kept (8.9.23 UI review)', instead: '"+1 VP, kept for good"', source: true },
  { pattern: /\bFactory\/Ruhr\b/, reason: 'both factions\' building names in one label (8.9.23 UI review)', instead: 'the viewer\'s own name: "Factories +2 Production", "Ruhr Foundries +2 Production"', source: true },
  { pattern: /\bone movement round behind\b/, reason: 'engine rounds read aloud (8.9.23 UI review)', instead: '"Moves through this tile right behind your Spearman."', source: true },
  { pattern: /\bcosts? [-]\d/, reason: 'a hyphen standing in for a minus sign (8.9.23 UI review: "Upgrade Gold -50%")', instead: 'a true minus: "costs −50%"' },
  { pattern: /(?:^|\s)-\d/, reason: 'a hyphen standing in for a minus sign before a number (2026-11-06 UI review: the Guide printed "-2 Food")', instead: 'a true minus: "−2 Food"' },
  { pattern: /\bGold is too short for\b/, reason: 'a stock phrase for an automation refusal (8.9.23 UI review)', instead: '"Not enough Gold to pay another unit\'s upkeep, so it builds the next best thing."', source: true },
  { pattern: /\bis ready: [a-z]/, reason: 'a ready line that repeats its own title (8.9.23 UI review: "The New Fire is ready: kindle it for +2")', instead: 'the title is the action, the line is the score: "Kindle the New Fire" over "+2 VP, kept for good"', source: true },
  { pattern: /\bguard radius\b/i, reason: 'a code term for how far a Fighter guards (8.9.23 UI review: "range 7, guard radius 5")', instead: '"Strikes up to 7 tiles away; guards 5 tiles around its city"', source: true },
  { pattern: /\bgross Gold\b/i, reason: 'an accounting word (8.9.23 UI review: "half their gross Gold")', instead: '"half the Gold they make"', source: true },
  { pattern: /\bresting defense\b/i, reason: 'American spelling of a rule name; the game uses British spelling (8.9.23 UI review)', instead: '"resting defence"', source: true },
  { pattern: /\bstays banked in [A-Z]/, reason: 'bookkeeping word order for what a city keeps (8.9.23 UI review: "Granary progress 12 stays banked in Astra")', instead: '"Astra keeps the 12 Production already put into the Granary."', source: true },
  { pattern: /\bwindows from your connection\b/i, reason: 'awkward, and the others may be other people on the same network (relay messages review)', instead: '"people on your internet connection"', source: true },
  { pattern: /\bsent too quickly\b|\btoo much (?:was )?sent\b/i, reason: 'passive network talk with no subject (relay messages review)', instead: '"You are sending too much, too fast."', source: true },
  { pattern: /“Game” with a number/, reason: 'the default name is "Game" and four characters, not a number (relay messages review)', instead: '"The game list will show “Game” and four characters."', source: true },
];

/**
 * The game's canonical terms. Each mechanic has one name: `canonical` is the word the game teaches, and every
 * `synonyms` pattern is another name for the same thing that must not appear in player text.
 */
export const GLOSSARY: { canonical: string; meaning: string; synonyms: RegExp[] }[] = [
  { canonical: 'tile', meaning: 'one map cell; distances are in tiles', synonyms: [/\bhex(es)?\b/i] },
  { canonical: 'population', meaning: 'a city\'s size: each growth claims a tile', synonyms: [/\bpop\b/i, /\bcitizens\b/i] },
  { canonical: 'Production', meaning: 'the yield that builds things', synonyms: [/\bhammers?\b/i] },
  { canonical: 'Gold, Food, Science', meaning: 'yields, always capitalised', synonyms: [/(?<![-\w])(gold|food|science)(?![-\w])/] },
  { canonical: 'city HP', meaning: 'the city\'s own health bar (200 at full)', synonyms: [/\bcity (health|hit points)\b/i, /\bbreak the walls\b/i] },
  { canonical: 'city strength', meaning: 'the city\'s combat strength (Walls add to it)', synonyms: [/\bcity defen[cs]e strength\b/i, /\bcity power\b/i] },
  { canonical: 'Walls', meaning: 'the building only; "walled city" is fine', synonyms: [/\bthe walls\b/] },
  { canonical: 'military unit', meaning: 'any unit that fights (Scouts count unless the rule says otherwise)', synonyms: [/\barmed units?\b/i, /\bregiments?\b/i, /\btroops\b/i] },
  { canonical: 'level', meaning: 'a unit\'s promotion level', synonyms: [/\bpromotion (tier|rank)\b/i] },
  { canonical: 'technology', meaning: 'in rules text; "tech diffusion" is the one fixed phrase', synonyms: [/\btechs\b/i] },
  { canonical: 'VP', meaning: 'victory points; the Guide spells it out once as "victory points (VP)"', synonyms: [/\bvictory points?\b(?! \(VP\))/i] },
  { canonical: 'era objectives', meaning: 'the three ways to score an era\'s 1 VP', synonyms: [/\bera (goals|targets|tasks)\b/i] },
  { canonical: 'decadence', meaning: 'the cut to a leader\'s yields', synonyms: [/\b(lead|leader) penalty\b/i, /\bcomplacency penalty\b/i] },
  { canonical: 'at Peace, at War, Alliance', meaning: 'treaty states, capitalised', synonyms: [/\bat peace\b/, /\bat war\b/] },
  { canonical: 'British spelling', meaning: 'defence, labour, centre, armour, honour, colour, favour', synonyms: [/\b(defense|labor|centers?|armor|honor|color|favor)\b/i] },
  { canonical: 'supply', meaning: 'the Iron, Coal or other strategic resource your deposits give; each building or unit that needs it uses 1 while it exists', synonyms: [/\bstockpil(e|es|ed|ing)\b/i, /\bstored (Iron|Niter|Coal|Oil|Silicon)\b/i, /\b(Iron|Niter|Coal|Oil|Silicon) (per|a|each) turn\b/i] },
  { canonical: 'interest', meaning: 'Switzerland\'s Gold from its treasury', synonyms: [/\btreasury (bonus|yield)\b/i] },
  { canonical: 'Share the Learning', meaning: 'Athens\' goal and its broadcast of free technologies; the code\'s old name for it was the Signal (8.9.14 UI review, 2026-10-28: "shared the Signal" survived the rename)', synonyms: [/\b(?:share|shares|shared|sharing) the Signal\b/i, /\bthe Signal is (?:not )?shared\b/i, /\bSignal broadcast\b/i] },
  { canonical: 'Company', meaning: 'a hired Mercenary Company', synonyms: [/\bhired (units?|troops|armies)\b/i, /\bmercenary (units?|troops)\b/i] },
];

/** A sentence longer than this many words is two sentences. */
export const MAX_SENTENCE_WORDS = 30;

/** A sentence with this many semicolons stacks too many ideas. */
export const MAX_SEMICOLONS = 2;

/**
 * Text that breaks a rule on purpose, with the reason. `rule` is the flagged phrase as matched (or "long",
 * "semicolons", "doubled"), `text` a substring of the block it may appear in. Keep this short: a real exception
 * is rare, and lore is the usual one.
 */
export const EXCEPTIONS: { text: string; rule: string; why: string }[] = [
  { text: 'Chivalry unlocks Knights, Castles and Sparta’s War Shrine', rule: 'War Shrine', why: 'the parallel text-guide-rules task owns this Guide research paragraph' },
  { text: 'standing regiments', rule: 'regiments', why: 'lore: the tagmata were regiments' },
  { text: 'trip-hammer', rule: 'hammer', why: 'lore: a real machine' },
  { text: 'Spartan citizens', rule: 'citizens', why: 'lore: the Spartiates were citizens; this is history, not a city\'s population' },
];
