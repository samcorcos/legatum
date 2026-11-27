# Modal dialogs escape the shell's `user-select: none`

2026-11-27. The game shell set `user-select: none` on `html, body, #root`, yet in Chrome shift+click, a drag or
select-all painted the whole research panel as a text selection. A `<dialog>` opened with `showModal()` sits in the
top layer, and Chrome resolves its `user-select` to `text` (computed style on `dialog.tech-modal` and its children),
so the shell's `none` never reaches it. Set `user-select: none` on `dialog` and `[popover]` as well; text fields and
the Guide opt back in with `text`. Safari also ignores unprefixed `user-select` and the Vite build does not autoprefix,
so write `-webkit-user-select` next to every `user-select`. Guarded by `app/e2e/text-selection.spec.ts`.
