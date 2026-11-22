# Security policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub's
[private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
("Security" tab, "Report a vulnerability") rather than a public issue. You will get an acknowledgement
within a few days. This is a hobby project without a bug bounty.

## Scope

Legatum is a static web app (served by Cloudflare Pages) with the Rust engine compiled to WebAssembly. It has no server program and installs nothing. The network surfaces are the online relay and the game upload Worker:

- Online play (`plans/online-play.md`, `infra/relay/README.md`): the game runs in the host's browser tab, which dials out over wss to a Cloudflare Worker that lists games and copies messages. The lobby is public: anyone can list rooms and open guest sockets, and the relay cannot tell a seat token from other text, so seat tokens and game messages stay between guest and host. The relay enforces its own limits, with no dashboard setup: 64 rooms in all; per client address (an IPv6 address counts as its /64 network), 2 live rooms, 6 room creations per 10 minutes and 4 guest sockets per room; 8 guests per room; message size caps; and per-socket message and byte rate caps that close a flooding guest or host tunnel (code 4029). Client addresses are kept only as salted hashes, for as long as the room, socket or 10-minute rate window lasts, and are never logged. Room names are reduced to letters, digits, spaces and a little punctuation, at most 40 characters, with a blocklist hook. A host that sends keepalive frames and then goes silent for 60 s leaves the list, and `LOBBY_OPEN=0` stops new rooms at once while running games carry on. Filling the lobby now takes many addresses; if that happens, add a Cloudflare rate-limiting rule or a shared secret on room creation.
- Game uploads (`infra/games/README.md`): the hosted web build sends each game's record, with every seat under its default name, to a Cloudflare Worker that stores it in a private R2 bucket. The Worker accepts only allow-listed page origins, caps bodies (4 MiB gzipped, 64 MiB unpacked), refuses any other seat name and keeps no IP address or request log. Anyone who can script a request from an allowed origin can store junk up to those caps; that is an accepted risk.

## Data kept on your computer

The game keeps data only in your browser: autosaves and game histories (IndexedDB and local storage) and the files you choose to download (saves, exported game histories, scenario runs). Clearing site data removes them. Nothing is written elsewhere on your computer.

In scope: the relay and upload Workers' request handling, the host's handling of guest messages, and save and scenario loading (malformed files must fail cleanly and never overwrite a good save). Out of scope: attacks that need control of the player's browser or machine.
