#!/usr/bin/env bash
# Tests for merge-gates.sh and tools/gates/ on a throwaway repository. Run: .claude/hooks/test-merge-gates.sh
# About 40 s: the two art cases render icons in headless Chromium (needs app/node_modules in this checkout).
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
HOOK="$HERE/merge-gates.sh"
SRC="$(cd "$HERE/../.." && pwd)"
T="$(mktemp -d "${TMPDIR:-/tmp}/merge-gates-test.XXXXXX")"
trap 'rm -rf "$T"' EXIT
R="$T/repo"
unset LEGATUM_GATE_BYPASS
export LEGATUM_ART_NODE_MODULES="$SRC/app/node_modules"
# The scenario gate's engine and copy lint are stubbed (the real ones are a cargo build and vitest; both are exercised
# by their own tests). The stub engine refuses a file that says BADHASH; the stub lint a briefing with "complete pairs".
mkdir -p "$T/stubs"
cat > "$T/stubs/engine" <<'STUB'
#!/usr/bin/env bash
rc=0; for f in "$@"; do if grep -q BADHASH "$f"; then printf 'FAIL\t%s\thash 1 does not match\n' "$f"; rc=1; else printf 'ok\t%s\n' "$f"; fi; done; exit $rc
STUB
cat > "$T/stubs/lint" <<'STUB'
#!/usr/bin/env bash
IFS=: read -ra fs <<< "$LEGATUM_LINT_SCENARIOS"
for f in "${fs[@]}"; do grep -q 'complete pairs' "$f" && { echo "+   \"$f | banned complete pairs\","; exit 1; }; done; exit 0
STUB
chmod +x "$T/stubs/engine" "$T/stubs/lint"
export LEGATUM_SCENARIO_ENGINE="$T/stubs/engine" LEGATUM_SCENARIO_LINT="$T/stubs/lint"
g() { git -C "$R" -c user.name=t -c user.email=t@t -c commit.gpgsign=false "$@" >/dev/null 2>&1; }
put() { mkdir -p "$R/$(dirname "$1")"; printf '%s\n' "$2" >> "$R/$1"; g add -- "$1"; }
commit() { put "$1" "$2"; g commit -q -m "$1"; }
branch() { g checkout -q -b "$1" master; }
ok() {  # ok <docs|ui|plain|art> <branch> [extra marker lines...]
  local kind="$1" b="$2"; shift 2
  mkdir -p "$R/.progress/$kind-review"
  { git -C "$R" rev-parse "$b"; printf '%s\n' "$@"; } > "$R/.progress/$kind-review/$(echo "$b" | tr / -).ok"
}

git init -q -b master "$R"
echo .progress/ >> "$R/.git/info/exclude"   # markers live outside git, as in the real checkout
put crates/core/src/lib.rs 'pub const RULES_VERSION: u32 = 5;'
put crates/core/src/state.rs '#[derive(Serialize)]
pub struct GameState {
    pub turn: u32,
}'
put crates/core/src/economy.rs 'pub fn upkeep(n: i32) -> i32 { n * 2 }'
put crates/core/Cargo.toml '[package]
name = "legatum-core"

[dependencies]
serde = "1"'
put crates/wasm/tests/fixtures/start.current.json '{"save_version":5,"turn":1}'
put crates/server/src/host.rs 'fn host() {}'
put db/migrations/001_init.sql 'create table t (id int);'
put docs/learnings.md base
put app/src/ui/Panel.tsx 'export const Panel = () => null;'
put app/src/data.ts 'export const TEXT = "Walls: +100 defence.";'
put app/package.json '{"name": "app", "private": true}'
mkdir -p "$R/tools/release" "$R/tools/art" "$R/app/src/assets/icons"
cp "$SRC/tools/release/export-public.sh" "$SRC/tools/release/scrub-patterns.txt" "$R/tools/release/"
cp "$SRC/tools/art/distinctness.py" "$SRC/tools/art/distinctness.json" "$SRC/tools/art/render-sets.mjs" "$R/tools/art/"
cp "$SRC/app/src/assets/icons/building-granary.svg" "$SRC/app/src/assets/icons/building-autoworks.svg" "$R/app/src/assets/icons/"
g add -A; g commit -q -m base

# The docs gate's original cases.
branch astra/rule-change; commit crates/core/src/economy.rs 'pub fn tax(n: i32) -> i32 { n / 3 }'
branch claude/docs-only; commit docs/learnings.md note; commit .claude/skills/x/SKILL.md s
branch astra/tests-only; commit app/e2e/a.spec.ts t; commit crates/core/src/foo_tests.rs 'let x = 0.5;'
branch astra/reviewed; commit crates/server/src/host.rs 'fn serve() {}'; ok docs astra/reviewed
REVIEWED_TIP="$(git -C "$R" rev-parse astra/reviewed)"
commit .claude/skills/legatum-rules/references/game-rules.md fixed   # the reviewer's own doc commit
branch astra/code-after; commit crates/server/src/host.rs 'fn a() {}'; ok docs astra/code-after
commit crates/server/src/host.rs 'fn b() {}'

# Engine invariants (each branch carries a docs marker, so only the static check can block it).
eng() { branch "$1"; commit "${3:-crates/core/src/economy.rs}" "$2"; ok docs "$1"; }
eng astra/float          'pub fn share(n: i32) -> f64 { n as f64 * 0.5 }'
eng astra/float-allowed  '// gate-allow: percentage for the debug overlay only, never feeds the rules
pub fn share(n: i32) -> f64 { n as f64 * 0.5 }'
eng astra/float-in-test  '#[cfg(test)]
mod tests {
    #[test]
    fn ratio() { assert!(0.5 < 1.0_f64); }
}'
eng astra/float-in-string 'pub const LABEL: &str = "costs 1.5x"; // 2.5 in a comment'
eng astra/hashmap        'pub fn count(v: &[u32]) -> usize { let s: std::collections::HashSet<_> = v.iter().collect(); s.len() }'
eng astra/rng            'pub fn roll() -> u32 { rand::thread_rng().gen() }'
eng astra/clock          'pub fn now() -> u64 { std::time::SystemTime::now().elapsed().unwrap().as_secs() }'
eng astra/ai-float       'pub fn score() -> f32 { 1.0 }' crates/ai/src/value.rs
eng astra/render-dep     'wgpu = "0.19"' crates/core/Cargo.toml
edit() { perl -pi -e "$2" "$R/$1"; }
branch astra/save-shape; edit crates/core/src/state.rs 's/^(    pub turn: u32,)$/$1\n    pub era: u8,/'
g add -A; g commit -q -m shape; ok docs astra/save-shape
branch astra/save-shape-bumped; edit crates/core/src/state.rs 's/^(    pub turn: u32,)$/$1\n    pub era: u8,/'
edit crates/core/src/lib.rs 's/= 5;/= 6;/'; edit crates/wasm/tests/fixtures/start.current.json 's/"save_version":5/"save_version":6/'
g add -A; g commit -q -m bump; ok docs astra/save-shape-bumped
branch astra/bump-stale-fixture; edit crates/core/src/lib.rs 's/= 5;/= 6;/'
g add -A; g commit -q -m bump; ok docs astra/bump-stale-fixture

# Hygiene (docs-only paths where possible, so no reviewer marker is involved).
# Probe lines for the personal-data classes come from the samples in tools/release/scrub-patterns.txt, so this file
# spells out none of them.
AWS="AKIA""ABCDEFGHIJKLMNOP"; HOME_PATH="/Users/""someone/LegatumRuns"; MAIL="jane.doe@""gmail.com"
RELATION="$(python3 "$SRC/tools/release/scrub.py" sample family)"; LAN_IP="$(python3 "$SRC/tools/release/scrub.py" sample private-ip)"
branch astra/secret;        commit docs/notes.md "key: $AWS"
branch astra/home-path;     commit docs/ops/run.md "runs land in $HOME_PATH"
branch astra/home-path-sim; commit docs/sim-results/run/README.md "runs land in $HOME_PATH"
branch astra/relation;      commit docs/playtests/notes.md "Note: $RELATION."
branch astra/lan-ip;        commit docs/ops/run.md "$LAN_IP"
branch astra/lan-ip-test;   commit crates/server/tests/peer.rs "// $LAN_IP"
branch astra/email;         commit docs/notes.md "ask $MAIL"
branch astra/email-example; commit docs/notes.md "ask you@example.com"
branch astra/save-file;     commit docs/game.legatum '{}'
branch astra/save-fixture;  commit crates/wasm/tests/fixtures/saves/a.current.legatum '{"save_version":5}'
branch astra/big-binary;    mkdir -p "$R/docs/run"; head -c 2500000 /dev/urandom > "$R/docs/run/blob.bin"; g add -A; g commit -q -m blob
branch astra/big-art;       mkdir -p "$R/docs/design/x"; head -c 2500000 /dev/urandom > "$R/docs/design/x/sheet.png"; g add -A; g commit -q -m art
branch astra/backup-file;   commit docs/learnings.md.backup old
branch astra/commented-code; commit crates/server/src/host.rs '// let port = config.port();'; ok docs astra/commented-code
branch astra/prose-comment;  commit crates/server/src/host.rs '// Legacy saves have no port; the host picks one.
// let the host pick one when the save has none.'; ok docs astra/prose-comment
branch astra/legacy-marker;  commit crates/server/src/host.rs 'fn old_host() {} // legacy'; ok docs astra/legacy-marker
branch astra/migration;      commit db/migrations/001_init.sql 'alter table t add column x int;'
branch astra/new-migration;  commit db/migrations/002_x.sql 'alter table t add column x int;'; ok docs astra/new-migration

# Reviewer markers by path: UI, copy, docs.
branch astra/ui-change;    commit app/src/ui/Panel.tsx 'export const Title = () => null;'; ok docs astra/ui-change
branch astra/ui-reviewed;  commit app/src/ui/Panel.tsx 'export const Heading = () => null;'; ok docs astra/ui-reviewed; ok ui astra/ui-reviewed 'ship'
branch astra/copy-change;  commit app/src/data.ts 'export const MORE = "Walls: +120 defence.";'; ok docs astra/copy-change
branch astra/copy-clean;   commit app/src/data.ts 'export const MORE = "Walls: +150 defence.";'; ok docs astra/copy-clean; ok plain astra/copy-clean 'reviewed 2026-11-21 by legatum-plain-language-reviewer'
# A sentence added to any app source file is player text too; a class name or import is not.
branch astra/prose-tsx;    commit app/src/ui/Panel.tsx 'export const Hint = () => <p>Barbarians never attack the Celts.</p>;'; ok docs astra/prose-tsx; ok ui astra/prose-tsx ship
branch astra/noprose-tsx;  commit app/src/ui/Panel.tsx 'export const Box = () => <div className="fs-tip fs-col big quiet" />;'; ok docs astra/noprose-tsx; ok ui astra/noprose-tsx ship

# Art: an identical pair fails the lookalike check, a distinct one passes (both carry docs and UI markers).
branch astra/art-dupe; cp "$R/app/src/assets/icons/building-granary.svg" "$R/app/src/assets/icons/building-forge.svg"
g add -A; g commit -q -m dupe; ok docs astra/art-dupe; ok ui astra/art-dupe ship; ok art astra/art-dupe pass
branch astra/art-ok; cp "$SRC/app/src/assets/icons/building-broodhall.svg" "$R/app/src/assets/icons/"
g add -A; g commit -q -m art; ok docs astra/art-ok; ok ui astra/art-ok ship; ok art astra/art-ok pass
branch astra/art-noreview; cp "$SRC/app/src/assets/icons/building-broodhall.svg" "$R/app/src/assets/icons/"
g add -A; g commit -q -m "art, unreviewed"; ok docs astra/art-noreview; ok ui astra/art-noreview ship

# Scenario-only branches: the fast scenario check stands in for the docs review and the extended verify.
scen() {  # scen <id> [save_version] [briefing] [state data suffix] [baseline docs: yes|no]
  local id="$1" v="${2:-5}" briefing="${3:-You are Athens. Take the city before turn 90.}" suffix="${4:-}" docs="${5:-yes}"
  python3 - "$R/scenarios/lab/$id.json" "$id" "$v" "$briefing" "$suffix" <<'PY'
import json, sys
path, sid, v, briefing, suffix = sys.argv[1:6]
doc = {"format": "legatum-scenario", "version": 1, "id": sid, "revision": 1, "title": sid, "briefing": briefing,
       "objective": {"summary": "Take it.", "turns": 5, "score": [{"metric": "ObjectiveMet", "weight": 1}]},
       "lab": {"question": "Can you?", "blurb": "Two lines."}, "baseline": {"runs": 5},
       "state": {"encoding": "json", "hash": "123", "data": json.dumps({"save_version": int(v), "note": suffix})}}
import os; os.makedirs(os.path.dirname(path), exist_ok=True)
json.dump(doc, open(path, "w"), indent=1)
PY
  g add -- "scenarios/lab/$id.json"
  if [ "$docs" = yes ]; then
    mkdir -p "$R/docs/sim-results/scenarios"; echo '{}' > "$R/docs/sim-results/scenarios/$id-baseline.json"; echo b > "$R/docs/sim-results/scenarios/$id-baseline.md"
    g add -- "docs/sim-results/scenarios/$id-baseline.json" "docs/sim-results/scenarios/$id-baseline.md"
  fi
  g commit -q -m "scenario $id"
}
branch claude/scn-ok;       scen scn-ok
branch claude/scn-stale;    scen scn-stale 4
branch claude/scn-nobase;   scen scn-nobase 5 "" "" no
branch claude/scn-engine;   scen scn-engine 5 "You are Athens. Take the city." " BADHASH"
branch claude/scn-copy;     scen scn-copy 5 "You score +1 Gold per complete pairs of cities."
branch claude/scn-and-code; scen scn-and-code; commit crates/server/src/host.rs 'fn serve() {}'
branch claude/scn-war;      mkdir -p "$R/scenarios/war"; echo '{}' > "$R/scenarios/war/war-x.json"; g add -A; g commit -q -m war
branch claude/scn-badname;  mkdir -p "$R/scenarios/lab"; echo '{}' > "$R/scenarios/lab/Bad_Name.json"; g add -A; g commit -q -m badname

g checkout -q master
g worktree add -q "$T/wt" astra/rule-change

pass=0; fail=0
check() {  # check <expect allow|deny> <cwd> <command> [description] [text the deny reason must contain]
  local out decision
  out="$(jq -n --arg c "$3" --arg d "$2" '{tool_input:{command:$c},cwd:$d}' | "$HOOK")"
  decision=allow; printf '%s' "$out" | grep -q '"deny"' && decision=deny
  if [ "$decision" = "$1" ] && { [ -z "${5:-}" ] || printf '%s' "$out" | grep -qF -- "$5"; }; then
    pass=$((pass + 1)); printf 'ok    %-5s %s\n' "$1" "${4:-$3}"
  else fail=$((fail + 1)); printf 'FAIL  want %s got %s: %s\n%s\n' "$1" "$decision" "${4:-$3}" "$out"; fi
}
m() { check "$1" "$R" "git merge --no-ff $2" "$3" "${4:-}"; }

echo "-- command parsing and the docs review"
check deny  "$R" 'git merge --no-ff astra/rule-change'                          "unreviewed code branch" "legatum-docs-reviewer"
check deny  "$R" 'git merge --no-ff -q astra/rule-change -m "Merge astra/rule-change: x"' "with -q and -m"
check deny  "$T" "git -C $R merge --no-ff astra/rule-change"                     "git -C <main checkout>"
check deny  "$T" "cd $R && git fetch -q . && git merge --no-ff astra/rule-change 2>&1 | tail -3" "chained after cd, piped"
check deny  "$T" "cd $R; B=astra/rule-change; git merge --no-ff \$B"             "branch in a shell variable"
check deny  "$T" "bash -c 'cd $R && git merge astra/rule-change'"                "inside bash -c"
check deny  "$R" "git merge --ff-only astra/rule-change"                         "fast-forward merge"
check deny  "$R" "git merge --no-ff astra/code-after"                            "code committed after the review"
check deny  "$R" "git merge --no-ff claude/docs-only astra/rule-change"          "octopus with one unreviewed branch"
check allow "$R" "git merge --no-ff claude/docs-only -m 'docs'"                  "docs/skills-only branch (auto)"
check allow "$R" "git merge --no-ff astra/tests-only"                            "test-only branch (auto, floats in tests are fine)"
check allow "$R" "git merge --no-ff astra/reviewed"                              "marker plus the reviewer's doc commit"
check allow "$R" "git merge --no-ff $REVIEWED_TIP"                               "merge by reviewed SHA"
check allow "$R" 'LEGATUM_GATE_BYPASS="hotfix for crash" git merge --no-ff astra/rule-change' "emergency override"
check allow "$T/wt" "git merge master"                                           "git merge master inside a worktree"
check allow "$T" "git -C $T/wt merge --no-ff -q master -m 'sync'"                "git -C <worktree> merge master"
check allow "$T/wt" "git merge --no-ff claude/docs-only"                         "merge into a non-master branch"
check allow "$R" "git merge --abort"                                             "merge --abort"
check allow "$R" "git log --merges -3"                                           "not a merge"
check allow "$R" "git merge-tree --write-tree master astra/rule-change"          "trial merge-tree"
check allow "$R" "echo 'git merge astra/rule-change'"                            "merge only inside a quoted string"
check allow "$R" "git merge origin/master"                                       "merging master's own remote"

echo "-- redirections (bug: \`git merge --no-ff B -m ... 2>&1\` read \`2>\` as a branch and was refused)"
check allow "$R" 'git merge --no-ff -q astra/reviewed -m "Merge astra/reviewed: x" 2>&1'   "2>&1 after -m"
check allow "$R" 'git merge --no-ff astra/reviewed -m "a > b & c" 2>&1 | tail -5'          "> and & inside the message"
check allow "$R" 'git merge --no-ff astra/reviewed >/dev/null 2>&1 && echo done'           ">/dev/null 2>&1 &&"
check allow "$R" 'git merge --no-ff astra/reviewed &> merge.log'                           "&> file"
check allow "$R" 'git merge --no-ff astra/reviewed 2> err.log; echo $?'                    "2> file"
check deny  "$R" 'git merge --no-ff astra/rule-change -m "Merge" 2>&1'                     "2>&1 still gates an unreviewed branch"
check deny  "$R" 'git merge --no-ff astra/rule-change 2>&1 &'                              "backgrounded merge still gates"

echo "-- engine invariants"
m deny  astra/float              "f64 and a float literal in core"                 "[float]"
m allow astra/float-allowed      "gate-allow on the line above"
m allow astra/float-in-test      "floats inside #[cfg(test)] mod"
m allow astra/float-in-string    "a number in a string and a comment"
m deny  astra/hashmap            "HashSet in core"                                 "[hash-order]"
m deny  astra/rng                "thread_rng"                                      "[rng]"
m deny  astra/clock              "SystemTime"                                      "[clock]"
m deny  astra/ai-float           "f32 in crates/ai"                                "[float]"
m deny  astra/render-dep         "wgpu in core's Cargo.toml"                       "[render]"
m deny  astra/save-shape         "GameState field without a RULES_VERSION bump"    "[save-shape]"
m allow astra/save-shape-bumped  "GameState field with a bump and refreshed fixtures"
m deny  astra/bump-stale-fixture "bump without refreshing .current fixtures"       "[fixtures]"

echo "-- hygiene"
m deny  astra/secret             "AWS access key"                                  "[secret]"
m deny  astra/home-path          "home path in an exported doc"                    "[personal]"
m allow astra/home-path-sim      "home path in docs/sim-results (not exported)"
m deny  astra/relation           "relationship word in an exported doc"            "[personal]"
m deny  astra/lan-ip             "private network address in an exported doc"      "[personal]"
m allow astra/lan-ip-test        "made-up private address in a test"
m deny  astra/email              "personal email address"                          "[personal]"
m allow astra/email-example      "example.com address"
m deny  astra/save-file          "committed save outside fixtures"                 "[save]"
m allow astra/save-fixture       "save under tests/fixtures"
m deny  astra/big-binary         "2.5 MB binary outside the art folders"           "[large-binary]"
m allow astra/big-art            "2.5 MB image under docs/design"
m deny  astra/backup-file        "*.backup file"                                   "[backup-file]"
m deny  astra/commented-code     "commented-out code"                              "[commented-code]"
m allow astra/prose-comment      "prose comments (legacy saves, a sentence starting with let)"
m deny  astra/legacy-marker      "// legacy marker"                                "[legacy-marker]"
m deny  astra/migration          "edit to a committed migration"                   "[migration]"
m allow astra/new-migration      "a new migration"

echo "-- reviewer markers by path"
m deny  astra/ui-change          "UI change without a UI review"                   "legatum-ui-reviewer"
m allow astra/ui-reviewed        "UI change with docs and UI markers"
m deny  astra/copy-change        "player text without the plain-language review"    "legatum-plain-language-reviewer"
m allow astra/copy-clean         "player text with the plain-language marker"
m deny  astra/prose-tsx          "a sentence added in a .tsx needs the plain review" "legatum-plain-language-reviewer"
m allow astra/noprose-tsx        "a class name in a .tsx is not player text"

echo "-- scenario-only branches (no docs review, no extended verify: the fast scenario check)"
m allow claude/scn-ok            "Lab scenario plus its baseline docs, no markers"
m deny  claude/scn-stale         "state made for another rules version"             "made for rules version 4"
m deny  claude/scn-nobase        "no Hard AI baseline doc"                          "baseline doc"
m deny  claude/scn-engine        "state that does not load on master's engine"      "does not load on master's engine"
m deny  claude/scn-copy          "briefing that fails the copy lint"                "[copy]"
m deny  claude/scn-and-code      "scenario plus code is the full flow"              "legatum-docs-reviewer"
m deny  claude/scn-war           "scenarios/war is not a Lab file"                  "legatum-docs-reviewer"
m deny  claude/scn-badname       "a file name outside the Lab pattern is code"      "legatum-docs-reviewer"
ok docs claude/scn-and-code; ok plain claude/scn-and-code
m allow claude/scn-and-code      "scenario plus code with the docs marker"
SCN_KIND="$(python3 -c "
import sys; sys.path.insert(0, '$SRC/tools/gates'); import check
b = check.git('$R', 'merge-base', 'master', 'claude/scn-ok').strip(); t = check.git('$R', 'rev-parse', 'claude/scn-ok').strip()
k = check.classify('$R', b, t); print(len(k['scenario']), len(k['code']), len(k['copy']))")"
[ "$SCN_KIND" = "1 0 0" ] && { pass=$((pass + 1)); echo "ok    classify: a Lab scenario is not code or copy"; } \
  || { fail=$((fail + 1)); echo "FAIL  classify: $SCN_KIND"; }

echo "-- art"
m deny  astra/art-dupe           "identical icon pair"                             "[art]"
m allow astra/art-ok             "distinct icons"
m deny  astra/art-noreview       "art with no art reviewer marker"                 "art review"

ok docs astra/rule-change; echo "$REVIEWED_TIP" > "$R/.progress/docs-review/astra-rule-change.ok"
check deny  "$R" "git merge --no-ff astra/rule-change"                           "marker naming a commit not in the branch"
ok docs astra/rule-change
check allow "$R" "git merge --no-ff astra/rule-change"                           "marker naming the exact tip"

grep -q 'override	astra/rule-change	.*hotfix for crash' "$R/.progress/merge-gates/bypass.log" \
  && { pass=$((pass + 1)); echo "ok    override logged to bypass.log"; } \
  || { fail=$((fail + 1)); echo "FAIL  override not logged"; cat "$R/.progress/merge-gates/bypass.log"; }
echo "passed $pass, failed $fail"
[ "$fail" -eq 0 ]
