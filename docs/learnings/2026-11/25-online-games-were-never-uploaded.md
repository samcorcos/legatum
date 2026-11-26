# Online games were never uploaded: the server refused their typed names (2026-11-25)

Of 1,394 uploaded games, none had two human seats. A single-player game uploads `public_archive`, the copy with every seat renamed to "Player N" / "AI N". An online game's host handed its histories to the same queue with the names players joined under, the upload server answered 422 ("seat names must be the defaults"), and the page treats 422 as final and drops the record. Nothing surfaced the loss: the only trace was a handful of 422s in the Worker's request counts.

The host now hands over renamed copies when the build uploads games (`BrowserHost::take_public_recordings`, `publicRecordings` in `app/src/net/hosting.ts`). Two things to keep in mind:

- A new path that files a history for upload must send the renamed copy. The server-side refusal protects privacy but hides the bug, so check the upload list for the new kind of game after shipping.
- A record that is refused as invalid disappears silently by design (retrying cannot help). When a class of games is missing from the data, look at the Worker's 4xx counts by status first.
