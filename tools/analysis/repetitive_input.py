#!/usr/bin/env python3
"""Repetitive per-city input in a human's recorded game, counted from the save's order tape.

    python3 -I tools/analysis/repetitive_input.py <save.legatum> [<save.legatum> ...] [--json]

A save's `game.archive.tape.turns[*].orders[seat]` holds the orders each seat committed at Ready, so this counts
committed orders (the lower bound on clicks: an item chosen and then replaced within one turn never reaches the tape).
Only the human seat is read. Standard library only. Method and results:
docs/design/research/repetitive-input-2026-10-31.md
"""
import argparse
import json
from collections import Counter, defaultdict

WINDOWS = (0, 5, 10)
CITY_ORDERS = ('SetProduction', 'SetProductionQueue', 'BuyItem')


def item_keys(item):
    """(exact key, type key): the type key drops the tile, so Quarry on two tiles is one pick of 'Quarry'."""
    exact = json.dumps(item, sort_keys=True)
    loose = {k: v for k, v in item.items() if k != 'tile'}
    return exact, json.dumps(loose, sort_keys=True)


def label(loose_key):
    item = json.loads(loose_key)
    return item.get('building') or item.get('unit') or item.get('improvement') or item.get('project') or item.get('kind')


def picks_of(order, last_queue):
    """The items a production order newly asks for: a queue's items not already in that city's previous queue."""
    if order['type'] == 'SetProduction':
        return [order['item']]
    if order['type'] == 'BuyItem':
        return []
    before = Counter(json.dumps(i, sort_keys=True) for i in last_queue)
    out = []
    for item in order['queue']:
        key = json.dumps(item, sort_keys=True)
        if before[key] > 0:
            before[key] -= 1
        else:
            out.append(item)
    return out


def analyse(path):
    save = json.load(open(path))
    game = json.loads(save['game'])
    archive = game['archive']
    seats = archive['metadata']['seats']
    me = next(s['player'] for s in seats if s['human'])
    turns = archive['tape']['turns']
    times = {t['turn']: t['active_seconds'] for t in archive['turn_times']} if 'turn_times' in archive else {}
    cities_by_turn = {}
    for row in archive.get('telemetry', []):
        for p in row['players']:
            if p['player'] == me:
                cities_by_turn[row['turn']] = p['cities']

    totals = Counter()
    last_queue = {}
    if archive['metadata']['first_turn'] > 1:       # a game resumed mid-way: seed each city with what it was building
        for c in json.loads(archive['initial_state'])['cities'].values():
            if c['owner'] == me:
                last_queue[c['id']] = ([c['producing']] if c.get('producing') else []) + c.get('production_queue', [])
    edits_by_city = defaultdict(list)       # city -> [(turn, queue)]
    pick_log = []                            # (turn, city, exact, loose)
    queue_log = []                           # (turn, city, tuple of loose keys)
    unit_runs = Counter()
    per_turn_orders = {}
    for entry in turns:
        turn = entry['turn']
        orders = entry['orders'].get(str(me), [])
        per_turn_orders[turn] = len(orders)
        for o in orders:
            totals[o['type']] += 1
        by_type = Counter(o['type'] for o in orders if 'unit' in o)
        for kind, n in by_type.items():
            if n >= 3:
                unit_runs[kind] += n
        for o in orders:
            if o['type'] not in CITY_ORDERS:
                continue
            city = o['city']
            if o['type'] == 'BuyItem':
                pick_log.append((turn, city, *item_keys(o['item']), 'buy'))
                continue
            for item in picks_of(o, last_queue.get(city, [])):
                pick_log.append((turn, city, *item_keys(item), 'pick'))
            queue = o['queue'] if o['type'] == 'SetProductionQueue' else [o['item']]
            queue_log.append((turn, city, tuple(item_keys(i)[1] for i in queue)))
            edits_by_city[city].append((turn, queue, last_queue.get(city)))
            last_queue[city] = queue

    picks = [p for p in pick_log if p[4] == 'pick']
    buys = [p for p in pick_log if p[4] == 'buy']
    result = {
        'file': path, 'human_seat': me, 'faction': next(s['faction'] for s in seats if s['human']),
        'first_turn': turns[0]['turn'], 'last_turn': turns[-1]['turn'], 'turns': len(turns),
        'complete_history': archive['metadata']['complete_history'],
        'active_seconds': sum(times.get(t['turn'], 0) for t in turns),
        'orders_total': sum(totals.values()), 'orders_by_type': dict(totals.most_common()),
        'city_orders': sum(totals[t] for t in CITY_ORDERS), 'item_picks': len(picks), 'buys': len(buys),
        'cities_first': cities_by_turn.get(turns[0]['turn']), 'cities_last': cities_by_turn.get(turns[-1]['turn']),
    }

    # A pick repeats when the same item went to a different city within the last W turns (W = 0: the same turn).
    flags = []                               # per pick: {(name, w): bool}
    for i, (turn, city, exact, loose, _) in enumerate(picks):
        f = {}
        for name, idx in (('exact', 2), ('type', 3)):
            for w in WINDOWS:
                f[(name, w)] = any(q[idx] == (exact if idx == 2 else loose) and q[1] != city and 0 <= turn - q[0] <= w for q in picks[:i])
        flags.append(f)
    for name in ('exact', 'type'):
        for w in WINDOWS:
            result[f'repeat_picks_{name}_w{w}'] = sum(f[(name, w)] for f in flags)

    # The same numbers for the turns an empire has 10 or more cities (the 10-15 city task), plus a click model:
    # inputs = a city opened to edit its production + each newly chosen item + each Buy.
    for label_, keep in (('all', lambda t: True), ('cities10plus', lambda t: cities_by_turn.get(t, 0) >= 10)):
        idx = [i for i, p in enumerate(picks) if keep(p[0])]
        opens = {(p[0], p[1]) for p in pick_log if keep(p[0])}
        nbuys = sum(1 for p in buys if keep(p[0]))
        sub = {'turns': sum(1 for t in turns if keep(t['turn'])), 'city_opens': len(opens), 'item_picks': len(idx), 'buys': nbuys,
               'production_inputs': len(opens) + len(idx) + nbuys}
        for w in WINDOWS:
            sub[f'repeat_picks_type_w{w}'] = sum(flags[i][('type', w)] for i in idx)
        by_kind = Counter(json.loads(picks[i][3]).get('kind', '?') for i in idx if flags[i][('type', 5)])
        # A city open whose every new pick repeats one made for another city within 5 turns exists only for repetition.
        per_open = defaultdict(list)
        for i in idx:
            per_open[(picks[i][0], picks[i][1])].append(flags[i][('type', 5)])
        sub['repeat_only_opens_w5'] = sum(1 for flags_ in per_open.values() if all(flags_))
        sub['repetitive_inputs_w5'] = sub['repeat_picks_type_w5'] + sub['repeat_only_opens_w5']
        sub['repeat_picks_w5_by_kind'] = dict(by_kind)
        result['subset_' + label_] = sub

    # Same-turn groups: one item set in k >= 2 cities in one turn.
    groups = []
    for turn in sorted({p[0] for p in picks}):
        c = defaultdict(set)
        for t, city, exact, loose, _ in picks:
            if t == turn:
                c[loose].add(city)
        for loose, cities in c.items():
            if len(cities) >= 2:
                groups.append((turn, label(loose), len(cities)))
    result['same_turn_groups'] = groups
    result['same_turn_group_cities'] = sum(g[2] for g in groups)
    result['same_turn_group_savings'] = sum(g[2] - 1 for g in groups)

    # The whole queue repeated in a different city within 10 turns.
    queue_repeats = 0
    for i, (turn, city, q) in enumerate(queue_log):
        if len(q) >= 2 and any(p[2] == q and p[1] != city and 0 <= turn - p[0] <= 10 for p in queue_log[:i]):
            queue_repeats += 1
    result['whole_queue_repeats_w10'] = queue_repeats

    # Queue edits: a city edited again within 10 turns; and edits that only reorder or remove.
    quick, reorder_only, cities_edited_twice = 0, 0, 0
    for city, edits in edits_by_city.items():
        if len(edits) >= 2:
            cities_edited_twice += 1
        for (t0, _, _), (t1, q1, q0) in zip(edits, edits[1:]):
            if t1 - t0 <= 10:
                quick += 1
            if q0 is not None and not Counter(json.dumps(i, sort_keys=True) for i in q1) - Counter(json.dumps(i, sort_keys=True) for i in q0):
                reorder_only += 1
    result['reedits_within_10_turns'] = quick
    result['reorder_or_remove_only_edits'] = reorder_only
    result['cities_edited_twice_or_more'] = cities_edited_twice
    result['cities_edited'] = len(edits_by_city)

    # Most repeated items (type key) across the whole tape.
    freq = Counter(label(p[3]) for p in picks)
    result['top_items'] = freq.most_common(8)
    result['unit_orders_in_runs_of_3_plus'] = dict(unit_runs)

    # Orders per turn at empire size.
    big = [per_turn_orders[t] for t in per_turn_orders if cities_by_turn.get(t, 0) >= 10]
    small = [per_turn_orders[t] for t in per_turn_orders if 0 < cities_by_turn.get(t, 0) < 10]
    result['orders_per_turn_10plus_cities'] = (len(big), sum(big) / len(big)) if big else None
    result['orders_per_turn_under_10_cities'] = (len(small), sum(small) / len(small)) if small else None
    return result


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('saves', nargs='+')
    ap.add_argument('--json', action='store_true')
    args = ap.parse_args()
    results = [analyse(p) for p in args.saves]
    if args.json:
        print(json.dumps(results, indent=1))
        return
    for r in results:
        print(f"== {r['file']}: {r['faction']} seat {r['human_seat']}, turns {r['first_turn']}-{r['last_turn']} ({r['turns']}), complete={r['complete_history']}")
        for k, v in r.items():
            if k not in ('file', 'faction', 'human_seat', 'first_turn', 'last_turn', 'turns', 'complete_history'):
                print(f'  {k}: {v}')


if __name__ == '__main__':
    main()
