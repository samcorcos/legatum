# Turn timers must share resolution eligibility

Gemini’s review of PR #374 found that a connected eliminated observer could keep the clock running while every living human was disconnected. Expiry marked absent players ready even though resolution was impossible, leaving a zero countdown and immediately advancing on reconnect.

Timer eligibility now uses the same seat predicate as normal resolution, ignoring only readiness. Living humans still need a connected participant and their initial joins; when all humans are eliminated, connected observers pace AI turns. An expired timer never marks seats ready while participation prevents resolution.

Regression tests fail before and pass after the fix for an eliminated observer with a disconnected living player, and for all-eliminated observers when another eliminated seat has never joined. Rehosting was already safe for the initial-join barrier (`Host::new` marks resumed seats joined); a preservation test verifies that behavior too. No core rules, save format or UI changes.
