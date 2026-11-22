#!/usr/bin/env python3
"""Summarize hash-checked gold_audit JSONL files and draw the three-game chart.

Usage: gold_usage_1026.py OUT.json OUT.png OUT.csv japan.jsonl athens.jsonl sparta.jsonl
The JSONL files come from gold_audit built at each save's metadata.engine_commit.
"""
import collections
import csv
import json
import statistics
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ERA = ["Ancient", "Classical", "Medieval", "Renaissance", "Industrial", "Modern", "Future"]
GAMES = ["Japan win", "Athens win", "Sparta win"]
COLORS = ["#ad452f", "#2c7186", "#c9973d", "#735985", "#4b8b62", "#a66b39", "#6c747b", "#d07c83"]
INCOME_COLORS = {"cities": "#7195a2", "buildings": "#cf9f4f", "trade": "#74a778", "interest": "#9b83bb", "sacks": "#c57d67", "plunder": "#af8e5b", "other": "#b8b8ac"}
SPEND_COLORS = {"units": "#b65d4b", "buildings": "#c58d43", "improvements": "#73996e", "upgrades": "#7f7aa4", "siege": "#a47769", "treaties": "#5999a9", "upkeep": "#8a8e8c", "companies": "#a8a36f"}


def item_kind(item):
    return {"Unit": "units", "Building": "buildings", "Wonder": "buildings",
            "Improvement": "improvements"}.get(item.get("kind"), "other")


def label_source(label, buildings):
    if label in buildings:
        return "buildings"
    if "commerce" in label.lower():
        return "trade"
    return "cities"


def event_accounting(event, spending, windfalls, buys, captures, vp_events, turn):
    typ = event["type"]
    owner = event.get("owner", event.get("player"))
    if typ == "ItemBought":
        kind = item_kind(event["item"])
        spending[owner][kind] += event["gold"]
        buys[owner].append(dict(turn=turn, category=kind, item=event["item"], gold=event["gold"]))
    elif typ == "SiegeBought":
        spending[owner]["siege"] += event["gold"]
    elif typ == "UnitUpgraded":
        spending[owner]["upgrades"] += event["gold"]
    elif typ == "TreatyPayment":
        spending[event["player"]]["treaties"] += event["gold"]
    elif typ == "CompanyHired":
        spending[owner]["companies"] += event["gold"]
        windfalls[event["seller"]]["trade"] += event["gold"]
    elif typ == "CityRazed":
        windfalls[event["by"]]["sacks"] += event.get("sack_gold") or 0
    elif typ == "CityCaptured":
        captures[event["to"]].append(turn)
        if event.get("bounty"):
            windfalls[event["to"]]["plunder"] += event["bounty"]
            windfalls[event["from"]]["plunder"] -= event["bounty"]
    elif typ in ("WonderDiscovered", "BarbarianCampCleared"):
        player = event.get("owner", event.get("player", event.get("by")))
        windfalls[player]["other"] += event["gold"]
    elif typ == "RuinExplored" and event["reward"]["kind"] == "Gold":
        windfalls[owner]["other"] += event["reward"]["amount"]
    elif typ in ("EraObjectiveScored", "FactionGoalCompleted", "GreatWorkCompleted", "TitleChanged"):
        vp_events.append(dict(turn=turn, **event))


def summarize(path):
    lines = [json.loads(line) for line in Path(path).read_text().splitlines()]
    meta = lines[0]["metadata"]
    seats = {s["player"]: s for s in meta["seats"]}
    by_era = collections.defaultdict(lambda: collections.defaultdict(lambda: {
        "turns": 0, "treasury": 0, "income": collections.Counter(), "spend": collections.Counter(),
        "windfalls": collections.Counter(), "delta": 0, "actual_regular_net": 0,
    }))
    balances = collections.defaultdict(list)
    buys = collections.defaultdict(list)
    captures = collections.defaultdict(list)
    vp_events = []
    purchase_turns = collections.defaultdict(list)
    wars = collections.defaultdict(set)
    for row in lines[1:]:
        t = row["turn"]
        spending = collections.defaultdict(collections.Counter)
        windfalls = collections.defaultdict(collections.Counter)
        for event in row["events"]:
            event_accounting(event, spending, windfalls, buys, captures, vp_events, t)
        for p in row["players"]:
            player = p["id"]
            if not p["alive"]:
                continue
            era = p["era"]
            rec = by_era[player][era]
            rec["turns"] += 1
            rec["treasury"] += p["gold"]
            balances[player].append((t, p["gold"]))
            if p["wars"]:
                wars[player].add(t)
            for c in p["cities"]:
                buildings = set(c.get("buildings", []))
                for source in c["sources"]:
                    g = source["yields"]["gold"]
                    rec["income"][label_source(source["label"], buildings)] += g
            for label, amount in p["income"]["parts"]:
                if label == "Switzerland: interest":
                    rec["income"]["interest"] += amount
                elif label.startswith("Decadence") and "interest" in label:
                    rec["income"]["interest"] += amount
                elif label == "Unit upkeep":
                    # The core's balancing entry includes city/treaty/overreach terms.
                    rec["spend"]["upkeep"] -= amount
                elif label.startswith("Treaty upkeep:"):
                    rec["spend"]["treaties"] -= amount
                elif label == "City administration" or label.startswith("Overreach:"):
                    rec["spend"]["upkeep"] -= amount
            for cat, value in spending[player].items():
                rec["spend"][cat] += value
                if cat != "upkeep":
                    purchase_turns[player].append((t, cat, value))
            for cat, value in windfalls[player].items():
                rec["windfalls"][cat] += value
                rec["income"][cat if cat in ("trade", "sacks", "plunder") else "other"] += value
            after = next(a["gold"] for a in row["after"] if a["id"] == player)
            delta = after - p["gold"]
            rec["delta"] += delta
            rec["actual_regular_net"] += delta + sum(spending[player].values()) - sum(windfalls[player].values())
    result = {"metadata": meta, "players": []}
    for player, seat in seats.items():
        era_rows = []
        for era, rec in sorted(by_era[player].items()):
            era_rows.append({"era": ERA[era], "era_index": era, "turns": rec["turns"],
                "average_treasury": round(rec["treasury"] / rec["turns"]),
                "income": dict(rec["income"]), "spend": dict(rec["spend"]),
                "windfalls": dict(rec["windfalls"]), "delta": rec["delta"],
                "actual_regular_net": rec["actual_regular_net"]})
        income = collections.Counter()
        spend = collections.Counter()
        for e in era_rows:
            income.update(e["income"])
            spend.update(e["spend"])
        b = buys[player]
        unit_buys = [x for x in b if x["category"] == "units"]
        unit_near_war = sum(any(abs(x["turn"] - w) <= 3 for w in wars[player]) for x in unit_buys)
        result["players"].append({"player": player, "faction": seat["faction"],
            "human": seat["human"], "final_vp": seat["final_vp"],
            "average_treasury": round(statistics.mean(g for _, g in balances[player])) if balances[player] else 0,
            "max_treasury": max((g for _, g in balances[player]), default=0),
            "treasury": balances[player], "eras": era_rows, "income": dict(income),
            "spend": dict(spend), "buys": b, "capture_turns": captures[player],
            "unit_buys_near_war": unit_near_war, "unit_buys": len(unit_buys),
            "war_turns": len(wars[player]), "purchase_turns": purchase_turns[player]})
    result["vp_events"] = vp_events
    return result


def font(size, bold=False):
    paths = ["/System/Library/Fonts/Supplemental/Arial Bold.ttf" if bold else "/System/Library/Fonts/Supplemental/Arial.ttf",
             "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]
    for path in paths:
        try:
            return ImageFont.truetype(path, size)
        except OSError:
            pass
    return ImageFont.load_default()


def draw_chart(games, out):
    heights = [max(825, 465 + (45 if len(game["players"]) > 4 else 0)
                   + len(game["players"]) * 55 + 75) for game in games]
    im = Image.new("RGB", (1600, 175 + sum(heights) + 30 * 2 + 80), "#f8f5ee")
    d = ImageDraw.Draw(im)
    d.text((70, 40), "Where the Gold goes", font=font(48, True), fill="#262b2c")
    d.text((72, 105), "Three recorded wins · each line is one empire · Gold totals by source and use", font=font(23), fill="#495154")
    y0 = 175
    for gi, game in enumerate(games):
        d.rounded_rectangle((55, y0, 1545, y0 + heights[gi]), radius=20, fill="#fffdf8", outline="#ded8ca", width=2)
        d.text((80, y0+24), GAMES[gi], font=font(34, True), fill="#272c2e")
        players = game["players"]
        x1,x2,yt,yb = 95,1500,y0+110,y0+360
        max_turn = max((q[0] for p in players for q in p["treasury"]), default=1)
        max_gold = max((q[1] for p in players for q in p["treasury"]), default=1)
        for frac in (0, .5, 1):
            y = int(yb-(yb-yt)*frac)
            d.line((x1,y,x2,y),fill="#e7e4dd",width=2)
            d.text((x1,y-25),f"{round(max_gold*frac):,}",font=font(18),fill="#697174")
        for p in players:
            color = COLORS[p["player"] % len(COLORS)]
            points = [(int(x1 + (x2-x1)*t/max_turn),int(yb-(yb-yt)*g/max_gold)) for t,g in p["treasury"]]
            if len(points)>1:d.line(points,fill=color,width=5 if p["human"] else 3,joint="curve")
        legend_y = y0+380
        for p in players:
            i=p["player"]
            x=95+(i%4)*350;y=legend_y+(i//4)*32
            d.line((x,y+10,x+28,y+10),fill=COLORS[i%len(COLORS)],width=6)
            d.text((x+38,y),f"{p['faction']}{' (human)' if p['human'] else ''}",font=font(20),fill="#333b3d")
        bar_start=y0+465+(45 if len(players)>4 else 0)
        max_total=max((max(sum(max(0,v) for v in p[k].values()) for k in ("income","spend")) for p in players),default=1)
        bar_w=1040
        d.text((90,bar_start-35),"Gold gained and spent · whole game",font=font(22,True),fill="#303739")
        for i,p in enumerate(players):
            yy=bar_start+i*55
            d.text((95,yy+8),p["faction"][:13],font=font(19,True) if p["human"] else font(19),fill="#333b3d")
            for typ,offset,palette in (("income",0,INCOME_COLORS),("spend",24,SPEND_COLORS)):
                left=315
                for cat,col in palette.items():
                    val=max(0,p[typ].get(cat,0))
                    width=round(bar_w*val/max_total)
                    if width:
                        d.rectangle((left,yy+offset,left+width,yy+offset+18),fill=col)
                        left+=width
                d.text((1390,yy+offset-2),f"{sum(p[typ].values()):,}",font=font(18),fill="#394244")
        legend_bottom=bar_start+len(players)*55+5
        d.text((92,legend_bottom),"Gain",font=font(17,True),fill="#343b3d")
        xx=150
        for cat,col in INCOME_COLORS.items():
            d.rectangle((xx,legend_bottom+3,xx+15,legend_bottom+18),fill=col)
            d.text((xx+21,legend_bottom),cat,font=font(16),fill="#596064")
            xx+=185
        d.text((92,legend_bottom+28),"Spend",font=font(17,True),fill="#343b3d")
        xx=170
        for cat,col in SPEND_COLORS.items():
            d.rectangle((xx,legend_bottom+31,xx+14,legend_bottom+45),fill=col)
            d.text((xx+20,legend_bottom+28),cat,font=font(16),fill="#596064")
            xx+=165
        y0 += heights[gi] + 30
    d.text((70,im.height-48),"Source: hash-checked original-engine tape replays. Income sources are start-of-turn forecasts; spending uses resolution events.",font=font(16),fill="#687174")
    im.save(out)


def main():
    summary, chart, table, *paths = sys.argv[1:]
    if len(paths) != 3:
        raise SystemExit(__doc__)
    games = [summarize(p) for p in paths]
    Path(summary).write_text(json.dumps(games, indent=2))
    draw_chart(games, chart)
    income_columns = ["cities", "buildings", "trade", "interest", "sacks", "plunder", "other", "tribute"]
    spend_columns = ["units", "buildings", "improvements", "tile_buys", "upgrades", "siege", "treaties", "upkeep", "companies"]
    with Path(table).open("w", newline="") as file:
        writer = csv.writer(file, lineterminator="\n")
        writer.writerow(["game", "player", "faction", "human", "era", "turns", "average_treasury",
                         *["income_" + c for c in income_columns],
                         *["spend_" + c for c in spend_columns], "actual_regular_net", "treasury_change"])
        for name, game in zip(GAMES, games):
            for p in game["players"]:
                for era in p["eras"]:
                    writer.writerow([name, p["player"], p["faction"], int(p["human"]), era["era"],
                                     era["turns"], era["average_treasury"],
                                     *[era["income"].get(c, 0) for c in income_columns],
                                     *[era["spend"].get(c, 0) for c in spend_columns],
                                     era["actual_regular_net"], era["delta"]])
    for name, game in zip(GAMES, games):
        print(name, "turns", game["metadata"]["turn_count"])
        for p in game["players"]:
            print(p["player"],p["faction"],"avg",p["average_treasury"],"income",p["income"],"spend",p["spend"],"captures",len(p["capture_turns"]),"unit war",p["unit_buys_near_war"],"/",p["unit_buys"])


if __name__ == "__main__":
    main()
