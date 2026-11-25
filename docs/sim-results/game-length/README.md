# Game length: what ends the game, and which levers move it (2026-11-23)

Sam, 2026-11-22: the game builds slowly, then escalates and ends within a few turns; he rarely gets past the cavalry (Industrial) era and has reached the Modern era about once. Question: is that the tech pace, and would more eras help?

**Decision (Sam, 2026-11-23): no rule change yet.** Wait for the Hard AI work (the hill climb) and Walls levels, then measure his games again. The levers below are ready if pacing still needs it.

## Re-measured with Walls levels (2026-11-25)

Same command and seeds (256 six-empire all-Hard games), on master just before Walls levels (06901ab8, rules 57) and with it (fe6bddee, rules 58):

| | before Walls levels | with Walls levels |
|---|---:|---:|
| no winner % | 0.8 | 0.4 |
| length p10/med/p90 | 178 / 215 / 261 | 179 / 214 / 254 |
| winner era at end (dist %) | Mach:1 WW:13 Mod:39 Fut:48 | Mach:0 WW:15 Mod:37 Fut:48 |
| winner: turns from 5 VP to the win (med) | 44 | 43 |
| captures in last 30 turns / per 30 turns before (mean) | 5.8 / 2.1 | 5.9 / 2.1 |

Walls levels do not change game length or the late capture burst in all-AI games. The median game (214) is a few turns longer than on 2026-11-23 (208), from the AI work since. All-AI sims do not reproduce a human steamroll, so the open check is Sam's own games on rules 58 and later (the uploaded-game analysis). The levers above are unchanged.

## His games

Three web wins on the rules-53-era build (from the uploaded-game analysis): T141, T156, T168, all VP holds. He entered each era every 22 to 25 turns, like the sim leaders, and won in the World War era while his rivals were in Industrial or earlier. 10 to 15 cities fell in his last 40 turns; a rival capital with a world wonder is 3 VP to the taker, so he went from 1 to 5 VP at T100 to the target within about 40 turns.

## Sims

All-Hard, Standard, barbarians on, seeds 9100+ (paired maps across arms), 256 games per arm, safety cap 600. Commands:

```
legatum-sim --games 256 --seed 9100 --players 6 --difficulty hard --safety-cap 600 --tape-out TAPES [--rules-experiment FLAGS]
cargo run --release -p legatum-sim --example escalation_curve -- TAPES 5 > ARM.jsonl
tools/sim/escalation_report.py base=base.jsonl ARM=ARM.jsonl ...
```

Arms: `tc80`, `tc65` = `tech-cost-percent=80/65` (every tech cheaper); `t11`, `t12` = `vp-target=11/12`; combinations; `p8*` = 8 empires. "Era" is 1-based (5 Industrial, 6 Machine Age, 7 World War, 8 Modern, 9 Future).

### 6 empires

| | base | tc80 | tc65 | t11 | t12 | tc80t11 | tc70t11 |
|---|---:|---:|---:|---:|---:|---:|---:|
| games | 256 | 256 | 256 | 256 | 256 | 256 | 256 |
| no winner % | 0.78125 | 0.390625 | 2.34375 | 1.171875 | 2.34375 | 0.0 | 2.34375 |
| length p10/med/p90 | 175 / 208 / 256 | 169 / 202 / 255 | 164 / 199 / 239 | 184 / 220 / 269 | 194 / 234 / 300 | 177 / 212 / 261 | 178 / 211 / 260 |
| winner era at end (median) | 8.0 | 9 | 9.0 | 9 | 9.0 | 9.0 | 9.0 |
| winner era at end (dist %) | Mach:2 WW:15 Mod:39 Fut:44 | Mach:0 WW:9 Mod:27 Fut:63 | WW:4 Mod:12 Fut:84 | Mach:1 WW:7 Mod:28 Fut:64 | Mach:0 WW:4 Mod:16 Fut:79 | WW:2 Mod:18 Fut:80 | Mach:0 WW:1 Mod:11 Fut:87 |
| most advanced empire era at end (median) | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 |
| median empire era at end (median) | 8.0 | 8.0 | 8.5 | 8.0 | 8.5 | 8.0 | 9.0 |
| newest Age revealed at end (median) | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 | 9.0 |
| Age reveal turn (median) II..IX | 21 57 83 108 134 158 181 206 - | 15 49 75 97 121 143 165 189 - | 11 41 67 90 111 131 151 172 - | 21 57 83 108 134 157 181 207 - | 21 57 83 108 134 158 182 207 - | 15 49 75 97 121 143 166 189 - | 12 44 70 92 115 135 156 178 - |
| games revealing Age IX (%) | 53.125 | 72.65625 | 85.9375 | 70.703125 | 83.203125 | 83.59375 | 92.96875 |
| winner: turns from 5 VP to the win (med) | 46 | 46 | 48 | 57.5 | 66.0 | 53.0 | 54.5 |
| winner VP: ages/capitals/wonders/goal | 4.4 / 1.3 / 3.3 / 1.9 | 4.4 / 1.4 / 3.4 / 1.9 | 4.5 / 1.4 / 3.2 / 1.8 | 4.8 / 1.4 / 3.6 / 2.0 | 5.4 / 1.5 / 3.5 / 2.2 | 4.9 / 1.5 / 3.4 / 1.9 | 5.0 / 1.5 / 3.4 / 1.9 |
| captures in last 30 turns (mean) | 5.7 | 5.7 | 5.4 | 5.9 | 5.7 | 5.9 | 5.5 |
| captures per 30 turns before that (mean) | 2.0 | 2.0 | 2.1 | 2.2 | 2.3 | 2.2 | 2.2 |
| prod per empire T50/100/150/200/250 | 49 / 97 / 126 / 143 / - | 51 / 97 / 125 / 146 / - | 54 / 98 / 125 / - / - | 49 / 97 / 127 / 148 / - | 49 / 97 / 128 / 145 / - | 51 / 97 / 129 / 148 / - | 53 / 97 / 125 / 160 / - |
| sci per empire T50/100/150/200/250 | 8 / 33 / 53 / 65 / - | 9 / 34 / 51 / 64 / - | 11 / 36 / 50 / - / - | 8 / 33 / 53 / 67 / - | 8 / 33 / 53 / 66 / - | 9 / 35 / 52 / 65 / - | 11 / 36 / 51 / 65 / - |
| army per empire T50/100/150/200/250 | 85 / 306 / 1113 / 2617 / - | 86 / 346 / 1316 / 3539 / - | 87 / 399 / 1621 / - / - | 85 / 303 / 1130 / 2895 / - | 85 / 303 / 1123 / 2841 / - | 86 / 354 / 1360 / 3719 / - | 87 / 388 / 1469 / 4859 / - |


### 6 and 8 empires, the recommended combination

| | base | t12 | tc80t12 | p8base | p8tc80t12 |
|---|---:|---:|---:|---:|---:|
| games | 256 | 256 | 256 | 256 | 256 |
| no winner % | 0.78125 | 2.34375 | 1.5625 | 1.5625 | 1.5625 |
| length p10/med/p90 | 175 / 208 / 256 | 194 / 234 / 300 | 187 / 222 / 300 | 176 / 212 / 261 | 190 / 228 / 296 |
| winner era at end (median) | 8.0 | 9.0 | 9.0 | 8.0 | 9.0 |
| winner era at end (dist %) | Mach:2 WW:15 Mod:39 Fut:44 | Mach:0 WW:4 Mod:16 Fut:79 | WW:0 Mod:11 Fut:88 | Mach:2 WW:21 Mod:37 Fut:39 | WW:1 Mod:13 Fut:86 |
| most advanced empire era at end (median) | 9.0 | 9.0 | 9.0 | 8.0 | 9.0 |
| median empire era at end (median) | 8.0 | 8.5 | 9.0 | 7.5 | 8.5 |
| newest Age revealed at end (median) | 9.0 | 9.0 | 9.0 | 8.0 | 9.0 |
| Age reveal turn (median) II..IX | 21 57 83 108 134 158 181 206 - | 21 57 83 108 134 158 182 207 - | 15 49 75 97 121 143 165 189 - | 20 56 82 108 137 162 189 217 - | 13 48 74 98 124 146 170 196 - |
| games revealing Age IX (%) | 53.125 | 83.203125 | 92.96875 | 46.484375 | 88.671875 |
| winner: turns from 5 VP to the win (med) | 46 | 66.0 | 62.5 | 46.0 | 65.0 |
| winner VP: ages/capitals/wonders/goal | 4.4 / 1.3 / 3.3 / 1.9 | 5.4 / 1.5 / 3.5 / 2.2 | 5.5 / 1.6 / 3.6 / 2.0 | 4.5 / 1.5 / 3.0 / 2.1 | 5.6 / 1.8 / 3.4 / 2.1 |
| captures in last 30 turns (mean) | 5.7 | 5.7 | 5.7 | 5.8 | 5.8 |
| captures per 30 turns before that (mean) | 2.0 | 2.3 | 2.3 | 2.2 | 2.7 |
| prod per empire T50/100/150/200/250 | 49 / 97 / 126 / 143 / - | 49 / 97 / 128 / 145 / - | 51 / 97 / 127 / 153 / - | 45 / 83 / 105 / 116 / - | 48 / 83 / 103 / 122 / - |
| sci per empire T50/100/150/200/250 | 8 / 33 / 53 / 65 / - | 8 / 33 / 53 / 66 / - | 9 / 34 / 52 / 67 / - | 7 / 30 / 45 / 54 / - | 9 / 31 / 44 / 55 / - |
| army per empire T50/100/150/200/250 | 85 / 306 / 1113 / 2617 / - | 85 / 303 / 1123 / 2841 / - | 86 / 354 / 1350 / 3802 / - | 83 / 271 / 916 / 2107 / - | 87 / 318 / 1111 / 2808 / - |


## Reading

- The tech pace is steady (a World Age every ~25 turns) and barely sets game length: 20% cheaper tech moves every Age ~15 turns sooner and shortens the median game by only 6 turns. It sets the era a game ends in.
- The VP target sets how long the end race lasts: 12 instead of 10 lengthens the winner's climb from 5 VP to the win from 46 to 66 turns and the median game by 26 turns, with 2.3% of games unfinished at turn 600.
- Cheaper tech plus target 12: +14 turns (6 empires) / +16 (8), climb ~63 turns, Future-era winners 44% to 88% (39% to 86% at 8). Faction win rates moved within the noise of 256 games.
- A 10th era added at the end of the tree would not be seen in games that end in the World War era.
- All-AI games do not reproduce the human steamroll: their median is ~T208, his wins T141 to T168. The gap is the AI failing to hold cities.
