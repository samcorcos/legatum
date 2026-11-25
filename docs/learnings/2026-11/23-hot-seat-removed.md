# Hot seat removed (2026-11-23)

Multiplayer now uses the online lobby: the host resolves the shared turn and each browser controls one human seat. The local pass screen, local seat switching, and local team pings were removed. Builds without a relay disable Multiplayer with a short reason.

Old saves with more than one human seat remain valid engine saves. Load Game and Continue take them straight to Multiplayer with that save ready to host ("Host this save"; `TitleScreen.tsx` `onHostSave`, 2026-11-25), or, in a build with no relay, leave Continue disabled with the reason. The store's `loadSave` still refuses them without replacing the current game. Guests then rejoin their seats. The archive source value `human-hotseat` stays readable for old recordings; online hosting continues to use the historical `lan` value.
