# A refused build looked chosen while Choose production stayed (2026-11-27)

Order validation is cumulative: `rules::validate_orders` checks city orders in list order against a projected state
(`validate_city_orders_projected`), so the last spare strategic resource goes to the first build that needs it. The
city panel's options (`resource_blocker`) come from the turn-start view and do not see this turn's other orders, so a
later city can still pick a Musketman that the engine then refuses ("No Niter to spare").

`pendingDecisions` drops refused orders before testing for idle cities, which is right (the city really will build
nothing), but the CTA only said "Choose production · <city>", the panel still showed the pick under Now producing,
and the refusal text sat at the foot of the city panel, below the fold. A player read it as the game ignoring the
pick and could not find a way forward.

Fix: a refused production order now names itself in the decision label and the CTA sub ("Musketman can't start:
No Niter to spare") and shows directly under Now producing in the city panel. Found by replaying an uploaded co-op
tape to turn 109 and validating the seat's recorded orders: three Niter builds after the first were refused.

Possible follow-up: make the panel's option blockers project this turn's earlier orders so the pick is greyed out
before it is made.
