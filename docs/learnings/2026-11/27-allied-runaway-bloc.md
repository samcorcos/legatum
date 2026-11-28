# An allied runaway read did not add to late defense

An Alliance can concentrate public VP, army strength, captures, and eliminations across its members. The `runaway_bloc` gene lets Hard treat that side as one rival and reconsider idle wars with other viable opponents. It changes diplomatic and target decisions using public information; it does not alter Alliance rules or grant resources.

Alone, the prior 300-pair duel (seed 7001) found +3.0 percentage points in wins, paired 95% interval [+0.3, +6.0], and +0.009 VP share. The required combination with `invasion_defense=1` (seed 7301, 300 paired six-seat games) won 55/300 in both arms: +0.0 points, paired interval [−3.67, +3.33]. VP share fell from 0.17095 to 0.16563 (−0.00532). This fails the combination gate, so `runaway_bloc` stays default 0 while defense is on.

In 32 paired T150 stand-in branches through T170, the leader bloc captured or razed 3.625 rival cities with both genes versus 3.75 at baseline; rival cities remaining rose from 14.34 to 14.78, while four rival seats remained alive in both arms. The T130 branch had 4.531 versus 4.781 captures, but its recorded snapshot has a load-hash compatibility caveat on the current engine. A favorable solo duel therefore did not establish a favorable combined default.
