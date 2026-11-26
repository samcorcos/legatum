# r2.dev refuses urllib's default User-Agent

2026-11-26. `tools/art/fetch-sources.py` failed with HTTP 403 for every file in the public art-sources bucket. The r2.dev public URL answers 403 to the default `Python-urllib/3.x` User-Agent and 200 to any named client. `artsources.fetch()` now sends `User-Agent: legatum-art-fetch`. Any new script that downloads from r2.dev with urllib needs the same header.
