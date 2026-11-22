# Real natural wonders

Eight recognizable landmarks replace the earlier fictional concepts. Each is assigned to a terrain that fits its real geography and keeps the existing wonder rules: the owning city's territory grants +25% of one yield, and the first discovery pays the existing one-time Gold or Science reward.

![Mobile portrait comparison of all eight real natural wonders in painted-vignette style A and ink-and-wash style B](real-wonders.png)

Style A is the painted jewel vignette used on tile cards and discovery notices. Style B is the ink-and-wash landmark used on the strategic map; it integrates into the terrain and stays legible at the map's normal zoom. Far zoom continues to use the existing compact engraved mark.

| Landmark | Terrain | City yield | Art |
| --- | --- | --- | --- |
| Mount Fuji | Mountains | Science | `mount-fuji-a/b` |
| Victoria Falls | Hills | Food | `victoria-falls-a/b` |
| Grand Canyon | Desert | Science | `grand-canyon-a/b` |
| Giant's Causeway | Hills | Production | `giants-causeway-a/b` |
| Uluru | Desert | Gold | `uluru-a/b` |
| Ha Long Bay | Hills | Food | `ha-long-bay-a/b` |
| Iguazu Falls | Forest | Food | `iguazu-falls-a/b` |
| Great Barrier Reef | Coast | Science | `great-barrier-reef-a/b` |

## Subject prompts

Each source is a transparent square PNG master. The B (ink-and-wash) masters carry real alpha: their black matte was cut out in 8.8.1, because the map draws them unmasked (`app/src/wonders.art.test.ts` checks every one's corners). The A and B prompts below use the same subject descriptions so their silhouettes can be compared directly.

### 1. Mount Fuji

`Mount Fuji in Japan, immediately recognizable symmetrical snow-capped volcanic cone, rising above a low dark band of Japanese pines and foothills.`

- [A painted master](assets/mount-fuji-a-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)
- [B ink-and-wash master](assets/mount-fuji-b-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)

### 2. Victoria Falls

`Victoria Falls in southern Africa, a broad immense curtain of white water spans a basalt cliff, deep green river gorge and a subtle veil of mist; distinctive horizontal falls.`

- [A painted master](assets/victoria-falls-a-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)
- [B ink-and-wash master](assets/victoria-falls-b-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)

### 3. Grand Canyon

`The Grand Canyon in Arizona, monumental layered red and ochre canyon walls receding toward the horizon, Colorado River far below; stepped canyon silhouette.`

- [A painted master](assets/grand-canyon-a-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)
- [B ink-and-wash master](assets/grand-canyon-b-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)

### 4. Giant's Causeway

`The Giant's Causeway in Northern Ireland, dense dark basalt hexagonal columns descend in broad steps into the sea; unmistakable geometric coastal formation.`

- [A painted master](assets/giants-causeway-a-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)
- [B ink-and-wash master](assets/giants-causeway-b-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)

### 5. Uluru

`Uluru in central Australia, one great rounded red sandstone monolith rising alone from flat ochre desert with sparse scrub; unmistakable solitary dome silhouette.`

- [A painted master](assets/uluru-a-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)
- [B ink-and-wash master](assets/uluru-b-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)

### 6. Ha Long Bay

`Ha Long Bay in Vietnam, dense steep limestone karst islands rise vertically from a calm jade sea, distant layered islands; iconic karst seascape.`

- [A painted master](assets/ha-long-bay-a-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)
- [B ink-and-wash master](assets/ha-long-bay-b-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)

### 7. Iguazu Falls

`Iguazu Falls on the Argentina-Brazil border, broad white cascades step over dark basalt into a lush subtropical forest gorge, soft spray; iconic jungle falls.`

- [A painted master](assets/iguazu-falls-a-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)
- [B ink-and-wash master](assets/iguazu-falls-b-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)

### 8. Great Barrier Reef

`The Great Barrier Reef off Queensland, Australia, clear shallow sea with visible branching coral heads, turquoise lagoons and reef islets; iconic living reef pattern.`

- [A painted master](assets/great-barrier-reef-a-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)
- [B ink-and-wash master](assets/great-barrier-reef-b-512.png) · 128 px (archived outside the repository) · 64 px (archived outside the repository)

## Shared style prompts

**A — painted jewel vignette**

> Transparent square PNG game-map art master. Paint the subject as a refined oil-and-gouache jewel vignette sitting entirely within one hex, with a clear iconic silhouette, restrained mineral and earth pigments, elegant atmospheric depth, and a warm parchment-compatible palette. Serious historical strategy-game art. Isolated art with genuine transparent background. No buildings, people, hex outline, frame, cast shadow, sticker, text, labels or UI.

**B — ink-and-wash landmark**

> Transparent square PNG game-map art master. Draw the subject as an engraved atlas landmark embedded into the ground across one or two adjoining hexes: fine sepia contours, controlled cross-hatching, muted watercolor wash, readable at small map zoom, and terrain visible around and through the landmark. Serious historical atlas style with a warm parchment-compatible palette. Isolated art with genuine transparent background. No buildings, people, hex outline, frame, cast shadow, sticker, text, labels or UI.

The mobile portrait comparison sheet shows the painted card vignette and map landmark side by side for every wonder. The app serves the 512 px A art to the tile card and discovery notice and the 128 px B art to the strategic map.
