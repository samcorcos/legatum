# Third-party notices

Legatum's own code is MIT (`LICENSE`) and its art is CC BY 4.0 (`LICENSE-ART`). It builds on the open-source
components below, each under its own licence. Last audited 2026-11-21 at commit 6641660 with `cargo metadata` on both
shipped Cargo workspaces and the licence fields of `app/package-lock.json`. No dependency is GPL, LGPL-only or AGPL,
none is unlicensed, and none comes from git or from a path outside this repository.

Where a component offers a choice of licences, Legatum uses it under MIT or, where MIT is not offered, Apache-2.0.

Distributed builds (the web build and the Mac app) carry the full licence texts in `THIRD_PARTY_LICENSES.txt`, written by
`tools/release/third-party-licenses.mjs` from the locked dependencies and shown in the game under How to Play, Credits.

## Rust crates

Three lock files: the engine workspace (`Cargo.lock`, 195 third-party crates: core, AI, simulation runner, WASM
bindings, LAN server), the desktop shell (`app/src-tauri/Cargo.lock`, 440 third-party crates, Tauri) and a
development-only probe (`tools/analysis/Cargo.lock`, a subset of the engine's dependencies, never distributed).
Together the first two name 487 distinct crate versions. Regenerate the per-crate list with `cargo deny list` or
`cargo metadata --format-version 1` in the root and in `app/src-tauri`.

| Licence | Crates | What it asks of a binary distribution |
| --- | --- | --- |
| `MIT OR Apache-2.0` (293), MIT (110), `MIT OR Apache-2.0 OR Zlib` (20), `MIT OR Unlicense` (11), and other expressions that offer MIT or Apache-2.0 | most crates, including `tauri`, `wry`, `serde`, `tokio`, `axum`, `rayon`, `wasm-bindgen`, `flate2`, `mimalloc` (which builds the MIT-licensed mimalloc C library) | Keep the copyright and permission notice. |
| Apache-2.0 only | `tao`, `sync_wrapper` | Include the Apache-2.0 text. Neither crate ships a NOTICE file. |
| `Apache-2.0 AND MIT` | `dpi` | Both texts. |
| BSD-3-Clause, alone or with MIT | `zstd-sys` and `zstd-safe` (which build the zstd C library; zstd is BSD-3-Clause or GPL-2.0, used here under BSD-3-Clause), `brotli`, `alloc-no-stdlib`, `alloc-stdlib`, `matchit` | Keep the copyright notice, conditions and disclaimer. |
| Unicode-3.0 | `icu_collections`, `icu_locale_core`, `icu_normalizer`, `icu_normalizer_data`, `icu_properties`, `icu_properties_data`, `icu_provider`, `litemap`, `potential_utf`, `tinystr`, `writeable`, `yoke`, `yoke-derive`, `zerofrom`, `zerofrom-derive`, `zerotrie`, `zerovec`, `zerovec-derive`; `unicode-ident` carries Unicode data under the same licence | Keep the Unicode copyright and permission notice. |
| Zlib | `foldhash` (linked into the desktop app), `zlib-rs` (not linked on the targets we build) | None for binaries; do not misrepresent the origin. |
| MPL-2.0 | `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`. All five are linked into the desktop app through Tauri (`tauri-utils` → `dom_query`, and `dirs`). They are not in the engine workspace. | File-level copyleft. The crates are used unmodified. Their source is available from crates.io at the versions pinned in `app/src-tauri/Cargo.lock`. If you modify them, publish those files under MPL-2.0. |
| ISC | `libloading` 0.7.4 (Windows and Linux builds of the shell only) | Keep the copyright notice. |
| `Apache-2.0 WITH LLVM-exception` | `target-lexicon` (build-time only) | Not distributed. |
| `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | `r-efi` (UEFI targets only, never built here) | Used under MIT or Apache-2.0; the LGPL option does not apply. |

The Rust standard library, linked into every binary, is `MIT OR Apache-2.0`.

A Linux build of the desktop shell links dynamically to the system's GTK 3, WebKitGTK and libsoup, which are LGPL.
They are not bundled. macOS and iOS builds use the system WebKit.

Thirty-seven crate tarballs (among them the `objc2-*` framework crates, `selectors`, `alloc-stdlib`,
`webview2-com*`, `ndk*` and `r-efi`) carry no licence file of their own, only the SPDX expression in `Cargo.toml`.
For those, use the standard text of the named licence and the authors in the crate's manifest or repository.

## JavaScript packages

`app/package-lock.json` names 116 packages: 12 for production and 104 development-only.

Shipped in the app:

| Package | Licence | Copyright |
| --- | --- | --- |
| `react`, `react-dom`, `scheduler` | MIT | Meta Platforms, Inc. and affiliates |
| `zustand` | MIT | 2019 Paul Henschel |
| `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-fs`, `@tauri-apps/plugin-haptics` | `MIT OR Apache-2.0` (used under MIT) | 2017 - Present Tauri Apps Contributors |
| `@fontsource/cormorant` (the Cormorant typeface) | SIL Open Font License 1.1 | 2015 The Cormorant Project Authors (github.com/CatharsisFonts/Cormorant) |
| `@fontsource/libre-franklin` (the Libre Franklin typeface) | SIL Open Font License 1.1 | 2020 The Libre Franklin Project Authors (github.com/googlefonts/Libre-Franklin) |

`@types/react` and `csstype` (MIT) are in the production set of the lock file but contain only type declarations, so
nothing from them reaches the built app. The WebAssembly engine's JavaScript glue is generated by `wasm-bindgen`
(`MIT OR Apache-2.0`).

The fonts are installed from npm and their Latin `.woff2` files are bundled into the built app; they are not stored
in this repository. The OFL requires the copyright notice and licence text to travel with the font files. The npm
packages include them (`LICENSE` in each package); a distributed build must carry them too (see "Notices in
distributed builds").

Development-only tools (Vite, Vitest, Playwright, TypeScript, Testing Library, happy-dom, fake-indexeddb, the Tauri
CLI and their dependencies) are not distributed with the game. Their licences are MIT (75 packages in all, including
the shipped ones), `MIT OR Apache-2.0` (16), MPL-2.0 (12), Apache-2.0 (8), OFL-1.1 (2, the fonts above),
BSD-2-Clause (1), BSD-3-Clause (1) and ISC (1). The MPL-2.0 packages are `lightningcss` and its eleven platform
binaries, pulled in by Vite, unmodified and build-time only.

## Python and other tooling

Scripts under `tools/` and `scripts/` are development tooling and are not part of the game. Some import NumPy
(BSD-3-Clause), PyTorch (BSD-3-Clause), Pillow (MIT-CMU) and boto3 (Apache-2.0), which you install yourself; none is
vendored here. The build also calls `wasm-pack` and `wasm-opt` (Binaryen), and the audio and film scripts call
`ffmpeg`; none of them is redistributed.

No third-party source code is vendored or copied into this repository. `CODE_OF_CONDUCT.md` is adapted from the
Contributor Covenant 2.1 (CC BY 4.0), as it says.

## Audio

All shipped music and sound effects (`app/public/audio/`, 38 files) are released under CC0 1.0 (no attribution
required; courtesy credit is given). The per-file source, author and processing are in `app/public/audio/CREDITS.md`
and `app/public/audio/manifest.json`: Kenney (kenney.nl: Music Jingles, RPG Audio, Impact Sounds; 34 files),
Breviceps ("Short Choir", freesound.org; 2 files) and vitalezzz ("Eclipse Legion", opengameart.org; both music
tracks are edits of that one work). The licence text is in `app/public/audio/licenses/CC0-1.0.txt`. The same credits
appear in the game under How to Play → Credits.

## Art

All of the art is AI-generated and is licensed as described in `LICENSE-ART`. No third-party artwork is included. Real
landmarks and historical peoples are shown in generated illustrations; no photographs are used.

## Notices in distributed builds

MIT, Apache-2.0, BSD, ISC, Unicode-3.0 and the OFL all require their notices to accompany a binary. A distributed
build of Legatum (the macOS app, an iOS build, or `legatum-server` with the web UI) must therefore include this file
and the full licence texts of the linked crates and bundled packages, for example a `THIRD_PARTY_LICENSES` file
generated with `cargo about` and `license-checker --production` and shipped as a Tauri bundle resource and linked
from the in-game Credits.

## Trademarks

Legatum is an independent project and is not affiliated with, endorsed by or derived from Sid Meier's Civilization, Take-Two Interactive Software, Inc. or Firaxis Games; "Civilization" is their trademark and is named only to describe the genre.

Tauri, Rust, React, OpenAI, ChatGPT, Codex, GPT, Claude and Anthropic are names or trademarks of their owners, mentioned only
to describe the tools used; no endorsement is implied.
