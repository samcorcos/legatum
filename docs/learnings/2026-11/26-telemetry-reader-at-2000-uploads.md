# The nightly telemetry reader went blind at 2,000 uploads a day (2026-11-26)

`analyse_games.py telemetry` listed games in key order and stopped at 200. After the game went public a day
held about 2,100 uploads, 1,240 of them zero-turn, and the first keys in the listing were test-looking keys
with old dates, so the nightly read almost no real game. It also made its requests back to back and two
thirds came back refused for a while (every one read fine minutes later).

Now it prints newest first, takes `--min-turns`, waits out a refused request, and each game line carries the
engine commit and the long tasks split by stage of the game with what filled them.

What the first full read showed (232 Chrome games of 30+ turns, one day):

- Judge by the median game, never the mean. The median game has 1 to 2 long tasks per turn early and 2 to 5
  after turn 90. One game in ten has over a second of long tasks per turn (the worst about 11 s per turn) and
  drags the mean of every stage to 5 to 8 per turn.
- No build stands out: the newest build's median game after turn 90 is 1.6 long tasks per turn.
- Terrain painting is the largest named cause at every stage (43 to 48% of long-task time, from turn 1), and
  engine calls grow from 4% early to 22% after turn 200. Small canvases (under 2.5 megapixels) have the most
  terrain time per turn, which points at weak or software-rendering machines, not at Retina screens.
