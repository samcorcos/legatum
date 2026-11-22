# Security policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub's
[private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
("Security" tab, "Report a vulnerability") rather than a public issue. You will get an acknowledgement
within a few days. This is a hobby project without a bug bounty.

## Scope

Legatum is a single-machine game. The network surfaces are the optional LAN host,
`crates/server`, which lets players on the same network join a game in a browser, and the online relay described below:

- It listens on all interfaces over plain HTTP and WebSocket. Seat tokens travel in the clear, so run it
  only on networks you trust, never on public Wi-Fi or the open internet.
- Requests are checked against a host and origin allowlist and a per-seat token; scenario uploads are
  capped (2 MiB per request, 500 files, 100 MiB in total).
- Online play (`plans/online-play.md`, `infra/relay/README.md`): the desktop app binds its server to `127.0.0.1` only and dials out over wss to a Cloudflare Worker that lists games and copies messages. The lobby is public for now: anyone can list rooms and open guest sockets, and the relay cannot tell a seat token from other text, so seat tokens and game messages stay between guest and host. Limits bound the cost of a stranger (64 rooms, 8 guests per room, message size caps). Anyone can fill the lobby with rooms; that is an accepted risk. Relayed guests share one admission peer, so the per-IP connection limits apply to them together.
- Game uploads (`infra/games/README.md`): the hosted web build sends each game's record, with every seat under its default name, to a Cloudflare Worker that stores it in a private R2 bucket. The Worker accepts only allow-listed page origins, caps bodies (4 MiB gzipped, 64 MiB unpacked), refuses any other seat name and keeps no IP address or request log. Anyone who can script a request from an allowed origin can store junk up to those caps; that is an accepted risk.
- Saves are plain files in the saves directory (default `saves/` in the platform data folder:
  `~/Library/Application Support/Legatum/` on macOS, `~/.local/share/legatum/` on Linux,
  `%APPDATA%\Legatum\` on Windows). Its replies never contain the host's file paths.

## Files written on your computer

The desktop app and the LAN host write only under your home folder:

- saves and the autosave copy in the app's data folder (`com.legatum.game` for the desktop app, the
  saves directory above for the LAN host);
- a local, compressed archive of finished games in `~/LegatumData/archive/` (`%APPDATA%\Legatum\archive\`
  where `HOME` is unset), which you can move with `--archive-dir` or `LEGATUM_ARCHIVE`. If it cannot be
  opened, the game runs without it;
- for the desktop app, scenario runs in `~/LegatumData/scenarios/` and `~/LegatumData/app-info.json`.

In scope: the server's request handling, save and scenario loading (malformed files must fail cleanly
and never overwrite a good save), and the desktop app's file access. Out of scope: attacks that need
control of the host machine or its network.

The desktop app is unsigned. On macOS, Gatekeeper blocks it on first open: right-click and choose
Open, or run `xattr -dr com.apple.quarantine Legatum.app` on a build you made yourself.
