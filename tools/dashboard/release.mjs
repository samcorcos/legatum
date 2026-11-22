// Release progress, derived from the living delivery queue (plans/playtest-feedback.md) so it
// can't go stale the way the hand-maintained .progress/plan.json did.

// Status cell → completion weight. The cell is free text ("review (branch …: bb1f7e6)",
// "queued (after A merges)", "increment 1 queued: astra …"), so we take the FIRST status keyword
// in it (normally the leading word):
//   done, merged                       → 1    (merged and verified on master)
//   review                             → 0.8  (on a branch, awaiting the lead's check)
//   implemented                        → 0.7  (built, not yet reviewed)
//   in progress, integration           → 0.4
//   queued, todo                       → 0    (not started)
//   held, hold, after, paused, blocked,
//   decision                           → 0    (waiting on something else)
//   anything else                      → 0    ("unknown", shown as such)
const KEYWORDS = /\b(done|merged|review|implemented|in progress|integration|queued|todo|held|hold|after|paused|blocked|decision)\b/i;
const CATEGORY = {
  done: "done", merged: "done",
  review: "review",
  implemented: "implemented",
  "in progress": "in progress", integration: "in progress",
  queued: "queued", todo: "queued",
  held: "held", hold: "held", after: "held", paused: "held", blocked: "held", decision: "held",
};
export const WEIGHTS = { done: 1, review: 0.8, implemented: 0.7, "in progress": 0.4, queued: 0, held: 0, unknown: 0 };
export const CATEGORIES = Object.keys(WEIGHTS);

export function parseStatus(text) {
  const clean = String(text ?? "").replace(/[*`_]/g, "").replace(/\s+/g, " ").trim();
  const m = KEYWORDS.exec(clean);
  const category = m ? CATEGORY[m[1].toLowerCase()] : "unknown";
  return { category, weight: WEIGHTS[category] };
}

const cells = (line) => line.trim().replace(/^\|/, "").replace(/\|$/, "").split(/(?<!\\)\|/).map((c) => c.trim());
const isSeparator = (line) => /^\|?[\s:|-]+\|?$/.test(line.trim()) && line.includes("-");
const pct = (sum, n) => (n ? Math.round((1000 * sum) / n) / 10 : 0);

function summarize(rows) {
  const counts = Object.fromEntries(CATEGORIES.map((c) => [c, 0]));
  let sum = 0;
  for (const r of rows) {
    counts[r.category] += 1;
    sum += r.weight;
  }
  return { pct: pct(sum, rows.length), done: counts.done, total: rows.length, counts, sum };
}

// Work tables live under "## <Letter>. <Title>" headings; anything else (the "Done today" log,
// prose) is ignored. Malformed rows are skipped; the rest still count.
export function parseQueue(markdown) {
  const sections = [];
  let sec = null;
  let header = null;
  const lines = String(markdown ?? "").split("\n");
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^#{1,2} /.test(line)) {
      const m = /^## ([A-Z])\.\s+(.+?)\s*$/.exec(line);
      sec = m ? { id: m[1], title: m[2], name: m[2].split(/:| \(/)[0].trim(), rows: [] } : null;
      if (sec) sections.push(sec);
      header = null;
      continue;
    }
    if (!sec || !line.trim().startsWith("|") || isSeparator(line)) continue;
    const row = cells(line);
    if (i + 1 < lines.length && isSeparator(lines[i + 1])) {
      header = row.map((c) => c.toLowerCase());
      continue;
    }
    if (row.length < 2 || !row[0]) continue;
    const col = (name, fallback) => {
      const j = header ? header.indexOf(name) : -1;
      return row[j >= 0 ? j : fallback] ?? "";
    };
    // A row with a missing or extra cell (e.g. no Owner) still has its status somewhere after the
    // item: take the Status column if it parses, else the last cell that does.
    const aligned = header && header.length === row.length;
    const candidates = [aligned ? col("status", row.length - 1) : null, ...row.slice(2).reverse()].filter((c) => c != null);
    const status = candidates.find((c) => parseStatus(c).category !== "unknown") ?? candidates[0] ?? "";
    sec.rows.push({
      id: row[0],
      item: col("item", 1),
      owner: aligned && header.includes("owner") ? col("owner", -1) : "",
      status,
      ...parseStatus(status),
    });
  }
  const out = sections.filter((s) => s.rows.length).map((s) => {
    const { sum, ...sum2 } = summarize(s.rows);
    return { ...s, ...sum2 };
  });
  const { sum, ...all } = summarize(out.flatMap((s) => s.rows));
  return { ...all, sections: out };
}
