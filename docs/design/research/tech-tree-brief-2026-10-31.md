# Tech tree pass: brief (2026-10-31)

Two independent proposals are wanted, one from Astra (Codex) and one from Fable (Claude). Each works alone from this brief; the lead compares them for the designer.

## The designer's feedback (2026-10-31, in his words)

> I also think we should do a pass at rethinking the tech tree. Things that are great right now:
> 1. the resources are now meaningful. I find myself prioritizing getting the thing that discovers the resource first and moving settlers and taking cities based on that. that's a big improvement.
> 2. But some of the technologies feel like placeholders... e.g. the "great library" tech just lets you make the great library and nothing else. Some others are basically just there to advance the game forward but serve no purpose otherwise... (e.g. if you can't build mounted units, you have to just get it for the sake of getting it).
>
> So I'm not saying we need a full rethink. the "5 techs per era" dynamic is working and the eras evolve well. but some of the techs just feel... like they're not aligned with what i want. future tech era in particular. at a minimum we need to be able to upgrade the planes to something meaningful, but it feels like it just doesn't quite land near the end there... like i said, we're on the right track here, but not quite there yet.

## Keep

- Seven eras, about five techs per era, and the era progression.
- Resource reveals tied to techs (Iron, Niter, Coal, Oil and so on); he now plans expansion and conquest around them.
- Historical grounding: real peoples, real history, no sci-fi or fantasy. The Future era exists but must stay plausible near-future engineering (he rejected humanoid robots).

## Problems to solve

1. **Placeholder techs:** a tech whose only unlock is one building or one wonder (Philosophy only unlocks the Great Library today) or that a faction can't use at all (Aztecs and mounted techs). Every tech should matter to every faction, or at least offer each faction something.
2. **The late game doesn't land:** Modern and Future eras; aircraft need a meaningful upgrade path; the Future era should feel like the climax, not a list.
3. **Context that changes the tree right now:**
   - World wonders (decided 2026-10-31): one wonder per era, unique in the world, each unlocked by one tech of its era (Pyramids, Great Library, Hagia Sophia, Grand Bazaar, Royal Arsenal, Eiffel Tower, a grounded Future wonder). A wonder alone must not be a tech's whole payoff.
   - After the last tech, Science converts to Gold at 50% (decided 2026-10-31); research must never be a path to victory.
   - Era objectives are being redesigned as three cards per era, revealed on entering it.
   - No faction should be a pure turtle; conquest is part of every win.

## Constraints

- Read first: `CLAUDE.md` (pillars), `.claude/skills/legatum-rules/references/game-rules.md` (the tech table near line 129, unit lines, buildings, §9 "Systems that do NOT exist"), `.claude/skills/legatum-design/SKILL.md` (decided and rejected ideas), `.claude/skills/legatum-factions/SKILL.md` (faction drawbacks such as Aztec no-horses, Mongol no-walls).
- Prefer techs that unlock several small things (a unit, a building, a rule tweak, a resource reveal) over one big thing. Prefer existing systems; any new mechanic must be named as new and justified.
- Civ 6 familiarity is a strong default (pillar 2); say where you differ and why.
- No hidden math; the AI must be able to value every unlock.
- Every faction gets something from every era; a faction-locked unlock needs a companion unlock that faction can use.

## Deliverable

A markdown proposal (under about 2,000 words plus tables) with:
1. A diagnosis: every current tech rated keep / tweak / replace, one line each, with the reason.
2. The proposed tree: per era, the five techs with what each unlocks (units, buildings, rule effects, resource reveals, wonder), marking changes from today.
3. The Modern and Future eras in detail, including the aircraft line (what Fighters and Bombers upgrade into, what counters them) and what makes the Future era the climax.
4. What it costs to build (rough size: rules, AI, art, UI) and a skateboard first increment.
5. The three changes you'd make first if only three were allowed.

Write only your proposal file; change nothing else.
