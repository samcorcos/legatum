# Research queue advice needs its own continuation

`pendingDecisions` stops listing research once the player queues one technology. An advisor that recommends several technologies must follow a matching pending queue directly, then leave the player alone after the full queue is set. `ResearchQueueAdvanced` with `next: null` identifies a queue that ran dry after a technology finished; an empty queue alone can also mean the player cleared it.
