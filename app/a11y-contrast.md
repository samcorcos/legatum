# Atlas accessibility contrast measurements

Baseline: `56ff53d`. Ratios use linearized sRGB relative luminance, `(lighter + 0.05) / (darker + 0.05)`. Values are rounded to two decimal places. Text threshold: 4.5:1; meaningful graphics and control boundaries: 3:1. Disabled controls are excluded.

All altered or introduced color tokens are covered below. The opaque panel colors also match the SVG frame fills. Decorative rule tokens and the eight player colors are unchanged.

| Look / token | Before | After | Surface / usage | Ratio before → after |
|---|---|---|---|---|
| Atlas `--muted` | `#8a7c65` | `#655844` | `--panel` | 3.40 → 5.78 |
| Atlas `--muted` | `#8a7c65` | `#655844` | `--tip-bg` | 3.62 → 6.15 |
| Atlas `--muted` | `#8a7c65` | `#655844` | `--panel-2` | 3.08 → 5.23 |
| Atlas `--muted` | `#8a7c65` | `#655844` | `--panel-3` | 2.72 → 4.62 |
| Atlas `--good` | `#4f7f33` | `#3d6427` | `--topbar-bg` | 3.97 → 5.75 |
| Atlas `--good` | `#4f7f33` | `#3d6427` | `--panel-2` | 3.59 → 5.20 |
| Atlas `--good` | `#4f7f33` | `#3d6427` | `--panel-3` | 3.18 → 4.60 |
| Atlas `--y-food` | `#5e8c3b` | `#4e762e` | `--topbar-bg` | 3.31 → 4.43 |
| Atlas `--y-food` | `#5e8c3b` | `#4e762e` | `--panel-2` | 3.00 → 4.01 |
| Atlas `--y-food` | `#5e8c3b` | `#4e762e` | `--panel-3` | 2.65 → 3.55 |
| Atlas `--y-production` | `#b8662a` | `#965020` | `--topbar-bg` | 3.53 → 5.06 |
| Atlas `--y-production` | `#b8662a` | `#965020` | `--panel-2` | 3.19 → 4.58 |
| Atlas `--y-production` | `#b8662a` | `#965020` | `--panel-3` | 2.82 → 4.05 |
| Atlas `--y-gold` | `#a8841c` | `#80620d` | `--topbar-bg` | 2.93 → 4.78 |
| Atlas `--y-gold` | `#a8841c` | `#80620d` | `--panel-2` | 2.65 → 4.32 |
| Atlas `--y-gold` | `#a8841c` | `#80620d` | `--panel-3` | 2.34 → 3.82 |
| Atlas `--y-science` | `#2c7fab` | `#236b92` | `--topbar-bg` | 3.71 → 4.89 |
| Atlas `--y-science` | `#2c7fab` | `#236b92` | `--panel-2` | 3.35 → 4.42 |
| Atlas `--y-science` | `#2c7fab` | `#236b92` | `--panel-3` | 2.96 → 3.91 |
| Atlas `--control-border` (new) | `--rule-2`: `#9d8c6c` | `#8a7759` | `--field` | 2.91 → 3.83 |
| Atlas `--control-border` (new) | `--rule-2`: `#9d8c6c` | `#8a7759` | `--panel` | 2.74 → 3.61 |

`--on-player: #08101f` supplies dark ink for Atlas population, military glyphs, and numbered player badges. `ON_PLAYER` in `theme.ts` supplies the same canvas ink.

| Player / background | Population before | Military glyph before | Both after (`--on-player`) |
|---|---|---|---|
| Azure `#4aa8ff` | 2.52 | 2.34 | 7.55 |
| Jade `#3fd6a4` | 1.85 | 1.71 | 10.29 |
| Crimson `#e5534b` | 3.70 | 3.43 | 5.14 |
| Amber `#f2a63b` | 2.04 | 1.89 | 9.32 |
| Violet `#a97ff0` | 3.01 | 2.79 | 6.32 |
| Rose `#e86fb8` | 2.84 | 2.63 | 6.70 |
| Lime `#b9d24a` | 1.70 | 1.57 | 11.21 |
| Steel `#d8dde3` | 1.37 | 1.27 | 13.92 |

The following token applications change without changing the underlying primary or base ink tokens. The active Buy override also fixes its hover background so that the foreground remains readable during pointer use.

| Usage | Before | After |
|---|---|---|
| Atlas active Buy text | 12.81 | 12.81 |
| Atlas active Buy icon (`--y-gold` overridden with `--primary-ink`) | 4.37 | 12.81 |
| Atlas player names on paper (player color → `--ink`) | 1.14–3.09 | 12.81 |


Verification limits: these are source-derived calculations for the listed opaque surfaces, not measurements of anti-aliased pixels. Text over variable terrain/artwork, native modal focus containment, and actual screen-reader announcements still need browser/device review. Chromium launch was blocked by the macOS execution sandbox (`MachPortRendezvousServer: Permission denied`). DOM interaction tests cover tooltip relationships/dismissal, research focus return, faction navigation, report expansion, city selection, and separate seat readiness; they do not replace that browser review.
