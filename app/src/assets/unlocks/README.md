# Painted unlock miniatures

Direction A, chosen by the user on 2026-10-11 (`docs/design/unlock-art/`): every building, faction replacement, wonder (Great Work), tile improvement and technology effect is a AI-generated tabletop miniature on a small patch of ground, in the same family as the city (`../cities/`) and unit (`../units/`) miniatures. Units keep their own art; nothing here duplicates them.

## Files

Each slug has `<slug>-256.png`, `<slug>-128.png` and `<slug>-64.png`, all transparent (RGBA, corners fully transparent). The 1024 px masters and the exact prompt for each painting are in `docs/design/unlock-art/masters/`. Contact sheets: `docs/design/unlock-art/buildings.png`, `buildings-2.png` and `tiles-effects.png`.

| Slug | What |
| --- | --- |
| `building-<kind>` | A building by its core `BuildingKind`, lower case: `building-library` is the Library, `building-forge` the Forge, `building-greatlibrary` the Great Library. |
| `building-<kind>-<faction>` | A faction's own building under its own name: `building-walls-switzerland` (Alpine Redoubt), `building-forge-germany` (Ruhr Foundry), `building-library-athens` (Lyceum). |
| `improvement-<kind>` and `improvement-<upgrade>` | Farm, Pasture, Quarry, Mine, and Irrigated Farm, Ranch, Deep Quarry, Deep Mine. An upgrade is the same subject grown richer. |
| `improvement-saltpetre-works`, `improvement-oil-well`, `improvement-quartz-mine` | Later-era resource works, downscaled from their painted map masters in `docs/design/improvements-painted/sources/`. |
| `effect-<tech>` | A technology's effect, painted as a scene that stands for the rule (the Siege lines of Mathematics, Machinery and Metallurgy are a catapult, a trebuchet and a bombard each on its own ground patch, painted 2026-11-22; they used to reuse the floating siege-prop paintings) (Irrigation: an irrigated field; Railways: a locomotive). Code of Laws has two: `effect-codeoflaws-buy` and `effect-codeoflaws-claim`. Unit upgrade lines show the two unit miniatures instead. |
| `project-<name>` | A city project by its core `IndustryProject`, lower case, painted 2026-11-22 when Gold and Science opened to every city: `project-gold` a mint and treasury yard, `project-science` an open-air scholars' terrace with an armillary sphere, `project-grow` (Egypt's Grow) a Nile field with a shaduf, `project-food` (the Food project, served once `IndustryProject::Food` lands) a provisions yard with a threshing floor, sacks and baskets of produce. Prompts beside the files. An unpainted project falls back to its yield's icon (`ItemArt.tsx`). |

## How the game uses them

`src/unlockArt.ts` is the only lookup (`itemArtUrl`, `effectArtUrl`, `unlockEntryArt`, `slugUrl`). Surfaces: the research cards and their unlock hovercards, the production list and queue (`ui/ItemArt.tsx`), the unlock moment card, the era-up takeover's "New this era", faction piece cards and the Guide. A missing file falls back to the engraved icon, and `src/unlockArt.test.ts` fails if any expected slug lacks a size.

Painted art needs about 24 px to read (legatum-ui); below that, surfaces keep the engraved icon.

The four later world wonders were painted on 2026-11-01 to match the Great Library, Grand Bazaar and Royal Arsenal: `building-pyramids` (Giza, three pyramids with the temple and causeway), `building-hagiasophia` (Justinian's church of 537: dome, semi-domes, buttresses, atrium, no minarets), `building-eiffeltower` (the 1889 tower of puddled-iron lattice on the Champ de Mars; repainted 2026-11-22 with the real tower's curved taper and a full diorama, keyed with `--lattice`) and `building-iterfusionreactor` (the Cadarache tokamak complex, assembly hall and cooling towers). They went through `tools/art/wonder-art.py` (key the generator's magenta field, fit to the shared 1254 px footprint, then `landmark-atlas-edit.py`'s map-palette grade and ground wash); prompts are beside the masters.

Three faction goals with no building of their own have a painted scene for their goal card (2026-11-22): `goal-reserve` (Switzerland's Reserve, an Alpine strongroom with its gold), `goal-warbands` (the Celts' warband muster camp, thatched roundhouses and a carnyx standard, distinct from the barbarian camp) and `goal-newfire` (the Aztec New Fire altar, shown when the rite completes). `celebrate/moments.ts` `GOAL_ART` maps them; prompts sit beside the files.

India uses generated painted miniatures (given a ground patch by an image edit, 2026-11-22) `building-market-india-*` for its Revenue Office (Market replacement) and `building-imperialdiwan-*` for its held-goal building. Prompt records sit beside the assets.

Egypt's goal has a painted scene too (2026-11-25): `goal-royaltomb`, an Old Kingdom walled precinct with a battered mastaba, a sandstone chapel, two small mastabas and one small pyramid. It is the goal card's picture when the tomb stands, and the finished landmark on the map (from its graded copy in `../landmarks/`). Its prompt is beside the master in `docs/design/unlock-art/masters/`, like the world wonders'.

## Card art and map art (2026-11-22)

The seven world wonders and Great Works that stand on the map (`building-pyramids`, `-hagiasophia`, `-eiffeltower`, `-iterfusionreactor`, `-greatlibrary`, `-grandbazaar`, `-royalarsenal`), and Egypt's `goal-royaltomb` (graded by `tools/art/goal-landmark-grade.py`, 2026-11-25), have two copies. `../landmarks/<slug>-256.png` is the map painting, graded to the towns' palette with its ground washed into the hex (`map/landmarks.ts` `landmarkArtUrl` draws it). The files here are the same painting with that grade undone and the ground firmed up (`tools/art/card-grade.py`), because on cards beside ungraded neighbours the map grade read as washed out (art audit 2026-11-22). The Oil Well and Quartz Mine cards were lifted the same way. New pieces go through `tools/art/unlock-art.py RAW.png <slug>` (edge key, fit to the set's footprint, no grade).
