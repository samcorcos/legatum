# Celebration card vignettes

Nine square paintings in the reward cards in `MomentLane`. The slot is **104 × 104 px** on desktop and **56 × 56 px** below 960 px wide (`celebrate.css`). Each `moment-*-104.png` is a direct high-quality resize of its existing 1254 × 1254 master; the square framing is retained so each subject stays legible in both slot sizes. The card keeps its engraved reward emblem and text beside the painting. Captured cities use the gold card; lost cities keep the sober red frame, kicker and muted painting.

`moment-vignettes-104-sheet.png` is the nine-image contact sheet, with every image shown at its actual 104 px slot size. Source masters and their generation prompts remain together in [`../moments/`](../moments/); each prompt is named `moment-<name>.prompt.txt` beside its master.

| Moment kind in `app/src/celebrate/` | Card image | Source painting and prompt |
| --- | --- | --- |
| City founded (`CityFounded`) | `moment-founded-104.png` | [`moment-founded.png`](../moments/moment-founded.png) · [`prompt`](../moments/moment-founded.prompt.txt) |
| City grows (`CityGrew`, when shown as a card) | `moment-growth-104.png` | [`moment-growth.png`](../moments/moment-growth.png) · [`prompt`](../moments/moment-growth.prompt.txt) |
| Wild camp cleared (`BarbarianCampCleared`; also the Celts' camp-goal loss) | `moment-camp-cleared-104.png` | [`moment-camp-cleared.png`](../moments/moment-camp-cleared.png) · [`prompt`](../moments/moment-camp-cleared.prompt.txt) |
| Ruin explored (`RuinExplored`, any reward) | `moment-ruin-explored-104.png` | [`moment-ruin-explored.png`](../moments/moment-ruin-explored.png) · [`prompt`](../moments/moment-ruin-explored.prompt.txt) |
| Title, era objective, or veteran-retirement chronicle moment (`TitleChanged`, `EraObjectiveScored`, `VeteranRetired`) | `moment-milestone-104.png` | [`moment-milestone.png`](../moments/moment-milestone.png) · [`prompt`](../moments/moment-milestone.prompt.txt) |
| Great Work completed (`ProductionCompleted` for a Great Work) | `moment-greatwork-104.png` | [`moment-greatwork.png`](../moments/moment-greatwork.png) · [`prompt`](../moments/moment-greatwork.prompt.txt) |
| City captured (`CityCaptured`; also Heir captured or retaken) | `moment-captured-104.png` | [`moment-captured.png`](../moments/moment-captured.png) · [`prompt`](../moments/moment-captured.prompt.txt) |
| City lost or razed (`CityCaptured`, `CityRazed`, loss card) | `moment-lost-104.png` | [`moment-lost.png`](../moments/moment-lost.png) · [`prompt`](../moments/moment-lost.prompt.txt) |
| Technology learned (`TechResearched`, partner card) | `moment-tech-104.png` | [`moment-tech.png`](../moments/moment-tech.png) · [`prompt`](../moments/moment-tech.prompt.txt) |

The nine image names match the existing nine square paintings. Three wide takeover paintings are a separate stage and are not included here.
