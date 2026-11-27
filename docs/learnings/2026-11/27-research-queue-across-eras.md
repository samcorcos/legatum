# The research queue can run across eras without loosening the era gate (2026-11-27)

The strict gate (research only your current era) had been enforced twice: on research itself and on the queue.
The second was not needed. A queue is researched in order, so a later-era technology placed behind every unknown
technology of the eras before it can only come up once its era is open. Rules 67 checks exactly that
(`eras::queue_gap`: the era a queued technology would have to wait behind, given what is known plus what is
ahead of it in the queue) and lets the queue survive an era-up. `SetResearch` keeps the old check.

Gotchas:

- `normalize_research` used to drop every queued technology outside the current era, which is what emptied the
  queue on an era-up. It now cuts the queue at the first technology that breaks the gate instead, so any client
  bug that sends a bad order still ends with a legal queue.
- Every client edit of the queue must keep it legal or the whole order is refused: removing a technology also
  removes the later-era ones that waited behind it (`eraResearch.ts::removeFromQueue`), and the queue strip
  disables a move that would put a later technology ahead of one it waits for.
- The AI queues only its current era, so its queue is still empty after an era-up and no AI decision changes.
- The tree's research goal (rules 60) became redundant with a cross-era queue; the interface stopped setting it
  but still shows (and can clear) a goal from an older game.
