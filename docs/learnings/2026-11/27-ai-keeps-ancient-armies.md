# Why late armies kept Ancient units

The recorded eight-seat game exposed two independent decisions. At turn 150,
the Swiss empire was in era 5 with 2,594 Gold and 26 combat units eligible to
refit, including Spearmen that could become Riflemen for 210 Gold each. It
issued no refits. The refit pass used a 5,000-Gold faction-goal reserve, so the
available Gold never passed its spending check. `modern_army` level 1 lets an
obsolete unit draw on that saving while preserving the ordinary solvency
reserve and any Gold already scoring a held objective.

Research also failed to change what cities trained. From turns 90–150, the
Swiss empire trained 17 Spearmen, the Aztec empire 21, and the Spartan empire
seven Spearmen and six Archers despite access to later units. At turn 141,
all seven Swiss cities chose Spearmen in the threat pass. Two walled-city
selectors scored linear strength per turn, and the emergency selector scored
linear matchup strength per turn. A one-turn Spearman therefore beat a
three-turn Rifleman. The quick-defender selector had the same cheap-unit
preference. Level 1 now excludes candidates more than one era older than the
newest eligible defender; the walled selectors also score modern combat value.
The floor follows each selector's existing eligibility checks, so a defense
deadline can still leave an Ancient unit as the only possible choice.

Strategic supply and territory remain separate limits. The recorded Spartan
and Byzantine armies lacked spare Coal or Niter for some refits; some Aztec
units stood outside home territory or lacked Gold. This change does not alter
those constraints or any core rule.

The exact-engine replay changed the Swiss threat pass at turn 141 from seven
Spearman production orders to none, while issuing five refit orders. At turn
150, it changed zero refit orders to six. The turn-150 Spartan seat still
selected one Spearman; a timely older unit remains eligible when newer choices
do not survive the existing filters.

In 100 paired six-seat Hard games per arm (turn cap 250), mean combat-unit era
lag at turn 150 was 3.17 at level 0, 2.94 at level 1, and 2.60 at level 2.
Ancient-unit counts were 3,923, 3,225, and 1,867 respectively. At turn 200,
among games still active, lag was 4.15, 3.15, and 2.86; Ancient counts were
3,124, 1,436, and 895. Level 2 also had fewer total combat units, so the
unit-age result alone was insufficient to choose it.

In a separate 300-seed paired seat duel, level 0 won 9.0%, level 1 won 10.7%,
and level 2 won 13.3%. Level 2 versus level 0 gained 4.33 percentage points
in wins [95% interval 0.61, 8.06]; VP share changed by +0.44 points
[−0.88, 1.76]. Level 1's paired win change was +1.67 [−1.60, 4.93] and VP
share changed by −0.05 [−1.27, 1.16]. Level 2 is the shipped default;
its retirement step applies to Hard games.
