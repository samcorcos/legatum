# Crown upkeep and the Gold surcharge on the lead over the mean (2026-10-18, `claude/crown-mean`)

Follow-up to `decadence-mean-window50.md`. The user, asked whether crown upkeep should move to the mean too: "I'll take your recommendation." Crown upkeep's who-pays test (`scoring::ahead_of_average`, was `ahead_of_median`), the waived-crown line and the Gold surcharge now use the lead over the **mean** Power of the living empires, the same number as decadence (`scoring::lead_percent`; `PlayerPublic.lead_percent` and `mean_lead_percent` are equal unless a knob splits them). `crown-median` restores the median for crowns and the surcharge, `decadence-median` for decadence. Everything the player reads says "ahead of the average empire".

## Sweep (Hard + Search, Standard, 256 games per lobby and cell, 4p + 8p pooled, paired seeds, one binary; old = `crown-median`)

| Metric | Old (crowns on the median) | New (crowns on the mean) |
|---|---:|---:|
| Median game length (turns) | 300 | 301 |
| No winner (%) | 0.4 | 0.4 |
| T50 / T100 leader wins (%) | 43.5 / 42.7 | 41.4 / 43.5 |
| Lead changes per game | 11.52 | 11.23 |
| Crown flips per game | 44.8 | 44.5 |
| Overreach paid by the VP leader (Gold a turn / % of its city Gold) | 4.87 / 13.3 | 5.23 / 14.4 |
| Crown-holder turns with an empty treasury (%) | 1.1 | 1.2 |
| Units deserted for unpaid overreach per game | 1.79 | 1.89 |
| Top Power over median / over mean, T100 (%) | 35.5 / 37.1 | 36.1 / 37.4 |
| Top empire's decadence, mean (%) | 15.3 | 15.5 |
| Highest decadence rate in any game (%) / games past 50% | 64 / 4 | 60 / 3 |

Nothing moves outside the noise (about 2 points on a win share): a crown holder who is above the median but below the mean is rare, because the leader pulls the mean up and the empires that hold crowns are the ones ahead of both. The cost is the leader paying 0.4 Gold more on average.

## Human-seat benchmark (Hard, 256 salts per map, 8 maps, all seats; control by flag on the same binary)

| | Salts 1 to 256 | Salts 257 to 512 | Pooled |
|---|---:|---:|---:|
| Old (`crown-median`) | 29.4% (27.6 to 31.2) | 28.0% (26.3 to 29.7) | 28.7% |
| New | 28.7% (27.0 to 30.5) | 30.0% (28.4 to 31.8) | 29.4% |

Not worse (+0.7 pp pooled).
