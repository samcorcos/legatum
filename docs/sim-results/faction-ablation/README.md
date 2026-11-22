# Faction rule ablation: every faction, 2026-11-05/06

Each faction rule switched off on its own (`legatum-sim --rule-off`, `tools/sim/ablate.sh`), 4,096 eight-empire Hard AI-only games per arm, seed 9311003, paired by seed against one shared control (Germany ran against its own control on an earlier commit; the empty switch set is byte-identical). Δ is the faction's win share with the rule off minus control, with a 95% bootstrap interval. A strength should cost share when removed, a drawback should give share back. Per-faction tables: the `<faction>-<date>/README.md` folders here.

**Caveat:** these are AI-only games. A "dead" strength can mean the AI does not use the rule well, not only that the rule is weak; check telemetry before cutting.

## Rules that don't earn their place

| Rule | Kind | Δ win share | Reading |
|---|---|---|---|
| Germany: Ruhr Foundry (Factory slot) | strength | +0.05 | dead; arrives median T141 of a T183 game (being reworked to the Forge slot) |
| Germany: 10% first-building discount | strength | −0.38 | dead |
| Germany: Industry projects | strength | −0.05 | dead |
| Germany: Five Ruhr Foundries goal | strength | +0.14 | dead (5% of seats ever hold it) |
| Byzantium: Mercenary Companies | strength | **+8.84** | **backwards**: Byzantium wins 20.2% without them; the AI's hiring loses it games |
| Byzantium: thin levies | drawback | +0.80 | dead |
| Celts: Ambusher (+3 ranged) | strength | −0.05 | dead |
| Celts: sack Gold and camp | strength | −0.60 | dead (the cost of not keeping cities, +5.1, is what bites) |
| Mongols: Peace passage | strength | +0.66 | dead |
| Mongols: Keshig | strength | +0.57 | dead |
| Sparta: slow healing | drawback | +0.90 | dead |
| Sparta: desertion at peace | drawback | +0.69 | dead |
| Sparta: Honour the Fallen goal | strength | +0.00 | dead: scored in 7% of seats at median T166, changed 4 of 4,096 games |
| Switzerland: buying world wonders | strength | −0.44 | dead |
| Switzerland: 1,000 Gold hoard goal | strength | **+1.27** | **backwards**: chasing the hoard costs the AI more than the 2 VP gives |
| Athens: ruins Science | strength | −1.10 | inconclusive |
| Switzerland: cheaper purchases (175%) | strength | −1.08 | inconclusive |

## Rules that carry their faction

- **Biggest strengths:** India's network (−8.1), Athens's Lyceum (−7.7), Sparta's kill heal (−7.5) and veterans (−6.3), the Celts' barbarian peace (−5.7), Swiss tolls (−5.7) and redoubt (−4.9), Mongol +1 move (−4.3), India's close cities (−4.1), Swiss interest (−4.0).
- **Biggest drawbacks:** Sparta's +50% non-military cost (**+17.5**: Sparta would win 30%), India's remittance (+8.0) and corruption (+6.5), the Celts not keeping cities (+5.1), the Mongols' no Walls (+4.8), Germany's −1 move (+3.4), Swiss upkeep (+2.9), Aztecs no horses (+2.7), Athens's weak Markets (+2.6).
- Byzantium's Imperial Scriptorium pays, barely (−1.13 [−2.20, −0.06]).
- Every Aztec and India rule pays.

Run directories: `~/LegatumRuns/cloud/ablate-<faction>-<rule>-*`, control `ablate-athens-control-1105-225238` (Germany: `ablate-germany-control-1105-212712`).
