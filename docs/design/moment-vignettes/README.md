# Moment vignettes: art directions

This is a visual exploration only. It adds no app wiring. The existing `emblem-*.svg` reward marks remain the small engraved report treatment; these paintings are candidates for the celebration cards.

![Portrait comparison sheet: three moment-card directions and nine painted scenes](moment-vignettes-directions.png)

The portrait comparison sheet is [moment-vignettes-directions.png](moment-vignettes-directions.png). It compares card mocks at the requested 1000 × 600 and 1440 × 900 viewports, then shows all nine paintings. The card mock follows the real card's 440 px maximum width, parchment panel, headline, effect row, and 44 px “Show on map” action. The map around it is a drawn context mock, not a screen-tour capture. `art-audit/notification-card.png` remains the real card reference.

## Directions

| Direction | Treatment | Height budget at 1000 × 600 |
|---|---|---|
| A · Round medallion | One 60 px painting crop inside the existing double-ring gilt frame. | 330 px card mock; no added height over today's card. The 44 px Show on map button is visible with about 130 px of vertical room beneath the card. |
| B · Wide banner | A 3:1 painted strip across the 440 px card; the headline plaque overlaps its lower edge. | 440 px card mock, centered at y=87; Show on map ends at y=434, leaving 166 px before the viewport bottom. |
| C · Full takeover | 16:9 painted backdrops behind the parchment card for city founded, Great Work completed, and city captured. The other six use A's round painted medallion. | 370 px overlay card mock; Show on map ends at y=435. Art fills the screen so the card does not need a taller content stack. |

These are measured from the design mocks, not runtime CSS. The live UI still needs a final 1000 × 600 check when a direction is wired. The button remains visible in each mock.

## Recommendation

Choose C. The three defining moments receive the broad, memorable scene a playtester asked for; all six other frequent moments still show their complete painting in a 60 px ringed medallion. This keeps routine cards compact while giving founding, completion, and capture the ceremony that benefits from a takeover. A is the lowest-risk fallback if takeovers feel too interruptive. B makes the painted art most visible on every card, and the mock fits, though its 440 px card uses most of the short-window height budget.

## Paintings and prompts

Each painting has a generated master PNG, a 512 × 512 PNG, a 256 × 256 JPEG (quality 88), and an adjacent `.prompt.txt`. Takeover masters are 1672 × 941 (16:9); their derivatives are 512 × 288 PNG and 256 × 144 JPEG. The nine square masters are 1254 × 1254; their derivatives are 512 × 512 PNG and 256 × 256 JPEG.

| Moment | Asset basename | Scene |
|---|---|---|
| City founded | `moment-founded` | Foundation stone, survey cord, stakes, hill settlement |
| City grows | `moment-growth` | Sprouting field, added clay-roof houses |
| Wild camp cleared | `moment-camp-cleared` | Broken palisade, torn pennant, cold fire |
| Ruin explored | `moment-ruin-explored` | Half-buried colonnade, recovered amphora |
| Milestone or title won | `moment-milestone` | Laurel, ribbon, blank scroll |
| Great Work completed | `moment-greatwork` | Timber crane lifting a carved stone arch |
| City captured | `moment-captured` | Gate with raised banner |
| City lost | `moment-lost` | Gate under distant smoke |
| Technology learned | `moment-tech` | Lamp-lit blank scroll, astrolabe, dividers; era-neutral |
| Founded takeover | `moment-takeover-founded` | Wide foundation scene, negative space for overlay |
| Great Work takeover | `moment-takeover-greatwork` | Wide crane and arch scene, negative space for overlay |
| Captured takeover | `moment-takeover-captured` | Wide captured gate and banner, negative space for overlay |

All prompts specify refined oil technique, parchment-compatible pigments, no text, no faces, no fantasy or sci-fi, and no relay-star motif. Per-image prompt text is next to its source asset under `app/src/assets/art/moments/`.

Prompt subjects, in that same order: “newly founded ancient Mediterranean hilltop settlement, large limestone foundation stone, taut survey cord and stakes”; “ripe sprouting grain field, newly added clay-roof homes”; “broken timber palisade, torn pennant, cold hearth, scattered shield”; “half-buried classical colonnade and recovered amphora”; “laurel wreath tied with faded vermilion ribbon beside a blank scroll”; “timber crane lifting a carved limestone arch”; “captured stone gatehouse, raised muted vermilion banner, abandoned shields”; “lost city gatehouse under distant smoke at dusk”; “oil lamp, bronze astrolabe, dividers, blank scroll.” The three wide prompts reuse the founding, arch-lifting, and captured-gate subjects in a cinematic landscape composition with center space for the card. Full prompts, including each common style and exclusion clause, are saved beside the corresponding masters.

## In-report scale and file sizes

The bottom strip of the comparison sheet shows representative engraved marks at an actual 24 px diameter, within the existing 22–26 px report size. The source emblems remain `app/src/assets/rewards/emblem-*.svg`; no replacement report glyphs are proposed here.

| Deliverable | Dimensions | Size |
|---|---:|---:|
| Moment | Master PNG | 512 px PNG | 256 px JPEG |
|---|---:|---:|---:|
| Founded | 3.2 MB | 551 KB | 44 KB |
| Growth | 3.0 MB | 516 KB | 39 KB |
| Camp cleared | 3.3 MB | 541 KB | 42 KB |
| Ruin explored | 3.2 MB | 544 KB | 43 KB |
| Milestone | 2.6 MB | 470 KB | 35 KB |
| Great Work | 2.9 MB | 498 KB | 37 KB |
| Captured | 3.1 MB | 507 KB | 37 KB |
| Lost | 2.7 MB | 453 KB | 30 KB |
| Technology | 2.5 MB | 460 KB | 33 KB |
| Takeover: founded | 3.0 MB | 297 KB | 22 KB |
| Takeover: Great Work | 2.6 MB | 280 KB | 21 KB |
| Takeover: captured | 2.8 MB | 278 KB | 20 KB |

The comparison sheet is 1200 × 4050 PNG, 2.2 MB. The seven full-size card mocks range from 49 KB to 1.5 MB (takeover scenes carry most of that size). Sizes are rounded as shown by `ls -lh` on 2026-10-10. Individual assets are intentionally kept separate so the lead can select and wire a direction later.

## Review files

- `mockups/a-1000x600.png`, `mockups/a-1440x900.png`
- `mockups/b-1000x600.png`, `mockups/b-1440x900.png`
- `mockups/c-founded-1000x600.png`, `mockups/c-greatwork-1440x900.png`, `mockups/c-captured-1000x600.png`
- `moment-vignettes-directions.png` — portrait, mobile-scroll comparison plate

The requested screen-tour shots were not present. A full screen-tour run was attempted; its first 15 cases all failed at Chromium launch (`bootstrap_check_in … Permission denied`) and the remaining 103 did not run. These are labeled card mocks rather than captures from the running app.
