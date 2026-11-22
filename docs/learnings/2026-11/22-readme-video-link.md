## 2026-11-22: The README poster linked to a page that cannot play the video

- **A README link to a repository's own mp4 opens GitHub's file page, and for a file over about 5 MB that page says "(Sorry about that, but we can't show files this big right now.)" instead of playing it** (user, 2026-11-22, on the 7.2 MB teaser). The poster shipped with that link because the check was "the image looks sharp", not "click it".
- **Now:** `scripts/build-web.sh` copies `docs/media/legatum-teaser.mp4` to `media/legatum-teaser.mp4` on the site, and the README poster links to `https://playlegatum.com/media/legatum-teaser.mp4`, which the browser plays directly. The copy happens after `keep-builds`, so the 7 MB is not repeated under every `v/<id>/`.
- **Check a link by following it**, in a browser, signed out, before calling a README change done.
