# Return on a held moment never clicks the plate's own button

2026-11-27. A held moment card (era-up, unlock, first contact) answers Return in `MomentLane`'s key handler, which calls
`dismiss()` and then the card's own primary action (`activateUnlock`, `activateEraUp`, `showMeetingPlace`). It does not
click the plate's Continue button. Anything a primary button does besides closing must be wired in both places, or
Return and a click behave differently. The first-contact plate's map jump lives in one exported function
(`MeetingTakeover.tsx::showMeetingPlace`) that both call.
