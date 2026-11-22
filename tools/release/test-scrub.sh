#!/usr/bin/env bash
# Tests for tools/release/scrub.py and the rule that no value to hide is written in the tracked tree.
# Usage: tools/release/test-scrub.sh   (a few seconds; needs git and python3)
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
scrub="$here/scrub.py"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
fail=0
ok() { echo "ok: $1"; }
bad() { echo "FAIL: $1" >&2; fail=1; }

# 1. Every class has a sample, and the sample is caught in an ordinary document.
mkdir -p "$work/tree/docs"
for cls in $(python3 -I "$scrub" classes); do
  if ! python3 -I "$scrub" sample "$cls" > "$work/tree/docs/$cls.md"; then bad "class $cls has no sample"; continue; fi
  out="$(LEGATUM_SCRUB_PATTERNS=/nonexistent python3 -I "$scrub" scan "$work/tree" 2>/dev/null)"
  if grep -q "^docs/$cls.md:1: \[$cls\]" <<<"$out"; then ok "sample for [$cls] is caught"; else bad "sample for [$cls] is not caught: $out"; fi
  rm -f "$work/tree/docs/$cls.md"
done

# 2. An email address is caught unless it is on the allow-list.
printf 'write to jane.doe@%s\nwrite to you@example.com or noreply@%s\n' "mail-provider.com" "mail-provider.com" > "$work/tree/docs/mail.md"
out="$(python3 -I "$scrub" scan "$work/tree" 2>/dev/null)"
if [ "$(grep -c '\[email\]' <<<"$out")" = 1 ] && grep -q '^docs/mail.md:1: ' <<<"$out"; then ok "email outside the allow-list is caught, allowed ones pass"
else bad "email check: $out"; fi
rm -f "$work/tree/docs/mail.md"

# 3. allow-path: a made-up private address is fine in a test, not in a document.
mkdir -p "$work/tree/crates/server/tests"
python3 -I "$scrub" sample private-ip | tee "$work/tree/crates/server/tests/peer.rs" > "$work/tree/docs/net.md"
out="$(python3 -I "$scrub" scan "$work/tree" 2>/dev/null)"
if grep -q '^docs/net.md:1: \[private-ip\]' <<<"$out" && ! grep -q 'crates/server/tests' <<<"$out"; then ok "allow-path exempts test folders only"
else bad "allow-path: $out"; fi
rm -rf "$work/tree/crates" "$work/tree/docs/net.md"

# 4. The private file adds patterns; without it the same text passes. The literal here is invented for the test.
echo 'the box is called zebra-quartz-41' > "$work/tree/docs/ops.md"
printf '# invented for the test\nzebra-quartz-[0-9]+\n' > "$work/private.txt"
if LEGATUM_SCRUB_PATTERNS=/nonexistent python3 -I "$scrub" scan "$work/tree" >/dev/null 2>&1; then
  # A real private file in this checkout may still be found; it cannot match the invented literal.
  ok "text passes without the private pattern"
else bad "invented literal was flagged without its pattern"; fi
out="$(LEGATUM_SCRUB_PATTERNS="$work/private.txt" python3 -I "$scrub" scan "$work/tree" 2>/dev/null)"; rc=$?
if [ "$rc" = 1 ] && grep -q '^docs/ops.md:1: \[private\]' <<<"$out"; then ok "\$LEGATUM_SCRUB_PATTERNS adds owner patterns"
else bad "private pattern not applied (exit $rc): $out"; fi
if LEGATUM_SCRUB_PATTERNS="$work/private.txt" python3 -I "$scrub" pattern | grep -q 'zebra-quartz'; then ok "combined pattern includes the private ones"
else bad "combined pattern lacks the private ones"; fi
printf '([unclosed\n' > "$work/bad.txt"
if LEGATUM_SCRUB_PATTERNS="$work/bad.txt" python3 -I "$scrub" scan "$work/tree" >/dev/null 2>&1; then bad "a bad private regex was accepted"
else ok "a bad private regex is an error"; fi

# 5. The pattern files are not scanned with themselves, and the private folder is git-ignored.
mkdir -p "$work/tree/tools/release" "$work/tree/.private.example"
cp "$here/scrub-patterns.txt" "$work/tree/tools/release/"; cp "$repo/.private.example/scrub-patterns.txt" "$work/tree/.private.example/"
rm -f "$work/tree/docs/ops.md"
if python3 -I "$scrub" scan "$work/tree" >/dev/null 2>&1; then ok "pattern files do not flag themselves"; else bad "pattern files flag themselves"; fi
if git -C "$repo" check-ignore -q .private/scrub-patterns.txt; then ok ".private/ is git-ignored"; else bad ".private/ is not git-ignored"; fi
if [ -z "$(git -C "$repo" ls-files .private)" ]; then ok "nothing under .private/ is tracked"; else bad "files under .private/ are tracked"; fi

# 6. The exporter and the gate carry no inline pattern list: everything comes from the two pattern files.
if grep -qE "^[[:space:]]*pat='" "$here/export-public.sh"; then bad "export-public.sh has an inline pattern list again"
else ok "export-public.sh has no inline pattern list"; fi

# 7. The tracked tree has no hit for any generic pattern (or for the private ones, when the file exists here).
out="$(python3 -I "$scrub" scan --tracked 2>/dev/null)"; rc=$?
if [ "$rc" = 0 ]; then ok "tracked tree is clean"
else bad "tracked tree has $(grep -c . <<<"$out") personal-data hit(s):"; head -20 <<<"$out" | cut -c1-200 >&2; fi

exit "$fail"
