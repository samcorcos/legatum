# Era takeover key art

Seven era-generic full-screen paintings for the approved era overhaul. These shared scenes celebrate human progress and are intentionally not faction-specific. They are wired into the era-up takeover (`app/src/celebrate/EraUpTakeover.tsx`, `eraArt.ts`), which loads the two JPEG derivatives below (`<era>-1280.jpg`, `<era>-1920.jpg`, quality 84, about 0.5 and 0.9 MB) through `srcset`, so the bundle does not carry the 3.5 MB PNG masters twice. The 720 × 1280 phone crops stay unused (phone is not a target).

Each master is 1920 × 1080. The 1280 × 720 image is a landscape derivative; the 720 × 1280 portrait file is a focal-aligned 9:16 crop composed for a phone takeover. Each master has a neighboring .prompt.txt with the complete generation prompt. The series uses a refined oil-paint technique, visible brushwork, satin glazes, subtle atlas-like shadow hatching, and parchment-compatible pigments.

| Era | 1920 × 1080 master | 1280 × 720 | 720 × 1280 phone crop | Prompt |
|---|---|---|---|---|
| Ancient | [ancient.png](../../../app/src/assets/eras/ancient.png) | [ancient-1280.png](../../../app/src/assets/eras/ancient-1280.png) | [ancient-portrait.png](../../../app/src/assets/eras/ancient-portrait.png) | [ancient.prompt.txt](../../../app/src/assets/eras/ancient.prompt.txt) |
| Classical | [classical.png](../../../app/src/assets/eras/classical.png) | [classical-1280.png](../../../app/src/assets/eras/classical-1280.png) | [classical-portrait.png](../../../app/src/assets/eras/classical-portrait.png) | [classical.prompt.txt](../../../app/src/assets/eras/classical.prompt.txt) |
| Medieval | [medieval.png](../../../app/src/assets/eras/medieval.png) | [medieval-1280.png](../../../app/src/assets/eras/medieval-1280.png) | [medieval-portrait.png](../../../app/src/assets/eras/medieval-portrait.png) | [medieval.prompt.txt](../../../app/src/assets/eras/medieval.prompt.txt) |
| Renaissance | [renaissance.png](../../../app/src/assets/eras/renaissance.png) | [renaissance-1280.png](../../../app/src/assets/eras/renaissance-1280.png) | [renaissance-portrait.png](../../../app/src/assets/eras/renaissance-portrait.png) | [renaissance.prompt.txt](../../../app/src/assets/eras/renaissance.prompt.txt) |
| Industrial | [industrial.png](../../../app/src/assets/eras/industrial.png) | [industrial-1280.png](../../../app/src/assets/eras/industrial-1280.png) | [industrial-portrait.png](../../../app/src/assets/eras/industrial-portrait.png) | [industrial.prompt.txt](../../../app/src/assets/eras/industrial.prompt.txt) |
| Modern | [modern.png](../../../app/src/assets/eras/modern.png) | [modern-1280.png](../../../app/src/assets/eras/modern-1280.png) | [modern-portrait.png](../../../app/src/assets/eras/modern-portrait.png) | [modern.prompt.txt](../../../app/src/assets/eras/modern.prompt.txt) |
| Future | [future.png](../../../app/src/assets/eras/future.png) | [future-1280.png](../../../app/src/assets/eras/future-1280.png) | [future-portrait.png](../../../app/src/assets/eras/future-portrait.png) | [future.prompt.txt](../../../app/src/assets/eras/future.prompt.txt) |

## Prompts

The complete prompts are saved beside each master. Each scene brief below is the central subject instruction used for that era; follow its link for the full prompt.

### Ancient

a thriving Bronze Age river city at first light, monumental mudbrick terraces and temple precinct beside a broad fertile river, irrigation canals, fields, grain boats, workshops, builders placing a foundation stone. Historically grounded ancient Near Eastern visual language, no anachronisms.

Full prompt: [ancient.prompt.txt](../../../app/src/assets/eras/ancient.prompt.txt).

### Classical

a magnificent Classical-era Mediterranean city at the height of civic life: marble colonnaded civic buildings and temple on a gentle acropolis above a lively harbor, stone roads, workshops, scholars and artisans gathering in a sunlit public square, sailing ships and terraced orchards beyond. Historically grounded Greek and Mediterranean classical visual language, no anachronisms.

Full prompt: [classical.prompt.txt](../../../app/src/assets/eras/classical.prompt.txt).

### Medieval

a thriving 13th-century European cathedral-and-castle town: soaring stone cathedral with flying buttresses and towers, fortified hilltop castle, dense guild streets, timber bridges across a river, market and craft workshops, cultivated fields beyond the walls; a small procession of mounted knights returns peacefully under banners without writing while townspeople celebrate civic life. Historically grounded architecture, clothing, materials and scale; no fantasy.

Full prompt: [medieval.prompt.txt](../../../app/src/assets/eras/medieval.prompt.txt).

### Renaissance

a prosperous 15th-century Italian city at golden morning, elegant domed cathedral and civic palazzi, workshops where artisans and architects study drawings and a carefully built wooden model, a printing press in an open guild courtyard, merchants and citizens crossing a great piazza, distant hills and aqueduct. Respect period architecture, dress and tools; no identifiable famous person.

Full prompt: [renaissance.prompt.txt](../../../app/src/assets/eras/renaissance.prompt.txt).

### Industrial

a late 19th-century industrial panorama: a new railway sweeps across the foreground on an elegant iron viaduct toward a prosperous city of brick factories and tall chimneys releasing light, modest clouds of steam; a monumental iron-and-glass world's fair exhibition hall glows at the center, surrounded by gardens, fountains and visitors. A steam locomotive and ships show engineering, labor and trade; dignified small figures. Historically grounded technology and architecture, no anachronisms.

Full prompt: [industrial.prompt.txt](../../../app/src/assets/eras/industrial.prompt.txt).

### Modern

a flourishing international mid-20th-century city with elegant early modern skyscrapers, broad tree-lined boulevard, harbor and civic park; several silver propeller aircraft cross the open sky above, with one early jet distant. Trams, cars and people animate the streets in a peaceful public celebration. Period-accurate mid-20th-century buildings, aircraft and vehicles, no anachronisms, no flags or readable signage.

Full prompt: [modern.prompt.txt](../../../app/src/assets/eras/modern.prompt.txt).

### Future

a hopeful near-future panorama, plausible within the next generation and firmly human-centered: a beautiful sustainable city at sunrise with elegant contemporary architecture and generous gardens, a monumental fusion energy plant integrated into civic infrastructure, calm and luminous but without an energy beam; small service drones in the sky; a few friendly humanoid robots and people working together in a plaza, families visiting a new public bridge. Humanity has advanced into a peaceful flourishing future; never dystopian, militarized or fantastical.

Full prompt: [future.prompt.txt](../../../app/src/assets/eras/future.prompt.txt).

## Contact sheet

![Seven era takeover paintings in a vertically stacked phone-friendly contact sheet](era-key-art.png)


## In the game

The takeover at 1000 × 600 and 1440 × 900, and the board repainted by era, are in [`in-game/`](in-game/) (written by `app/e2e/era-takeover.spec.ts`). The takeover covers the screen with the era's painting, a short animation (under 3 s; none under reduced motion), then what changes: new units and buildings, the era's three objectives, the crown race and how your cities now look. Return, Esc or Continue closes it at any moment.
