# Ancient unlock art samples

Three painted treatments for the same five Ancient-era unlocks. Each row's base file is the master; A and B derivatives are 256×256 and 128×128 transparent PNGs, and C derivatives are 300×200 and 180×120 opaque PNGs. Each image has a neighboring `.prompt.txt` with the exact generation prompt. `contact.png` compares the sets at 192 px wide on parchment.

| Direction | Subject | Files |
| --- | --- | --- |
| A miniature | Workshop | `a-workshop*` |
| A miniature | Walls | `a-walls*` |
| A miniature | Market | `a-market*` |
| A miniature | Library | `a-library*` |
| A miniature | Irrigation | `a-irrigation*` |
| B roundel | Workshop | `b-workshop*` |
| B roundel | Walls | `b-walls*` |
| B roundel | Market | `b-market*` |
| B roundel | Library | `b-library*` |
| B roundel | Irrigation | `b-irrigation*` |
| C scene plate | Workshop | `c-workshop*` |
| C scene plate | Walls | `c-walls*` |
| C scene plate | Market | `c-market*` |
| C scene plate | Library | `c-library*` |
| C scene plate | Irrigation | `c-irrigation*` |

Transparent A/B masters had faint alpha specks and vivid red edge spill cleared before Lanczos downsampling; corners are fully transparent. C masters were already full-bleed and opaque.

Preparation fixes, one per transparent image:

- `a-workshop`: Cleared faint alpha specks and vivid red edge spill.
- `a-walls`: Cleared faint alpha specks and vivid red edge spill.
- `a-market`: Cleared faint alpha specks and vivid red edge spill.
- `a-library`: Cleared faint alpha specks and vivid red edge spill.
- `a-irrigation`: Cleared faint alpha specks and vivid red edge spill.
- `b-workshop`: Cleared faint alpha specks and vivid red edge spill beyond the roundel.
- `b-walls`: Cleared faint alpha specks and vivid red edge spill.
- `b-market`: Cleared faint alpha specks and vivid red edge spill.
- `b-library`: Cleared faint alpha specks and vivid red edge spill.
- `b-irrigation`: Cleared faint alpha specks and vivid red edge spill.

The five C scene plates needed no cleanup.
