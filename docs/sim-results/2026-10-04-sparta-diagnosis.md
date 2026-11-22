# Sparta 7a diagnosis — Standard, 8-player Hard

Source: `faecae92` game code (the starting `9c7da03e` differed only in docs),
200 seeds `5200001..5200200`, scripted Hard, six sim threads. The final baseline
uses observer-only telemetry; its 200 winners and Sparta capture counts are
identical to the first telemetry run. Raw reports are in
`~/LegatumRuns/sparta-diagnosis/2026-10-04-sparta-*.json`.
The original 400-game 7a result was 8/400 Sparta wins at both speeds. This
matched Standard subset has **3/200 (1.5%)**; small win counts are noisy.

## What happens

The table gives mean owned cities at resolved turns 50, 100 and 150; median
first empire war/capture turn among seats that reached each event; mean captures,
Gold spent on item purchases and upgrades, estimated Production cost of naturally
completed items, and mean resolved turns at war/peace while alive. T50/T100 include
200 seats per faction, T150 193 (seven games ended earlier). Production excludes
Gold purchases and unfinished work; end-of-turn quotes can overstate a Settler's
actual price after its training count rises. Gold excludes upkeep and treaty fees.

| Faction | Wins | Cities T50 / T100 / T150 | First war / capture | Captures | Gold | Production | War / peace turns |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: |
| Japan | 13 | 1.86 / 2.58 / 2.80 | 67 / 142 | 0.60 | 114 | 4,527 | 112 / 103 |
| Germany | 27 | 1.89 / 2.71 / 3.11 | 138 / 167 | 0.01 | 692 | 5,369 | 56 / 163 |
| Mongols | 61 | 1.82 / 2.92 / 3.93 | 99 / 136 | 1.20 | 569 | 4,907 | 102 / 116 |
| Sparta | 3 | 1.92 / 2.52 / 2.97 | 27 / 107 | 0.74 | 219 | 4,084 | 177 / 42 |
| Byzantium | 27 | 1.77 / 2.51 / 2.99 | 94 / 153 | 0.70 | 234 | 4,300 | 88 / 129 |
| Athens | 29 | 1.93 / 2.91 / 3.51 | 105 / 154 | 0.42 | 88 | 4,962 | 85 / 132 |
| Switzerland | 19 | 1.89 / 2.92 / 3.31 | 121 / 176 | 0.17 | 447 | 5,220 | 66 / 152 |
| Celts | 21 | 1.89 / 3.06 / 3.57 | 153 / none | 0 | 620 | 5,390 | 41 / 178 |

First-war observations reached all 200 Sparta seats; first-capture reached
90. Only 8 Sparta seats captured by T50, 43 by T100 and 60 by T150. The
faction goal completed in 73/200 seats. Conquest/kill events paid mean 298 Gold
per Sparta seat and capture healed 281 units across the cohort. These are
real identity uses, but they arrive late relative to the premium on civilian
growth. Its completed Production value is the lowest of the eight factions.

There were **109 desertions in 29/200 seats** (median resolved turn 186): 54
Soldiers, 32 Rangers, 21 Troopers and 2 Scouts. Every event followed the core
condition of no empire war after grace; these are unit losses, not inferred
missing orders. Sparta itself was eliminated only twice, with the last city
captured once by Mongols and once by Athens. Most losses are surviving
empires falling behind in the VP race, not outright military defeat.

## AI versus rules

The AI uses the redesigned mechanics: core starts/promotes military with faster
XP and the Iron Guard level (`rules/promotion.rs`); combat and capture events
grant Gold and healing (`rules/battle.rs`), and its campaign target values nearby
healing (`ai/military.rs`). Production scoring sees premium-adjusted quoted
costs and trains a larger field army (`ai/production.rs`). Its wounded units
seek home healing (`ai/military.rs::rest`); diplomacy seeks one sustainable war
and resists peace after grace (`ai/diplomacy.rs::sparta_restless_target,
wants_peace`). The 177 war turns, early first war, 0.74 captures/game and 298
iron-price Gold/game confirm the identity is active, though only 90/200 seats
capture and the median first capture is T107.

Two isolated AI changes were tried and removed. Letting home-city melee join a
selected campaign raised captures from 0.74 to 0.95/game and wins from 3/200
to 5/200. Requiring two soldiers to stage near the target before an optional
declaration moved the median first war to T39, lowered captures to 0.69/game,
and won 7/200. A third, narrower AI correction is retained: when a wounded
Sparta unit has no safe road home and cannot heal in the field, recovery no
longer spends its order holding on zero-healing ground before combat and
campaign planning can use it (`ai/tactics.rs::recover_wounded`). It raised
captures to 0.81/game, wins to 5/200, iron-price Gold to 327/seat, and
lowered desertions to 0.46/game. Goal completion stayed 73/200.
The user's 7a Hard playtest at T53 showed the relevant failure mode directly:
a one-city Sparta declared war, sent two of four units, and wounded
attackers held outside healing territory. The correction does not close the
win gap on these seeds. Its remaining cause is mixed: the AI still converts
few wars into captures, while the civilian premium constrains growth and
goal completion. These results do not establish how a skilled human would
fare. Desertion is secondary in this cohort.

## Rule options for Sam

These were isolated, opt-in 200-game Standard experiments on the unchanged-AI
baseline seeds. They were removed after measurement; **default rules remain
unchanged**. Paired
win changes count seeds gained/lost relative to the 3/200 baseline. The sample
is useful for triage, not final balance acceptance at either speed.

| Option | Sparta wins | Gained / lost paired seeds | Captures/game | Cities T150 | Goal completions |
| --- | ---: | ---: | ---: | ---: | ---: |
| Current rules, before recovery fix | 3/200 (1.5%) | — | 0.74 | 2.97 | 73/200 |
| Non-military premium +25% instead of +50% | 10/200 (5.0%) | 7 / 0 | 0.82 | 3.17 | 94/200 |
| Permit 2 HP/turn field healing instead of zero | 4/200 (2.0%) | 3 / 2 | 0.72 | 2.96 | 67/200 |
| Exempt all trained Settlers from the +50% premium | 4/200 (2.0%) | 4 / 3 | 0.62 | 3.16 | 77/200 |

The premium reduction is the only option with a consistent paired gain here,
but 5% remains below the other factions' baseline range and its story cost is
real: it softens the iron-price constraint on every civilian build. The Settler
exemption grows cities without improving wins on these seeds. Field healing
weakens capture-as-healing and did not help. None should be selected from this
Standard sample alone; any selected rule needs Quick and held-out validation.
