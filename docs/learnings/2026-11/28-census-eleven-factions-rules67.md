# Eleven-faction census on rules 67: best/worst 11.1, Egypt 35.5% and Rome 3.2%

2026-11-28. Published as measured, no tuning, no rule or AI change. Reports: `docs/sim-results/2026-11-28-census-eleven-factions-rules67.md` (master `7c9bb07b`, rules 67), and three earlier arms on the same seeds, run on 2026-11-27 and not rerun: `docs/sim-results/2026-11-27-census-eleven-factions-rules66.md` (master `82a0e544`), `docs/sim-results/2026-11-27-census-eleven-factions-rules64.md` (master `025325a4`) and `docs/sim-results/2026-11-27-census-eleven-factions-rules64-before-322.md` (master `5f98e07d`).

- **Why:** the last full-size census was rules 50 (2026-11-22, ratio 9.1, Egypt over Rome), and the city-upkeep check of 2026-11-26 ran on the runner's default pool, which has no Egypt or Rome.
- **Runner:** `legatum-sim --all-factions` seats the census design of 2026-11-22 in one run: the 11 cyclic rosters of 8 over `Faction::ALL`, one roster per 8 games, turned one seat per game. `--players` must be 8 and `--games` a multiple of 88; then every faction plays every seat position equally often. Off by default: the nine-faction seeded draw, `--factions` rosters and pinned tapes are unchanged. Before this the design took 11 runs with hand-written `--factions` rosters and `--rotate-factions`.
- **Run:** `cargo run --release -p legatum-sim -- --players 8 --difficulty hard --all-factions --games 2112 --seed 9311003 --label census-eleven-factions-rules67`. 2,112 games, seeds 9311003 to 9313114, 1,536 seats per faction, 0 panics, 0 stalls, about 26 minutes on 8 cores. Held-out seeds were not run. The seeds are not the per-roster seeds of the rules 50 census, so compare rates with it, not games.
- **Four arms, same seeds.** Rules 67 is master `7c9bb07b` with this change's `crates/sim` commits on top (the report records that commit). The older arms are master `82a0e544` (rules 66), `025325a4` (rules 64, after #322, `egypt_peace_army`) and `5f98e07d` (rules 64, before it). Between the two rules 64 arms #322 is the only commit that changes what the AI or the rules do. Between `82a0e544` and `7c9bb07b` four commits touch `crates/ai` or `crates/core` (army refits #357, city automation #376, research queues across eras #335 with rules 67, and the removal of Hard's purchasing and reserve caps #402), so the rules 67 changes are not attributed to one commit here. The rules 67 census was run twice and every results section of the two reports was identical.
- **Win rate per seat (95% Wilson interval for rules 67; even is 12.5%; Change is rules 67 against rules 66):**

| Faction | Seats | Wins | Rules 67 (`7c9bb07b`) | 95% interval | Rules 66 (`82a0e544`) | Change | Rules 64 (`025325a4`) | Rules 64 before #322 (`5f98e07d`) | Rules 50 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Egypt | 1,536 | 546 | 35.5% | 33.2 to 38.0 | 34.0% | +1.5 pp | 37.0% | 14.6% | 21.4% |
| Celts | 1,536 | 252 | 16.4% | 14.6 to 18.3 | 15.9% | +0.5 pp | 16.7% | 18.9% | 14.1% |
| Athens | 1,536 | 202 | 13.2% | 11.6 to 14.9 | 12.6% | +0.6 pp | 12.6% | 17.6% | 14.5% |
| Germany | 1,536 | 199 | 13.0% | 11.4 to 14.7 | 13.4% | −0.5 pp | 13.5% | 16.0% | 14.3% |
| Sparta | 1,536 | 174 | 11.3% | 9.8 to 13.0 | 12.3% | −1.0 pp | 12.3% | 15.5% | 12.0% |
| Switzerland | 1,536 | 172 | 11.2% | 9.7 to 12.9 | 7.9% | +3.3 pp | 9.4% | 13.3% | 10.0% |
| Byzantium | 1,536 | 160 | 10.4% | 9.0 to 12.0 | 11.3% | −0.8 pp | 12.6% | 14.1% | 9.0% |
| Aztecs | 1,536 | 142 | 9.2% | 7.9 to 10.8 | 9.8% | −0.6 pp | 10.4% | 12.6% | 12.8% |
| India | 1,536 | 137 | 8.9% | 7.6 to 10.4 | 10.1% | −1.2 pp | 1.9% | 2.1% | 8.3% |
| Mongols | 1,536 | 79 | 5.1% | 4.1 to 6.4 | 6.0% | −0.8 pp | 6.4% | 6.8% | 18.7% |
| Rome | 1,536 | 49 | 3.2% | 2.4 to 4.2 | 4.0% | −0.8 pp | 4.7% | 5.8% | 2.3% |

- **Ratio:** best/worst 11.1 (Egypt over Rome; interval 7.9 to 15.7, best's lower bound over worst's upper bound and the reverse). Pillar 8's 1.5x is not met. On rules 66 it was 8.4 (Egypt 34.0% over Rome 4.0%), on rules 64 19.6 (Egypt 37.0% over India 1.9%), and 8.8 before #322 (Celts 18.9% over India 2.1%). Without Egypt the rules 67 spread is 5.1 (Celts over Rome).
- **Egypt:** 14.6% before #322, 37.0% after it, 34.0% on rules 66, 35.5% on rules 67: still 2.2 times the next faction (Celts, 16.4%). Its tomb stood in 490 of 1,536 seats (475 on rules 66).
- **Switzerland:** 7.9% to 11.2%, the only move from rules 66 whose two intervals do not overlap (6.7 to 9.4 against 9.7 to 12.9).
- **India:** 8.9% (10.1% on rules 66, 1.9% on rules 64); it reached its goal in 38.5% of seats.
- **Below even in all four arms:** Rome (5.8%, 4.7%, 4.0%, 3.2%) and the Mongols (6.8%, 6.4%, 6.0%, 5.1%; 18.7% on rules 50).
- **Rome's goal:** Rome reached its faction goal in 38 of 1,536 seats (2.5%), the lowest of any faction (next lowest the Mongols, 7.8%), and its Colonia stood in 0 of 1,536 in all four arms.
- **Length and endings (rules 67):** median end turn 244 (mean 252.0, 140 to 500), against 245 on rules 66 and 220 on rules 50. VP hold 91.5%, capitals 8.0% (5.3% on rules 66), turn limit 0.5% (10 games). 99.3% of games reached the World War era.
