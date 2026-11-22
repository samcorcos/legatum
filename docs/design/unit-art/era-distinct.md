# Era-distinct units: options (2026-10-11)

![Options sheet](era-distinct-options.png)

The user, playtesting 8.8.1 as Sparta: "The Hoplite and the plus-one era from the Hoplite looked nearly identical when I'm playing the game, and I just attacked some things not realizing it." At normal zoom a miniature is about 30 px tall (`tokenHalf` caps at 18, so zooming in never makes it bigger), and the Spearman, Hoplite, Phalangite and Pikeman were all one upright man with an upright spear and a round shield.

Five options were mocked in the real map on branch `claude/unit-distinct` (not merged): a dev switch `?unitera=numeral,trim,strength,stat` (also `window.legatumMap.setUnitEra('numeral,strength')`), fixture `app/src/test/unitEraFixture.ts` (your Spartans against Athens, every era of each class within sight), and spec `app/e2e/unit-era.spec.ts`. The shots (JPEG) are in `era-distinct/` at 1000x600 and 1440x900; the `sil-` shots swap in the five repainted samples.

1. **Silhouette per era** (samples in `samples/`, Luna task `unit-era-samples`): Phalangite as a three-man block with sarissas lowered, Pikeman in bright steel with a braced diagonal pike, Composite Archer kneeling with a pointed hood, Cataphract with the lance couched horizontally on a grey scale-armoured horse, Cuirassier on a black horse firing a pistol. All pass the unit alpha test. The lookalike check (`tools/art/distinctness.py`, units-miniature, 45 px) drops phalangite/pikeman 0.66 to 0.34, archer/composite-archer 0.71 to 0.27, cataphract/horseman 0.69 to 0.45, cuirassier/horseman 0.75 to 0.38, cataphract/cuirassier 0.60 to 0.22. That check scored the user's actual pair (Hoplite and Phalangite) at only 0.38 before the repaint, so it is not enough on its own.
2. **Era tab:** a Roman numeral on the front of the base, filled with the era's metal. It is square-cornered, because the route preview already uses red-ringed Roman numerals for turns. Normal zoom only.
3. **Era-metal ring** around the base. It fights the owner colour, so it is not recommended.
4. **Strength cue:** with one of your military units selected, each rival military unit carries a red rising mark with how much stronger it is in that fight, a green falling mark with how much weaker, or an even mark, counting the class bonus. Far out, only the mark shows. The attack forecast leads with "Phalangite, one era newer: 26 vs your 22. It outclasses you."
5. **Strength number on every base.** It misleads for ranged units (an Archer shows 8 but shoots at 15) and ignores the class bonus, so it is not recommended.

**Recommendation:** 1 + 2 + 4. The user's reply is pending.

**If the silhouettes are chosen, the full repaint also fixes the UI review's minors on the five era units:**
- the roundels' weapons run over the owner rim, and the Cataphract and Cuirassier roundels carry parchment marks; redraw them as ink-only inside the rim;
- the Arquebusier roundel reads as a tick at 18 px;
- the build-card thumbnails are about 24 px, too small for tall figures; crop them.

Every figure must pass `unitArt.alpha.test.ts` and the distinctness check.
