# Siege props

Painted, free-standing siege gear shown beside a unit that has bought Siege (`app/src/siegeProps.ts`). No stone base, real alpha. Each subject has its full returned source raster, a fitted 128 px derivative (what the map draws), a Lanczos 64 px one, and its generation prompt under `prompts/`. The gear follows the siege tier of the unit's owner (core `data::siege_tier`: Mathematics 1, Machinery 2, Metallurgy 3) and the unit's class.

| Slug | Siege tier | Shown with | Subject |
| --- | --- | --- | --- |
| `catapult` | 1 (Classical) | Ranged units | Ancient wooden torsion catapult with raised arm and sling. |
| `trebuchet` | 2 (Medieval) | Ranged units | Counterweight trebuchet with a tall A-frame, long sling arm and box counterweight. |
| `bombard` | 3 (Renaissance) | Ranged units | Late medieval bronze bombard on a heavy timber bed. |
| `ladders` | 1 | Melee line and mounted | Two lashed scaling ladders with a woven wicker mantlet. |
| `siege-tower` | 2 and 3 | Melee line and mounted | Hide-covered wooden belfry on four wheels, with a raised drawbridge ramp. |
