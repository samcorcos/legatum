# "Every Way to Score", round 3: the score ledger

Approved design, 2026-10-24. The production panel is `app/src/ui/ScoreLedger.tsx` with `scoreLedger.css`; it opens from the VP chip or “All objectives” in the victory race. The original mockup shots below use the user's turn-203 save (8 empires, Modern era scored, 7 of 10 VP). The release tour in `app/e2e/tour/score-ledger.spec.ts` covers two and eight empires, teams, a hold countdown, a captured Heir goal, and a crown tie at 1000x600 and 1440x900.

| Shot | 1000x600 | 1440x900 |
| --- | --- | --- |
| The ledger as it opens | `ledger-1000-default.png` | `ledger-1440-default.png` (the era in focus opens by itself on a tall window) |
| A crown expanded | `ledger-1000-crown.png` | `ledger-1440-crown.png` |
| The faction goals expanded | `ledger-1000-goals.png` | `ledger-1440-goals.png` |
| A scored era expanded | `ledger-1000-era.png` | `ledger-1440-era.png` |

## Why the current sheet confuses

The round-2 sheet is five surfaces on one page: a race list with socket tracks and goal slips, three crown cards with bars, a decadence paragraph and bar list, a Great Work card, and one table per era, each with its own legend words (live, kept, entered, scored with this, nobody yet). The same fact appears in two or three encodings (a point is a socket in the race list, a starred row in a crown card and a marker on an era table), so the eye has to learn each block before it can read it. Nothing on the page is organised around what the player came to ask.

## What a player asks, and where the answer is

| Question | Answer on the ledger |
| --- | --- |
| Who's winning? | The columns are the empires, best first, each with its VP, a 10-pip track and the rank numeral. A hold-to-win countdown sits under the holder's VP. |
| How close am I? | Your column is gilt wherever it falls, so the gap to the leader is the distance between two columns. |
| What's my next point, and how do I get it? | Every way to score is a row. Your cell says held (a gilt token), open (a dashed socket) or not reached (a faint socket), with your private progress where only you can see it (the goal meter). Expanding a row shows its rule and, for an era, its three objectives with your progress. The ledger ranks nothing and recommends nothing (user, 2026-10-06). |
| What can I take from the leader? | The three crown rows show every empire's number beside the holder's, with a bar scaled to the top. Expanding one lists them all, best first. |

Pillar 4 (the race is always visible): the whole race is the column head, and every point of it is a filled cell below. Pillar 5 (no hidden math): the columns add up on the page (3 crowns + 2 + 2 + 6 eras); each cell carries its number or a tooltip with the fact behind it; the detail row gives the rule in full. Pillar 9: the one primary is Close; every row looks clickable (chevron, hover tint) and expands in place.

## What was cut or folded

- **The legend.** Three states are told by three marks that look like what they mean: a filled token in the owner's ink (held), a dashed socket (open), a faint socket (not reached). Live versus kept is the section the row sits in ("Crowns: held by the leader, lost with the lead"; "Kept for good"), not a legend entry.
- **The race list with socket tracks.** The columns are the race; the cells are the sockets.
- **The decadence paragraph and bar list.** Decadence is not a way to score. It is one red word under the VP of each empire it applies to ("decadence −21%"), with the full sentence in its tooltip. Its breakdowns stay where they were (city yields, the Gold and Science hovers).
- **One era table per era, each with its own notes.** Eras are six one-line rows in one section; the counting rule ("counts only what you gain after entering the era and after your last era score") is said once, in the expanded row.
- **Repeated rule text.** The rule under each crown and the Great Work is one line. The faction goal row says "Each empire has its own"; the names sit in the cells and the eight rules in the expanded row.
- **The jargon words** live, kept, entered, scored with this. The words left are held, open, not reached, scored, done, not yet.

## Visual language

From the faction sheet and the diplomacy inbox: Cormorant for names and numbers of rank, Libre Franklin for figures and rules, tracked small-caps section heads with the accent icon and a fading hairline, portrait roundels with the crest pinned, gilt reserved for yours, the empire's ink (never its raw seat colour) for every mark on parchment, and paper grain over the whole dialog. The tokens are small control markers in the TI4 sense: an empire's colour on the objective it scored.

## Fit

1440x900: the whole ledger and the open era in view, no scrolling. 1000x600: every row opens closed, and the last era row fits without scrolling. The duplicate-faction head name abbreviates “Switzerland” to “Swiss” while its full name stays in the title tooltip. The column heads stay pinned when an expanded row needs scrolling.

## Decisions after approval

1. **Columns stay in rank order, with your column gilt wherever it lands.**
2. **A short window opens with every row closed; a tall window opens the era in focus.**
3. **Teams** (two humans, 20 VP): a team's columns stay together, with team VP over its first member.

## Rules the design wants but does not change

None. Everything shown comes from the existing `buildVpModel`; the next era's objectives are visible because the view publishes them today (the 2026-10-22 decision to reveal them only on entry is not yet built; the ledger will follow the data).

## Shipped implementation

The ledger replaced the card body. The dialog shell, keyboard controls and VP chip remain. Cells carry numbers or explanatory tooltips. The release tour records the listed states, and the old sheet-only CSS and tests were retired.
