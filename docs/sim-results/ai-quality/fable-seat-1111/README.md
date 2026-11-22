# Fable seat: a model plays one seat as text (2026-11-11)

**Question.** Can a Fable-class model, seeing only what a seat sees, play clearly better than the scripted Hard AI?

**Answer.** In a short tactical fight, yes. Over a whole game, no: I led on VP until T116 and then lost the empire.

| Test | Me | Hard | Verdict |
| --- | --- | --- | --- |
| Card 2, After the Riflemen (take a city by T155) | 3 wins of 3; scores 1440, 1350, 1490 | Same three salts: 760 lost, 770 lost, 1320 won. 50-run baseline: 42% wins, median 801, p90 1360 | Better |
| Card 1, Stop the Snowball (keep 5 of 8 cities) | 0 wins of 3; scores 130, 150, 160 | Same salts: 0, 110, 0. Baseline: 8% wins, median 110, p90 410 | No real difference; the card is lost before it starts |
| Whole game, Athens, 7 Hard seats, seed 20261111 | One game, stopped at T137: 1 city, 4 VP, cannot win | Hard in the same seat, 16 dice salts: 3 wins, 2 eliminated, median final VP 4.5 | Not better |

Sample sizes are small: three runs per card and a single whole game. The Card 2 result is the only one I would call evidence (three wins against a 42% base rate happens by luck about 7% of the time, and all three scores are at or above Hard's p90). Card 1 run 0 is slightly contaminated: I saw four lines of Hard's first-turn orders while testing the harness.

Turn-by-turn notes, plans and mistakes are in [`journal.md`](journal.md). The whole game (config and order tape, with Hard's shadow orders per turn) is in `whole-game/`; `fable_seat view --dir whole-game` replays it.

## How it was run

`crates/sim/examples/fable_seat.rs` is a Rust example that replays a game from a config and an order tape, prints the seat's `PlayerView` as text (cities, units, combat previews, an ASCII map, events), and takes one order file per turn. It reads nothing but the `PlayerView`, changes no rules and no AI code, and is deterministic. Each turn it also records what the Hard brain would have ordered from the same state; `hard` prints that, `autoplay` lets Hard play the seat, and `fork` copies a game up to a turn.

```
cargo build --release -p legatum-sim --example fable_seat
fable_seat init --dir D --new-game --seed 20261111 --faction Athens --seat 0 --run 0
fable_seat view --dir D
fable_seat step --dir D --orders orders.txt
```

## Whole game in numbers

| | T50 VP | T50 cities | T100 VP | T100 cities | T150 VP | T150 cities |
| --- | --- | --- | --- | --- | --- | --- |
| Hard in my seat, mean of 16 salts (range) | 0 (0-0) | 3.3 (2-5) | 2.4 (0-5) | 3.6 (2-5) | 3.1 (0-6) | 3.5 (0-8) |
| Me, one game | 2 | 4 | 4 | 3 | 4 (T137) | 1 (T137) |

I was ahead of every Hard run on VP at T50 and ahead of 14 of 16 at T100 (level with one, behind one), and reached 6 VP at T116 (the lead; the next empire had 4) by finishing the Great Library. I did it with 2 cities and the smallest army on the map. Three neighbours attacked between T107 and T126, the capital fell in one turn, and the wonder's 2 VP went with it.

Hard finishing my game from a fork, one run each: from T25 it won (10 VP, 10 cities, T215); from T50 it ended on 4 VP; from T75 on 5; from T100 it was eliminated at T171; from T117 and T137 it ended on one city and 4 VP. One run each is an anecdote, but it points the same way: my opening was fine, and the position was losing from about T50, when the first Aztec war started.

## Why I lost the whole game

1. **I picked fights I could not finish.** I walked Soldiers past the Aztec capital (war at T49), rejected an Aztec Peace offer at T64 to retake a city and failed, and advanced units onto tiles next to several enemies. Hard's shadow orders had none of these.
2. **I scored VP instead of building an army.** Gold went on buildings that closed era objectives; army strength stayed at about a tenth of my neighbours'. The VP leader with the weakest army is the target the AI's coalition logic is built to hit.
3. **Alliances dragged me into wars.** The Mongol alliance (T83) put me at war with India and then Switzerland; neither would have reached me otherwise.
4. **I stepped several turns per call to save tokens** and missed two Indian Peace offers, a city left idle for four turns, and an empty research queue. A human would have seen all four on screen.

## What Hard could be taught

Each of these is something I did that Hard's orders from the same state did not, and that paid off.

1. **Send two melee attackers at a city on the capture turn.** When the garrison dies, an adjacent enemy unit swaps in before the next melee lands. The first attacker kills the garrison, the second kills the swap-in and takes the city. Hard sends one and fails repeatedly. (Card 2, all three runs.)
2. **Kill the units standing next to the city before walking in.** They are the reinforcements. A Rifleman kills an adjacent Soldier at almost no cost; then the walk-in has nothing to swap against.
3. **Never make an attack the preview says loses.** I held a fortified Rifleman next to a 0 HP city for two turns ("deal 22, take 39") and let three ranged units wear the garrison down, then moved in on the turn the shots would kill. Hard attacks into the bad trade.
4. **Attack on the turn the nearby reinforcement has walked away.** The second Aztec Rifleman moved two tiles off at T137 in run 2; that was the turn to commit everything.
5. **Spend gold on the one unit that can take the objective.** Upgrading a Soldier to a Rifleman (210 gold) won runs 1 and 2. Hard spent on ranged upgrades that could not capture.
6. **Do not lead with mounted units against a city** (-33%), and do not park ranged units within 2 hexes of a city that is about to regain HP: the city strike kills them.
7. **Use Athens' Share the Learning early.** I used it at T25 for 2 permanent VP. Hard had 0 VP at T50 in all 16 runs. The cost is real (each rival gets 3 techs), so this needs a sim, but 2 VP on turn 25 with no production spent looks under-priced by Hard.
8. **Buy the last building an era objective needs.** I bought Workshops at T52 and Mills at T77 to close two objectives for 1 VP each. Hard's mean VP at T100 was 2.4 against my 4.

Where Hard was right and I was wrong, from the shadow orders: it held the Settler when no legal site existed (I built one that never found a site), it wanted Rangers and Walls in threatened cities at T55 and T79 while I built economy, it made no counter-attack after a Peace offer, and it proposed the Great Library at T97 before I did.

## What the text view hid from me

- **The build list offers things that cannot start.** Clearinghouse (needs Niter), College, Forge and Composite Archer (no spare Iron) and Rifleman (needs Coal) all appeared as options. The reason only shows when the order is rejected.
- **Wire names differ from display names** (Laboratory is the Library, MachineShop the Forge, Autoworks the Factory, RelayUplink the Great Library; Spearman, Archer, Pikeman and Crossbowman in events are Soldier, Ranger, Trooper and Marksman).
- **Combat previews ignore everything else that happens the same turn.** "Kill 100%, die 0%" was shown for an attack that killed my unit, because three other enemies hit it in the same resolution. Previews also ignore city strikes on approach.
- **Nothing says that combat resolves before movement**, so a unit ordered to retreat is still hit where it stands. I lost two units learning this.
- **No warning that a foreign Scout can declare surprise war and capture a Settler**, and no warning of air units on Card 1.
- **No list of legal city sites.** Sites need more than 3 hexes from every city; I built a Settler that had nowhere to go.
- **Tech effects, buy prices and upgrade prices are not shown** until an order is refused or affordable.
- **The ASCII map is hard to read** in axial coordinates; I counted cells by hand for every move.
- **Harness faults of my own:** one rejected line refuses the whole order file, and an idle city or an empty research queue is one line in a long view.

## Engine oddities seen (reported, not fixed)

- A `MoveBlocked` message printed offset coordinates ("at 19,27") where every other line uses axial ones.
- Era 3 objective counters went back to 0 after a later era's objective was scored (library count 1/4 to 0/4, population progress to 0/42), and the population counter reset again when I lost a city.
- The players line showed India with "army 1" at a time it was fielding Arquebusiers.
- In the T117 fork, `score` reported the seat alive with 0 cities at T150 and 1 city at the end. It may have been holding a Settler; I did not check.
- Event text uses internal names ("AI 4's Spearman").

None of these changed a result.

## Cost (estimates)

Wall-clock from file timestamps: Card 1 took 4 to 18 minutes per run (18 turns), Card 2 about 3 minutes per run, the whole game about 25 minutes for 137 turns, plus the harness build before that. The 16 Hard comparison games took 5 seconds each. Tokens are a rough estimate from the transcript size: about 4,000 to 6,000 new tokens per whole-game turn when stepping one turn at a time with a filtered view, more than 15,000 with the full view and map, and on the order of 1.5 to 2.5 million tokens of new context for the whole session, with cached re-reads on top.

## What I would do next

The tactical lessons (1 to 6) are specific enough to write as Hard rules or to mine from tapes; Card 2 is a ready test for them. A whole-game teacher needs a second attempt that keeps an army and plays every turn, before its strategic choices are worth copying.
