# Ranged garrison regression (playtest-6)

`garrison-t32.json.gz` is the Influence-free migration of the complete pre-T32 `GameState` from the user's
playtest-6 autosave (saved at turn 36, `complete_history: true`, game id
`web-1791054439310-56e37f48728f8000-d5047d6a3768c000`). `garrison-t32-t33-orders.json`
holds every player's recorded orders for T32 and T33 with the migrated resolved hashes.
The original save stays in the user's application data; only this migrated extract is committed.

Archive engine: `a05bf36de6ce15c5bfa72bc6231900b6d3af84eb` (tag `playtest-6`).
Archive rules fingerprint: `724fdc05f876eb0b525809757c2c5a6aa37b2149628ff6a0c836cabaa67bb09a`.
Verified current revision: `ab953373` (rules fingerprint `cf9130b2...`).

The fingerprints differ only because `6a5f3e5f` stopped serializing a zero
`byzantium_salvage_science`. The test restores that field before hashing, which reproduces the
archive's pre-T32 hash `5621332671116605650` and the resolved T32 and T33 hashes. With the same
adjustment, replaying the whole tape on `ab953373` matched the initial hash and all 35 resolved
hashes. A Hard `ScriptedBrain` for P4 (Athens), kept across turns from T1 with
`Rng::derive(setup.seed, [purpose::AI, 4]).next_u64()`, reproduced every recorded P4 order on
every turn, on both `a05bf36` and `ab953373`.

After the Sparta and master rule merges, the unmodified pre-T32 archive state
still has its normalized archived hash. Replaying its recorded T32 and T33 orders
under current rules produces hashes `5236378719036082491` and
`3043467445758995780`. The integration test pins those current replay hashes
and separately checks the archive's original hash strings; it does not rewrite
the saved orders or claim the old resolution is unchanged.

The human (P0, Mongols) declared war on P4 on T29. P4's Ranger u44 entered Resonator (20,15)
on T31. On T32, T33 and T34 it was ordered `HoldUnit` while the human Ranger u37 at (22,13),
two tiles away, shot it every turn (100, 65, 27 HP). On T34 the human Soldier u45 at (19,14) was
also in range. u44 died in the T34 ranged phase without having fired.

Cause: `plan_military` runs `garrison_cities` before `ranged_fire` whenever a Mongols empire is
at war with us (Mongols can cross a screen, so city tiles are secured first). That pass gives
the unit already in the city `hold_or_upgrade`, which used its one order for the turn. The
synthetic test `wounded_ranged_garrison_still_fires...` passed because its enemy is not
Mongols. Holding gained nothing here: a unit that is shot neither heals nor fortifies.

The tests use a fresh brain each turn, at Normal and Hard. The counterfactual moves u45 to
(20,14) (where it attacked from on T35) and sets the city to 10 HP, so the Soldier could take
the city if the garrison falls. The garrison must then shoot u45 rather than u37; with the
capture stake removed from `enemy_threat` it shoots u37 instead.

## Terrain sight integration (2026-10-03)

The archive and its original T32/T33 orders and hashes remain unchanged. Replaying
those orders under terrain sight produces hashes `11201853274943576964` and
`6097713124559188745` (the archived values are `11095118923252569731` and
`12724000898736536245`). From Resonator, the terrain at (21,14) blocks the
two-hex line to the human Ranger at (22,13). The Ranger's recorded volleys no
longer wound the garrison; u44 stays at 100 HP through pre-T34. Holding on
T32–T34 is now legal because there is no visible enemy with a clear shot.

The regression tests open only that firing corridor in copies of the archived
states. With a legal shot restored, the garrison must fire on both Normal and
Hard. The T34 counterfactual also opens the corridor before comparing the
Ranger and adjacent Soldier; both targets are legal, and the Soldier's city
capture threat makes it the more valuable shot. `tactics.rs::enemy_threat`
uses the same PlayerView LOS check when estimating incoming ranged danger.


### Terrain sight integration (2026-10-04)

Resonator is at axial `(20, 15)` and the human Ranger is two hexes northeast.
Their line crosses the Mountain at `(21, 14)`. Current LOS blocks both directions:
the archived T32/T33 shots do not damage the city or garrison; the city instead
regenerates from 117 to 122 to 127 HP. The archived save and orders remain unchanged.
`ranged_garrison.rs` checks that geometry and the resulting holds. Counterfactual
cases replace only that Mountain with Plains to exercise firing from holds and
threat-ranked targeting, retaining the city-shield expectations. An explicit
resolution check verifies that the now-legal incoming shot damages the city,
leaving its garrison at 100 HP. The low-HP capture case still sets a 10-HP city and
27-HP garrison to test the ranged-then-melee exposure sequence.

## Strongest Army scoring integration (2026-10-25)

Current-rule replay hashes now live in `garrison-current-hashes.json`. The new
quality-weighted crown changes recorded military history and can change title
ownership; original archive orders, saved state and historical hashes remain intact.
Regenerate current pins after an intentional rule change, then run the behavioral tests:

```sh
tools/sim/build-slot.sh -- cargo test -p legatum-ai --test ranged_garrison repin_current_garrison_hashes -- --ignored --exact
tools/sim/build-slot.sh -- cargo test -p legatum-ai --test ranged_garrison
```

## Rules cleanup (2026-10-31)

The stored playtest orders and recorded hashes remain provenance. Removing retired scoring and
war-rule state changes the current serialization, so the checked pre-T32 serialization pin and
`garrison-current-hashes.json` were refreshed. The latter was generated with the ignored
`repin_current_garrison_hashes` test above; current-rule T32/T33 replay hashes are
`2686930147353590314` and `18001280338391360285`.
