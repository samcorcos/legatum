import assert from "node:assert/strict";
import { test } from "node:test";
import { parseQueue, parseStatus } from "./release.mjs";

test("status cells map to completion weights by their first keyword", () => {
  const cases = [
    ["done (merged 3715536)", "done", 1],
    ["merged", "done", 1],
    ["review (branch `worktree-x`: bb1f7e6)", "review", 0.8],
    ["Review", "review", 0.8],
    ["implemented; branch checks recorded in handoff", "implemented", 0.7],
    ["in progress", "in progress", 0.4],
    ["in progress (integration)", "in progress", 0.4],
    ["queued (after A merges)", "queued", 0],
    ["queued: astra `science-pacing`", "queued", 0],
    ["increment 1 queued: astra `celebrate-ruins`", "queued", 0],
    ["**todo**", "queued", 0],
    ["held until diplomacy is merged", "held", 0],
    ["after A lands its harness", "held", 0],
    ["", "unknown", 0],
    ["claude sound agent", "unknown", 0],
  ];
  for (const [text, category, weight] of cases) assert.deepEqual(parseStatus(text), { category, weight }, text);
});

const md = `# Queue

Status key: **todo** · **done**

## A. UI quality: the full QA pass

| # | Item | Status |
| --- | --- | --- |
| A1 | One | done (merged abc) |
| A2 | Two | review (branch) |
| A3 | Three | in progress |
| A4 | Four | queued |

## B. Controls and flow (Civ 6 conventions)

| # | Item | Owner | Status |
| --- | --- | --- | --- |
| B1 | Missing owner cell | queued |
| B2 | Extra | astra | implemented | stray cell |
| garbage
| B3 | Normal | claude | todo |

## Done today (for the record)

| X1 | not a work row | done |
- Dashboard fix.
`;

test("parses A–F work tables, skips the Done log and survives malformed rows", () => {
  const q = parseQueue(md);
  assert.deepEqual(q.sections.map((s) => [s.id, s.name, s.total]), [["A", "UI quality", 4], ["B", "Controls and flow", 3]]);
  const a = q.sections[0];
  assert.equal(a.done, 1);
  assert.equal(a.pct, 55); // (1 + 0.8 + 0.4 + 0) / 4
  const b = q.sections[1];
  assert.deepEqual(b.rows.map((r) => [r.id, r.category]), [["B1", "queued"], ["B2", "implemented"], ["B3", "queued"]]);
  assert.equal(b.rows[2].owner, "claude");
  assert.equal(q.total, 7);
  assert.equal(q.done, 1);
  assert.equal(q.pct, 41.4); // (2.2 + 0.7) / 7
});

test("missing or empty input yields an empty, well-formed result", () => {
  for (const input of [null, "", "no tables here"]) {
    const q = parseQueue(input);
    assert.deepEqual([q.pct, q.done, q.total, q.sections.length], [0, 0, 0, 0]);
  }
});
