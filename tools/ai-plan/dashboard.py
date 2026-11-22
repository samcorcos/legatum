#!/usr/bin/env python3
"""Build plans/ai-war-dashboard.html, a local dashboard for the AI war plan.

Reads plans/ai-war-status.json (the only data source) and plans/ai-war.prd.json
(story count only), and writes one self-contained HTML file with inline CSS and
no JavaScript, so it opens straight from disk with file://.

Standard library only. Run: python3 tools/ai-plan/dashboard.py
"""

import datetime as dt
import html
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS_PATH = ROOT / "plans" / "ai-war-status.json"
PRD_PATH = ROOT / "plans" / "ai-war.prd.json"
OUT_PATH = ROOT / "plans" / "ai-war-dashboard.html"

STEP_STATES = ("implemented", "validated", "shipped", "played")
STATE_LABELS = {"yes": "yes", "no": "no", "unknown": "unknown"}

CSS = """
:root {
  --bg: #f6f7f8;
  --fg: #1c1f24;
  --muted: #5d6570;
  --line: #d5d9de;
  --track: #e4e7eb;
  --accent: #2c6cb0;
  --done: #2e7d46;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--bg);
  color: var(--fg);
  font: 15px/1.5 system-ui, -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}
main { max-width: 860px; margin: 0 auto; padding: 24px 16px 48px; overflow-wrap: anywhere; }
h1 { font-size: 1.6rem; margin: 0 0 4px; }
h2 { font-size: 1.05rem; margin: 0 0 8px; }
p { margin: 4px 0; }
ul.plain { margin: 0; padding-left: 1.2em; }
ul.plain li { margin: 4px 0; }
section { border-top: 1px solid var(--line); padding: 16px 0 8px; }
.goal { font-size: 1.05rem; margin-top: 8px; }
.small, .meta { color: var(--muted); font-size: 0.85rem; }
.row { padding: 10px 0; border-bottom: 1px solid var(--line); }
.row:last-child { border-bottom: 0; }
.row-head { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: baseline; gap: 2px 12px; }
.name { font-weight: 600; }
.sid { color: var(--muted); font-weight: 400; margin-right: 4px; }
.step-statuses { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 6px; margin: 8px 0; }
.step-status { min-width: 0; border: 1px solid var(--line); border-radius: 6px; padding: 6px 8px; }
.step-status strong { display: block; font-size: .78rem; }
.step-status span, .step-status a { font-size: .78rem; overflow-wrap: anywhere; }
.state-yes strong { color: var(--done); }
.state-no strong { color: #9a6400; }
.state-unknown strong { color: #6b7280; }
.track { height: 8px; background: var(--track); border-radius: 4px; overflow: hidden; margin: 6px 0 4px; }
.fill { height: 100%; background: var(--accent); }
.fill.done { background: var(--done); }
.fill.warn { background: #c27c00; }
.fill.over { background: #b42318; }
.checklist { font-weight: 600; }
table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
th, td { text-align: left; vertical-align: top; padding: 6px 8px; border-bottom: 1px solid var(--line); }
th { color: var(--muted); font-weight: 600; }
th, td { overflow-wrap: normal; }
th:nth-child(-n+3), td:nth-child(-n+3) { white-space: nowrap; }
footer { margin-top: 24px; }
@media (max-width: 600px) {
  main { padding: 18px 12px 36px; }
  .step-statuses { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  thead { display: none; }
  table, tbody, tr, td { display: block; width: 100%; }
  tr { border-bottom: 1px solid var(--line); padding: 6px 0; }
  td { border: 0; padding: 2px 0; }
  td::before { content: attr(data-label) ": "; color: var(--muted); }
}
"""


def esc(value):
    return html.escape("" if value is None else str(value), quote=True)


def obj(value):
    return value if isinstance(value, dict) else {}


def records(value):
    """A list of dicts; bare strings become {"text": ...}; empty entries are dropped."""
    if not isinstance(value, list):
        return []
    return [v if isinstance(v, dict) else {"text": v} for v in value if v not in (None, "")]


def strings(value):
    if not isinstance(value, list):
        return []
    return [str(v) for v in value if v not in (None, "")]


def number(value):
    if isinstance(value, bool):
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def show(value):
    """Plain-text number for display; 'n/a' when missing; other values as text."""
    if value is None or value == "":
        return "n/a"
    n = number(value)
    if n is None:
        return str(value)
    if n.is_integer():
        return str(int(n))
    return f"{n:.2f}".rstrip("0").rstrip(".")


def clamp(percent):
    return max(0.0, min(100.0, percent))


def ratio(part, whole):
    p, w = number(part), number(whole)
    if p is None or w is None or w == 0:
        return None
    return clamp(p / w * 100.0)


def pct_value(value):
    n = number(value)
    return None if n is None else clamp(n)


def toward(baseline, current, target):
    """Progress from baseline to target, 0 to 100."""
    b, c, t = number(baseline), number(current), number(target)
    if c is None or t is None:
        return None
    b = 0.0 if b is None else b
    if t == b:
        return 100.0 if c >= t else 0.0
    return clamp((c - b) / (t - b) * 100.0)


def parse_date(value):
    try:
        return dt.date.fromisoformat(str(value)[:10])
    except ValueError:
        return None


def to_go(day, today):
    if day is None:
        return "date unknown"
    n = (day - today).days
    if n > 0:
        return f"{n} days to go"
    if n == 0:
        return "today"
    return f"{-n} days ago"


def bar(percent, label, spend=False):
    p = 0.0 if percent is None else percent
    if spend:
        # A budget bar warns as it fills; a full one is exhausted, not done.
        css = "fill over" if p >= 95 else "fill warn" if p >= 80 else "fill"
    else:
        css = "fill done" if p >= 100 else "fill"
    return (
        f'<div class="track" role="img" aria-label="{esc(label)}">'
        f'<div class="{css}" style="width:{p:.1f}%"></div></div>'
    )


def opt(tag, text, css=None):
    if not text:
        return ""
    cls = f' class="{css}"' if css else ""
    return f"<{tag}{cls}>{esc(text)}</{tag}>"


def section(heading, inner):
    return f"<section>\n<h2>{esc(heading)}</h2>\n{inner}\n</section>"


def bullets(lines, empty):
    if not lines:
        return f'<p class="small">{esc(empty)}</p>'
    items = "".join(f"<li>{esc(line)}</li>" for line in lines)
    return f'<ul class="plain">{items}</ul>'


def time_section(status, today):
    started_raw = status.get("started")
    started = parse_date(started_raw)
    cp = obj(status.get("checkpoint"))
    earliest_raw, latest_raw = cp.get("earliest"), cp.get("latest")
    earliest, latest = parse_date(earliest_raw), parse_date(latest_raw)

    elapsed_pct = None
    if started and latest:
        elapsed_pct = ratio((today - started).days, (latest - started).days)
    lines = []
    if started:
        lines.append(f"{(today - started).days} days since start ({started.isoformat()}).")
        if latest:
            lines.append(
                f"Elapsed {(today - started).days} of {(latest - started).days} days "
                "from start to the latest checkpoint."
            )
    else:
        lines.append("Start date unknown.")
    lines.append(f"Checkpoint earliest {show(earliest_raw)}: {to_go(earliest, today)}.")
    lines.append(f"Checkpoint latest {show(latest_raw)}: {to_go(latest, today)}.")

    body = "".join(f"<p>{esc(line)}</p>" for line in lines)
    body += bar(elapsed_pct, "Elapsed time from start to latest checkpoint")
    body += opt("p", cp.get("note"), "small")
    return section("Time", body)


def finish_section(status):
    rows = []
    for item in records(status.get("finish_line")):
        unit = item.get("unit") or ""
        current = item.get("current")
        of = item.get("of")
        if current is None:
            now = "pending"
            percent = None
        else:
            now = show(current) + (f" of {show(of)}" if of is not None else "")
            percent = toward(item.get("baseline"), current, item.get("target"))
        target = show(item.get("target")) + (f" ({unit})" if unit else "")
        name = item.get("name") or "Unnamed target"
        rows.append(
            '<div class="row">'
            f'<div class="name">{esc(name)}</div>'
            f'<p class="meta">Baseline {esc(show(item.get("baseline")))} &middot; '
            f"now {esc(now)} &middot; target {esc(target)}</p>"
            f"{bar(percent, name)}"
            f"{opt('p', item.get('note'), 'small')}"
            "</div>"
        )
    return section("Finish line", "".join(rows) or '<p class="small">No finish line set.</p>')


def steps_section(status):
    rows = []
    for step in records(status.get("steps")):
        name = step.get("name") or "Unnamed step"
        planned = step.get("planned") or "not dated"
        statuses = []
        for key in STEP_STATES:
            item = obj(obj(step.get("evidence")).get(key))
            state = str(item.get("state") or "unknown").lower()
            if state not in STATE_LABELS:
                state = "unknown"
            sha = item.get("commit") or "unknown"
            commit_link = (
                f'<a href="https://github.com/samcorcos/legatum/commit/{esc(sha)}">'
                f'{esc(sha)}</a>' if sha != "unknown" else "unknown"
            )
            evidence = item.get("file")
            evidence_link = (
                f'<a href="../{esc(evidence)}" title="{esc(evidence)}">'
                f'{esc(evidence.rsplit("/", 1)[-1])}</a>'
                if evidence else "evidence unknown"
            )
            statuses.append(
                f'<div class="step-status state-{state}"><strong>{esc(key)}: '
                f'{STATE_LABELS[state]}</strong><span>commit {commit_link} · {evidence_link}</span></div>'
            )
        rows.append(
            '<div class="row">'
            '<div class="row-head">'
            f'<span class="name"><span class="sid">{esc(step.get("id"))}</span>{esc(name)}</span>'
            "</div>"
            f'<div class="step-statuses">{"".join(statuses)}</div>'
            f'<p class="meta">planned {esc(planned)}</p>'
            f"{opt('p', step.get('summary'))}"
            f"{opt('p', ('Left: ' + step['left']) if step.get('left') else None, 'small')}"
            "</div>"
        )
    intro = (
        '<p class="small">Implemented = code exists; validated = measured or checked; '
        'shipped = on by default on master; played = the designer played it in a playtest. '
        'Each state links to its evidence file and commit; unknown stays explicit.</p>'
    )
    return section("Steps", intro + ("".join(rows) or '<p class="small">No steps.</p>'))


def budget_row(label, value, percent, note, spend=True):
    return (
        '<div class="row">'
        f'<div class="row-head"><span class="name">{esc(label)}</span>'
        f'<span class="small">{esc(value)}</span></div>'
        f"{bar(percent, label, spend)}"
        f"{opt('p', note, 'small')}"
        "</div>"
    )


def reset_text(day):
    return f", resets {day}" if day else ""


def budget_section(status):
    budget = obj(status.get("budget"))
    aws = obj(budget.get("aws"))
    claude = obj(budget.get("claude"))
    codex = obj(budget.get("codex"))
    designer = obj(budget.get("designer_time"))

    extra = ""
    if claude.get("extra_cap") is not None:
        extra = f"; extra ${show(claude.get('extra_spent'))} of ${show(claude.get('extra_cap'))}"

    # Spend and subscription usage are optional: the tracked status file leaves them out (they are personal account
    # data), so each row appears only when its numbers are present.
    rows = []
    if aws.get("plan_spent") is not None:
        rows.append(budget_row(
            "AWS plan spend",
            f"${show(aws.get('plan_spent'))} of ${show(aws.get('plan_high'))} plan high "
            f"(plan low ${show(aws.get('plan_low'))})",
            ratio(aws.get("plan_spent"), aws.get("plan_high")),
            aws.get("note"),
        ))
    if aws.get("month_to_date") is not None:
        rows.append(budget_row(
            "AWS month to date",
            f"${show(aws.get('month_to_date'))} of ${show(aws.get('month_cap'))} month cap",
            ratio(aws.get("month_to_date"), aws.get("month_cap")),
            None,
        ))
    if claude.get("weekly_used_pct") is not None:
        rows.append(budget_row(
            "Claude weekly",
            f"{show(claude.get('weekly_used_pct'))}% used"
            + reset_text(claude.get("resets"))
            + extra,
            pct_value(claude.get("weekly_used_pct")),
            claude.get("note"),
        ))
    if codex.get("weekly_used_pct") is not None:
        rows.append(budget_row(
            "Codex weekly",
            f"{show(codex.get('weekly_used_pct'))}% used" + reset_text(codex.get("resets")),
            pct_value(codex.get("weekly_used_pct")),
            codex.get("note"),
        ))
    rows.append(budget_row(
        "Designer playtests",
        f"{show(designer.get('playtests_done'))} of "
        f"{show(designer.get('playtests_planned'))} done",
        ratio(designer.get("playtests_done"), designer.get("playtests_planned")),
        designer.get("note"),
        spend=False,
    ))
    if aws.get("note") and aws.get("plan_spent") is None:
        rows.append(f'<p class="small">{esc(aws.get("note"))}</p>')
    return section("Budget", "".join(rows))


def learnings_section(status):
    entries = sorted(
        records(status.get("learnings")),
        key=lambda e: str(e.get("date") or ""),
        reverse=True,
    )
    items = []
    for entry in entries:
        date = entry.get("date") or "undated"
        text = entry.get("text") or ""
        items.append(f'<li><span class="small">{esc(date)}</span> {esc(text)}</li>')
    body = f'<ul class="plain">{"".join(items)}</ul>' if items else '<p class="small">No learnings yet.</p>'
    return section("What we've learned", body)


def playtests_section(status):
    rows = records(status.get("playtests"))
    if not rows:
        return section("Playtests", '<p class="small">No playtests recorded.</p>')
    columns = [
        ("Date", "date"),
        ("Build", "build"),
        ("Faction", "faction"),
        ("Result", "result"),
        ("Takeaway", "takeaway"),
    ]
    head = "".join(f"<th>{esc(label)}</th>" for label, _ in columns)
    body = []
    for row in rows:
        cells = "".join(
            f'<td data-label="{esc(label)}">{esc(row.get(key))}</td>' for label, key in columns
        )
        body.append(f"<tr>{cells}</tr>")
    table = (
        f"<table><thead><tr>{head}</tr></thead>"
        f"<tbody>{''.join(body)}</tbody></table>"
    )
    return section("Playtests", table)


def checklist_line():
    try:
        prd = json.loads(PRD_PATH.read_text(encoding="utf-8"))
        stories = [s for s in prd.get("stories", []) if isinstance(s, dict)]
    except (OSError, ValueError, AttributeError):
        return "Checklist: unavailable (could not read plans/ai-war.prd.json)"
    passed = sum(1 for s in stories if s.get("passes") is True)
    return f"Checklist: {passed} of {len(stories)} stories passed"


def render(status, checklist, today):
    title = status.get("title") or "AI plan"
    parts = [
        "<header>",
        f"<h1>{esc(title)}</h1>",
        opt("p", status.get("goal"), "goal"),
        f'<p class="small">Updated {esc(status.get("updated") or "unknown")}</p>',
        "</header>",
        time_section(status, today),
        finish_section(status),
        steps_section(status),
        budget_section(status),
        section("Running now", bullets(strings(status.get("running_now")), "Nothing running.")),
        section(
            "Open decisions",
            bullets(strings(status.get("open_decisions")), "No open decisions."),
        ),
        learnings_section(status),
        playtests_section(status),
        section("PRD", f'<p class="checklist">{esc(checklist)}</p>'),
        '<footer class="small">Generated by tools/ai-plan/dashboard.py from '
        "plans/ai-war-status.json. Re-run the script to refresh.</footer>",
    ]
    return (
        "<!doctype html>\n"
        '<html lang="en">\n<head>\n'
        '<meta charset="utf-8">\n'
        '<meta name="viewport" content="width=device-width, initial-scale=1">\n'
        f"<title>{esc(title)}</title>\n"
        f"<style>{CSS}</style>\n"
        "</head>\n<body>\n<main>\n"
        + "\n".join(p for p in parts if p)
        + "\n</main>\n</body>\n</html>\n"
    )


def main():
    try:
        status = json.loads(STATUS_PATH.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        sys.exit(f"dashboard: cannot read {STATUS_PATH}: {exc}")
    if not isinstance(status, dict):
        sys.exit(f"dashboard: {STATUS_PATH} must hold a JSON object")

    page = render(status, checklist_line(), dt.date.today())
    OUT_PATH.write_text(page, encoding="utf-8")
    print(f"wrote {OUT_PATH} ({OUT_PATH.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
