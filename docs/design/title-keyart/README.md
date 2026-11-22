# Historical title key art

Three key art candidates for the title screen and Guide backdrop. Direction C, “The road at dawn,” is approved and wired on 2026-10-10. Each master is 2560 × 1440 PNG; each 1600 × 900 JPEG derivative was made with `sips -s format jpeg -s formatOptions 88 -Z 1600`. Image generation returned 1672 × 941, so the PNG masters are resampled enlargements. Prompts are kept next to the masters in `app/src/assets/art/`.

The six title comps are at both requested window sizes. They use the supplied `docs/design/art-audit/title.png` crop for the original layout, with each painting under the title frame and wash. They are crop-based layout studies, not fresh app renders. The selected composition now appears on both the title screen and Guide.

## A · The Old World at dusk

Files: removed from the tree on 2026-11-21 (unused; git history has `app/src/assets/art/title-keyart-a-old-world-dusk.*`). The comparison sheet below shows it.

A continent-scale Alpine overlook with river plains, steppe, coasts, distant islands, warm cities, faint roads, and a fallen colonnade beside a dry-stone wall in the left foreground. The high ridge makes the land feel vast; the ruin gives the old-world theme an immediate emotional anchor.

Legibility: the upper centre is pale sky and distant haze behind the wordmark, while the lower-centre slip remains a solid light card over the land. The skyline under the tagline is a little more active than C, but the full wordmark and menu remain clear in the crop-based mock. The rich left foreground gives the title a memorable first read.

Pros: strongest balance of monument, inhabited continent and broad vista; exposed left third has specific detail; the toppled classical stone is an era-neutral sign of a past age.

Cons: the colonnade is recognizably classical and could pull the scene toward Greece if its prominence grows; warm dusk resembles the beloved current composition closely, so it changes the setting more than the mood.

## B · The great survey

Files: removed from the tree on 2026-11-21 (unused; git history has `app/src/assets/art/title-keyart-b-great-survey.*`). The comparison sheet below shows it.

An almost-overhead continent rendered as oil-painted land with engraved relief hachures and branching rivers. Small survey landmarks include a hill-fort, steppe camp, rice terraces, Byzantine dome and Greek temple.

Legibility: the upper centre is the calmest wordmark field of the three. The menu card sits over detailed lower terrain, but its parchment panel is opaque enough to hold contrast. Fine terrain detail becomes busiest at the edges and around the lower half of the screen.

Pros: the clearest atlas identity; calm pale title field; the landmarks make several historical settings visible without a national flag or leader portrait.

Cons: most map-like and least immediately cinematic; the dense relief and landmark vocabulary may feel illustrative at title scale; it is easiest to mistake for a labeled map if future iterations add more detail.

## C · The road at dawn

Files: the full-size master title-keyart-c-road-at-dawn.png (archived outside the repository; `python3 tools/art/fetch-sources.py --all` restores it), `title-keyart-c-road-at-dawn-1600.jpg`, and adjacent `.prompt.txt`.

A single long road crosses a ridge, plains and river valleys into dawn haze and distant sea. Tiny separated hearth smoke and watch-fires suggest many societies; weathered ruins sit in the lower-left grass.

Legibility: the wordmark has a wide, quiet, bright sky behind it, the strongest uninterrupted text field. The menu card crosses the road and lower land but remains distinct on its pale slip. The tagline field is the least cluttered.

Pros: best title and tagline separation; a simple road conveys connection, migration and history across the continent; restrained, calm and very legible.

Cons: less monumental than A; the road is a strong compositional line and could compete with the menu if it were made brighter or more central; hearth smoke is subtle enough that the separate peoples may not register on smaller displays.

## Approved direction

The lead selected **C, the road at dawn**. It gives the wordmark and tagline the clearest quiet sky and carries the eye from first settlement toward an open horizon. The title subtitle and Guide note use the short historical line “From the first city to the age to come.” The former “star empire” tagline and “Heirs of the Astrarchy” kicker have been removed.

The comparison sheet and the 1440 × 900 / 1000 × 600 mocks remain here; the active art is `title-keyart-c-road-at-dawn-1600.jpg`, the only candidate still in `app/src/assets/art/`.

## Comparison sheet

![Three directions, each shown bare and with the current title layout](title-keyart-directions.png)

Separate comps: `title-mock-<direction>-1440x900.png` and `title-mock-<direction>-1000x600.png` in this directory.
