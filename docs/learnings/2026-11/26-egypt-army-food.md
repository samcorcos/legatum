## 2026-11-26: Egypt’s army Food cost

- Cause: the Nile’s doubled Food supported an oversized army without drawing on the yield that builds Egypt’s cities.
- Evidence: the default-rule 300-game Egypt ladder measured 25.7% at seed 5001 (the earlier sim-only experiment measured 26.0%, versus 47.3% before the cost). The 2,000-game eleven-faction Hard census measured Egypt at 16.3% per seat and best/worst at 4.47×, still above the 1.5× target. Results are in `docs/sim-results/`.
- Change: every eligible Egyptian land military unit consumes 3 Food, shared by cities after doubling. The rule is independently ablatable with `egypt_army_food`; the Gold-buy, wartime-Food, and era-scaled alternatives were retired.
