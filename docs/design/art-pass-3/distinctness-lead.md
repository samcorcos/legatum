# Distinctness proposals for the lead

**Fresh check: FAIL, exit 1.** Cloud job `20261117T022430Z-a39c4f`, commit `327fad56766f`: 604 fresh renders, 19 sets, 9,750 comparisons, 114 flagged, 49 accepted, 54 advisory, **11 unaccepted failures**. No stale cache was used. All failures are unit miniatures at the check's 45px size (the delivered map sheet uses the requested 48px rasters).

Three failures exceed the 0.60 appearance threshold; eight are gist-only against 0.77. The shared disc is intentional, and no figure was mirrored or reposed to clear a metric. Pikes changed stance for body legibility; Phalangite changed because three tiny figures and chest-centred shields were wrong.

The exact accepted-entry shapes are in [distinctness-proposed-entries.json](distinctness-proposed-entries.json). **These are proposals, not applied approvals.** The first three need particular map-context judgement before acceptance. The remaining eight have clearly different equipment despite coarse silhouette overlap. An accepted similarity pair would not approve the unresolved Archer string, Infantry legibility or compressed pike lengths.

| Pair | Appearance | Gist | Review |
|---|---:|---:|---|
| Barbarian Pikeman / Pikeman | 0.71 | 0.84 | Strongest resemblance. Same pike class; gambeson versus steel plate. Confirm ownerless map treatment. |
| Infantry / Reisläufer | 0.62 | 0.72 | WWII rifle/grenade versus Renaissance pike. Infantry repaint remains blocked. |
| Barbarian Archer / Pikeman | 0.61 | 0.73 | Drawn bow/fur versus pike/plate. Archer string remains wrong. |
| Barbarian Crossbowman / Reisläufer | 0.58 | 0.81 | Hood and low crossbow prod versus feathered cap and pike. |
| Barbarian Archer / Barbarian Pikeman | 0.56 | 0.79 | Fur/bow versus padded cloth/pike. |
| Barbarian Pikeman / Phalangite | 0.50 | 0.80 | Quilted trousers/boots versus bronze/linen, sandals and shield. |
| Archer / Rifleman | 0.49 | 0.81 | Short classical tunic/bow versus blue coat/rifle. |
| Phalangite / Pikeman | 0.46 | 0.79 | Bronze/linen and shield versus full steel armour. |
| Barbarian Archer / Rifleman | 0.46 | 0.80 | Fur hood/bow versus kepi/rifle. |
| Rifleman / Tagmatic Soldier | 0.31 | 0.83 | Blue rifle carrier versus red kite shield and overhead spear. |
| Peltast / Pikeman | 0.26 | 0.78 | Javelin throw and crescent shield versus braced armoured pike carrier. |

Inspected phone sheets: [pairs 1–4](distinctness-review-01.png), [5–8](distinctness-review-02.png), [9–11](distinctness-review-03.png). They show colour and greyscale at 45px plus enlarged art. These Pillow previews are review evidence; scores come from the fresh Chromium renders. The default generated `distinctness-pairs.png` mostly contains intentionally shared roundels; use these failure-specific sheets instead.

**Existing Panzer/Tank exception:** appearance 0.69, gist 0.89; still flags. Keep the exception subject to the lead's judgement, but replace its stale “Panzer repaint queued” wording with the actual result: independent WWII paintings, Panzer IV hull/turret skirts, eight small wheels and long muzzle-braked gun versus the Tank's different hull/turret. The side skirts improve kit recognition; they do not solve the shared tank outline. No acceptance-file edit was made.

The four wall families are not a scored set in the current checker. Their registration and city compositing evidence is in the wall-stage sheets, not implied by this result.
