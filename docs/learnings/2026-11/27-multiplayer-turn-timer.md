# Multiplayer turn timer

The browser host owns the turn clock; the relay only transports messages. The staging setting is host metadata, not a deterministic core rule. Missing/zero means unlimited. Timed games wait for the initial living-seat joins, then resolve with staged orders at expiry, using the same path as Ready. All-ready turns still end early. A suspended host advances at most one turn on waking; it does not replay missed wall-clock turns.

The chosen duration is stored in the online save as `online_turn_timer_seconds`, outside core state. Reloading or rehosting starts a fresh planning interval. When no living human is connected, the clock is disarmed; if every human is eliminated, connected observers can pace AI turns. Countdown snapshots come from the host in seats messages, so clients need no synchronized clocks. Missing countdown fields clear the UI for older hosts and between turns.
