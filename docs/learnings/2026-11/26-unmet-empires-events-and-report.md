# Unmet empires in event cards and the turn report

Moment cards (`app/src/celebrate/moments.ts`), the turn report (`app/src/format.ts`) and the Share the Learning prompt named every empire with `empireOfPlayer`, so a Royal Tomb, a wonder race or a captured capital announced who did it before first contact. They now say "an unknown empire", as Civ does, and the card wears a dashed "?" (`Moment.unknown`) instead of the crest or the fallen empire's portrait.

The turn report needed no per-line change: its `ReportContext` (`contextFromViews`) is the one place that names players, so `empireName`, `teamName`, `empireOf` and `playerName` return `UNKNOWN_EMPIRE` for an unmet player, and `playerFaction` returns undefined so its units and buildings take their generic names. `line()` capitalises a line that starts with it.

What stays: a faction-only item still names itself (the Royal Tomb, the Heir), because the rules make those public with their tile; the card just doesn't say whose. Tests that check how met empires are named must meet everyone first (`app/src/test/meetEveryone.ts`): a fresh fixture game has met nobody.
