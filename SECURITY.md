# Security policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub's
[private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
("Security" tab, "Report a vulnerability") rather than a public issue. You will get an acknowledgement
within a few days. This is a hobby project without a bug bounty.

## Scope

Legatum is a static web app (served by Cloudflare Pages) with the Rust engine compiled to WebAssembly. It has no server program and installs nothing. The network surfaces are the online relay and the game upload Worker:

- Online play (`plans/online-play.md`, `infra/relay/README.md`): the game runs in the host's browser tab, which dials out over wss to a Cloudflare Worker that lists games and copies messages. The lobby is public for now: anyone can list rooms and open guest sockets, and the relay cannot tell a seat token from other text, so seat tokens and game messages stay between guest and host. Limits bound the cost of a stranger (64 rooms, 8 guests per room, message size caps). Anyone can fill the lobby with rooms; that is an accepted risk.
- Game uploads (`infra/games/README.md`): the hosted web build sends each game's record, with every seat under its default name, to a Cloudflare Worker that stores it in a private R2 bucket. The Worker accepts only allow-listed page origins, caps bodies (4 MiB gzipped, 64 MiB unpacked), refuses any other seat name and keeps no IP address or request log. Anyone who can script a request from an allowed origin can store junk up to those caps; that is an accepted risk.

## Data kept on your computer

The game keeps data only in your browser: autosaves and game histories (IndexedDB and local storage) and the files you choose to download (saves, exported game histories, scenario runs). Clearing site data removes them. Nothing is written elsewhere on your computer.

In scope: the relay and upload Workers' request handling, the host's handling of guest messages, and save and scenario loading (malformed files must fail cleanly and never overwrite a good save). Out of scope: attacks that need control of the player's browser or machine.
