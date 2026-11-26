# A unit's upgrade stops at the most advanced form its supply allows (2026-11-26)

Until rules 62 an upgrade order always aimed at the newest researched unit of the line. A unit whose
newest form needed a resource the empire lacked could not upgrade at all, even when an older form on
the same line used a resource it had: a Phalangite with Iron but no Niter was offered only the
Musketman and stayed a Phalangite.

Rules 63 (`rules::units::supplied_upgrade_plan`) walks back from the newest researched form to the
first one whose supply is spare. Points worth knowing:

- Only the final form uses supply (`resources::unit_upgrade_need_to`), so the intermediate steps
  never need checking.
- Validation passes the count of supply earlier upgrades in the same order list took, and resolution
  re-reads live supply (`resources::in_use` counts units by kind), so both pick the same target when
  a list spends the last Niter.
- The view's upgrade fields all come from that one plan, so the AI (which only reads the view) takes
  the same path with no AI change. Expect Hard to upgrade more often where it lacks a later resource.
- Gold does not shorten the path. Validation's Gold count prices the target without the list's
  earlier supply, so in a list that runs out of Niter it can count more Gold than resolution
  charges (it never counts less).
- The app's queued-order supply check (`upgradeCheck`'s `reserved`) still treats the view's target
  as fixed. When queued upgrades use up a resource it shows the card blocked, while the engine would
  fall back. This is rare and errs towards caution.
