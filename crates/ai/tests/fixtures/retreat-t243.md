# T243 emergency retreat regression

Game `native-1790550637054682000-56150-9`, incident I000819.
The state and simultaneous orders were extracted by replaying the archived tape
with engine `db4a21f05b3d327d30bd1e6d47f75282e236bb12`. All 343 post-turn hashes
matched. The archive is not replay-compatible with the current engine (the first
turn already differs), so the test loads this exact start-of-T243 snapshot and
uses the current AI and resolver. Missing newer save fields take their normal
serde defaults. No hidden state is passed to the AI: it receives `player_view`.

Source: `~/LegatumData/archive/2026/09/native-1790550637054682000-56150-9.tape.json.zst`.
Start-of-T243 historical state hash: `16598103840292138788`.

Before the fix, current Normal AI reproduces P1's archived T243 orders: Trooper
162 holds at (0,17), taking Ranger 68's volley from 54 to 41 HP. The planner's
retreat competes with Soldier 148 and Soldier 6. Holding 148 and moving 162 to
(-1,18) is legal and escapes with the same 41 HP. Ranged damage resolves before
movement, and ID permutations can change its seeded amount; the test permits it.
