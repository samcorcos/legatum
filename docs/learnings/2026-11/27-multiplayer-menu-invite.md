# Multiplayer menu and the host's invitation

Date: 2026-11-27

**What happened.** It was hard to tell which game in the list was yours. The game room named the listed game in one
small italic line under the role line ("In the game list, this game is ..."). The Multiplayer menu's Host and Join
doors were also unbalanced: Host held a code field, Host Game and a "Host from a save" button in tracked capitals,
while Join held one button above empty space.

**What changed.**
- The game room opens an invite band above the seats, so it shows at every window size. It shows the listed name in large type,
  the private code when the game is locked, and a Copy invite link button. The link is `?join=<room>` on this build's
  own versioned URL (`buildUrl(BUILD_ID, …)`), so a friend lands on the same version. It never carries the code. If
  the clipboard is refused, the band shows the link in a read-only box, already selected.
- The Multiplayer menu's doors are two framed cards of equal height. The Host door holds the private code field (its
  hint becomes the error when the code is too short; Enter presses the door's main button), Host Game, and "Host a
  saved game…" as a plain underlined link. The Join door lists the first two hosted games, each with its own
  Join button (`ui/OpenGamesList.tsx`, shared with the Hosted games screen), then Join Game ("See all N hosted games" when more than two are listed).
- Player text says "private code" everywhere: the Host door, the guest's box, the length error, the Guide and the
  README, and the engine's refusals (`crates/wasm/src/host/mod.rs`). A field inside the Host door never starts with "Join", so the old name "join code" had to go everywhere a player reads it.

**Why the names stay generated.** The list is public, and a typed name would give vandals a billboard. The relay
names each room from its id (`listedName`), and `net/roomName.ts` mirrors that rule so the host sees the same name.

**Gotcha.** `e2e/listedName.ts` now reads the listed name from `.invite-name`. Unit tests that mock `net/hosting`
need `hostedJoinCode` in `test/fakeHosting.ts`.
