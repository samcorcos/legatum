# Safari ignores unprefixed `user-select`

2026-11-27. The game shell set `user-select: none` on `html, body, #root`, which Chromium honours (shift+click,
double-click and select-all highlight nothing), but Safari still only reads `-webkit-user-select`, so whole panels such
as the research screen lit up as a text selection. The Vite build does not autoprefix CSS. Write both properties
wherever `user-select` is set, and give `input`, `textarea` and `[contenteditable]` an explicit `text` value so typing
fields stay selectable under the shell-wide `none`.
