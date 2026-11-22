"""Evidence and interchange contract for the AI review and evolution coach. Stdlib only."""
import copy
import json
from pathlib import Path
import re

PROPOSALS_SCHEMA = json.loads(Path(__file__).with_name("ai-proposals.schema.json").read_text())
PROPOSAL = PROPOSALS_SCHEMA["properties"]["proposals"]["items"]
EVIDENCE = PROPOSAL["properties"]["evidence"]
VERSION_LIMITATION = (
    "Version matching uses the archived engine commit, not today's checkout. Archives do not "
    "record brain policy or tuning parameter identity; verify those separately before adopting "
    "a result for an evolution candidate. These comparisons are observational, not causal."
)


def obj(properties):
    return {"type": "object", "properties": properties, "required": list(properties), "additionalProperties": False}


def extend_schema(base):
    result = copy.deepcopy(base)
    text = {"type": "string", "pattern": r"\S"}
    # Keep uniqueness in the interchange contract/local validator, not the model's
    # constrained-output schema. Repeated turn citations are still rejected locally.
    evidence = copy.deepcopy(EVIDENCE)
    evidence["items"]["properties"]["turns"].pop("uniqueItems")
    proposal = copy.deepcopy(PROPOSAL)
    proposal["properties"]["evidence"] = evidence
    seat = obj({"game_id": {"type": "string"}, "player": {"type": "integer", "minimum": 0, "maximum": 7}})
    result["properties"].update({
        "what_worked": {
            "type": "array", "maxItems": 30,
            "items": obj({
                "behavior": text, "why_succeeded": text,
                "evidence": evidence, "confidence": PROPOSAL["properties"]["confidence"],
            }),
        },
        "what_worked_limitations": {"type": "string"},
        "comparison": obj({
            "summary": text,
            "limitations": {"type": "array", "items": text, "minItems": 1, "maxItems": 20},
            "pairs": {
                "type": "array", "maxItems": 30,
                "items": obj({"winning": seat, "losing": seat, "explanation": text, "evidence": evidence}),
            },
        }),
        "proposals": {"type": "array", "maxItems": 30, "items": proposal},
    })
    result["required"] = list(result["properties"])
    result["properties"]["findings"]["maxItems"] = 30
    result["properties"]["findings"]["items"]["properties"]["expected_effect"] = text
    result["properties"]["findings"]["items"]["required"].append("expected_effect")
    return result


def validate_shape(value, schema, label="document"):
    """Validate the JSON Schema keywords used by our checked-in contract (no dependencies)."""
    kind = schema["type"]
    valid_type = {
        "object": isinstance(value, dict), "array": isinstance(value, list),
        "string": isinstance(value, str), "integer": type(value) is int,
    }[kind]
    if not valid_type:
        raise ValueError(f"{label}: expected {kind}")
    if "const" in schema and value != schema["const"]:
        raise ValueError(f"{label}: invalid constant")
    if "enum" in schema and value not in schema["enum"]:
        raise ValueError(f"{label}: invalid enum")
    if kind == "object":
        if set(schema.get("required", ())) - set(value):
            raise ValueError(f"{label}: missing fields")
        if schema.get("additionalProperties") is False and set(value) - set(schema["properties"]):
            raise ValueError(f"{label}: unknown fields")
        for key, child in value.items():
            if key in schema["properties"]:
                validate_shape(child, schema["properties"][key], f"{label}.{key}")
    elif kind == "array":
        if not schema.get("minItems", 0) <= len(value) <= schema.get("maxItems", float("inf")):
            raise ValueError(f"{label}: invalid item count")
        if schema.get("uniqueItems") and len({json.dumps(v, sort_keys=True) for v in value}) != len(value):
            raise ValueError(f"{label}: duplicate items")
        for i, child in enumerate(value):
            validate_shape(child, schema["items"], f"{label}[{i}]")
    elif kind == "integer":
        if not schema.get("minimum", -float("inf")) <= value <= schema.get("maximum", float("inf")):
            raise ValueError(f"{label}: out of range")
    elif "pattern" in schema and not re.search(schema["pattern"], value):
        raise ValueError(f"{label}: invalid string")


def seat_key(report, seat):
    """Conservative strata: no unknown revision, draws, unfinished/anchored games or humans."""
    revision = report.get("engine_commit")
    if (
        not revision or revision.lower() == "unknown" or seat.get("human") is not False
        or not report.get("finished") or not report.get("has_winner")
        or not report.get("complete_history") or not seat.get("difficulty")
    ):
        return None
    return (
        revision, report["rules_fingerprint"], tuple(report["map_size"]),
        seat["faction"], seat["difficulty"],
    )


def available_pairs(primary, reports):
    pairs = []
    for other in sorted(reports.values(), key=lambda r: r["game_id"]):
        if other["game_id"] == primary["game_id"]:
            continue
        for mine in primary["seats"]:
            key = seat_key(primary, mine)
            for theirs in other["seats"]:
                if key is None or key != seat_key(other, theirs) or mine["won"] == theirs["won"]:
                    continue
                a = {"game_id": primary["game_id"], "player": mine["player"]}
                b = {"game_id": other["game_id"], "player": theirs["player"]}
                pairs.append({"winning": a if mine["won"] else b, "losing": b if mine["won"] else a})
    return pairs


def validate_evidence(evidence, reports):
    validate_shape(evidence, EVIDENCE, "evidence")
    for citation in evidence:
        report = reports.get(citation["game_id"])
        if report is None:
            raise ValueError("evidence cites a game outside the verified review context")
        if not any(s["player"] == citation["player"] and not s["human"] for s in report["seats"]):
            raise ValueError("evidence must cite an AI seat")
        turns = {
            t["turn"] for t in report["turns"]
            if any(p["player"] == citation["player"] for p in t["players"])
        }
        if not set(citation["turns"]) <= turns:
            raise ValueError("evidence cites unobserved player turns")


def validate_additions(review, primary, reports):
    for success in review["what_worked"]:
        validate_evidence(success["evidence"], reports)
        if not any(e["game_id"] == primary["game_id"] for e in success["evidence"]):
            raise ValueError("what worked must cite the reviewed game")
    if not review["what_worked"] and not review["what_worked_limitations"].strip():
        raise ValueError("an empty what_worked list requires an evidence limitation")
    eligible = available_pairs(primary, reports)
    if eligible and not review["comparison"]["pairs"]:
        raise ValueError("comparison must discuss available same-version wins and losses")
    for pair in review["comparison"]["pairs"]:
        identity = {k: pair[k] for k in ("winning", "losing")}
        if identity not in eligible:
            raise ValueError("comparison must pair different games with matched versions and opposite outcomes")
        validate_evidence(pair["evidence"], reports)
        if not all(
            any(e["game_id"] == side["game_id"] and e["player"] == side["player"] for e in pair["evidence"])
            for side in identity.values()
        ):
            raise ValueError("comparison must cite turns for both seats")
    for proposal in review["proposals"]:
        validate_evidence(proposal["evidence"], reports)


def proposals_document(review, findings, primary, reports, revision, source_digest, generated_at):
    # Incident-backed fixes are also exported, so the coach never misses the existing task path.
    proposals = list(review["proposals"])
    incidents = {i["id"]: i for i in primary["incidents"]}
    for finding in findings:
        by_player = {}
        for incident_id in finding["incident_ids"]:
            incident = incidents[incident_id]
            by_player.setdefault(incident["player"], set()).add(incident["turn"])
        proposals.append({
            "kind": "new_behavior", "description": finding["proposal"],
            # The schema caps each evidence entry at 100 turns: split long citations.
            "evidence": [
                {"game_id": primary["game_id"], "player": player, "turns": ordered[i:i + 100]}
                for player, turns in sorted(by_player.items())
                for ordered in [sorted(turns)]
                for i in range(0, len(ordered), 100)
            ],
            "expected_effect": finding["expected_effect"], "confidence": finding["confidence"],
        })
    # Exact duplicate model proposals should not create duplicate coach experiments.
    proposals = list({json.dumps(p, sort_keys=True): p for p in proposals}.values())
    document = {
        "schema_version": 1, "review_game_id": primary["game_id"],
        "generated_at": generated_at, "reviewed_code": revision,
        "reviewed_source_digest": source_digest, "comparison_basis": "archived_engine_commit",
        "games": [
            {k: r[k] for k in ("game_id", "engine_commit", "rules_fingerprint")}
            for r in sorted(reports.values(), key=lambda r: r["game_id"])
        ],
        "proposals": proposals,
    }
    validate_proposals(document, reports)
    return document


def validate_proposals(document, reports):
    validate_shape(document, PROPOSALS_SCHEMA, "proposals")
    if document["review_game_id"] not in reports:
        raise ValueError("missing reviewed game")
    expected = sorted(
        ({k: r[k] for k in ("game_id", "engine_commit", "rules_fingerprint")} for r in reports.values()),
        key=lambda r: r["game_id"],
    )
    if sorted(document["games"], key=lambda r: r["game_id"]) != expected:
        raise ValueError("proposal provenance does not match verified games")
    for proposal in document["proposals"]:
        validate_evidence(proposal["evidence"], reports)


def compact_game(report, byte_limit=12000):
    """Sample the entire timeline, including incident-free games; disclose omitted frames."""
    frames = report["turns"]
    chosen, used = [], 0
    # Repeated midpoint splitting covers early, middle and late play before filling gaps.
    pending = [(0, len(frames) - 1)] if frames else []
    indices = [0, len(frames) - 1] if frames else []
    while pending:
        lo, hi = pending.pop(0)
        mid = (lo + hi) // 2
        indices.append(mid)
        if lo < mid:
            pending.append((lo, mid - 1))
        if mid < hi:
            pending.append((mid + 1, hi))
    for index in dict.fromkeys(indices):
        frame = frames[index]
        row = {"turn": frame["turn"], "players": frame["players"]}
        size = len(json.dumps(row, ensure_ascii=True).encode())
        if used + size <= byte_limit:
            chosen.append(row)
            used += size
    return {
        **{k: report[k] for k in (
            "game_id", "engine_commit", "rules_fingerprint", "complete_history",
            "finished", "has_winner", "map_size", "seats",
        )},
        "total_turns": len(frames), "omitted_turns": len(frames) - len(chosen),
        "turns": sorted(chosen, key=lambda t: t["turn"]),
    }
