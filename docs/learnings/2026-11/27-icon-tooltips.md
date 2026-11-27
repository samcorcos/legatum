# Icon-only labels need the game's tooltip, not a browser title (2026-11-27)

The rivals table's column icons (VP, Gold, Army points, Cities, Science), its chips and the Score ledger's tokens
explained themselves only through the HTML `title` attribute. Players did not find those: the browser shows them
late, small and unstyled, and never on touch. They now use `Tip` with `hoverOnly`, which shows the game's own
bubble on hover but takes no focus and no click, so it can sit inside a row that is itself a button (a rival's row
opens Diplomacy). Two gotchas:

- A `Tip` normally steers its bubble clear of its own panel and of `.seatbar`, the whole right sidebar. For a short
  label that pushed the bubble far below the icon (over the minimap). `hoverOnly` tips (`near` in `Bubble`) ignore
  any box that contains the trigger, so the bubble sits next to the icon.
- A native `title` on an ancestor still shows over a child's `Tip`. The rival row's button title (AI level, "Open
  Diplomacy") moved onto the rival's name as a `hoverOnly` tip.

Tests read hover text from the bubble the trigger's `aria-describedby` names (`tipText` in `RivalsTable.test.tsx`).
