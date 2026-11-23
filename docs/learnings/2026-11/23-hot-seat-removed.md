# Hot seat removed (2026-11-23)

Multiplayer now uses the online lobby: the host resolves the shared turn and each browser controls one human seat. The local pass screen, local seat switching, and local team pings were removed. Builds without a relay disable Multiplayer with a short reason.

Old saves with more than one human seat remain valid engine saves. The solo Load path refuses them before replacing the current game and directs players to Multiplayer > Host from a save. Guests then rejoin their seats. The archive source value `human-hotseat` stays readable for old recordings; online hosting continues to use the historical `lan` value.
