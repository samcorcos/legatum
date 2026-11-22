#!/usr/bin/env python3
"""Tiny deterministic simulator for harness tests, never real-game evidence."""
import argparse
import json
from pathlib import Path
import random
import time

NAMES = ['Default', 'Rush', 'GreedyExpand', 'Snowball', 'Economic']
p = argparse.ArgumentParser()
p.add_argument('--opening-schema', action='store_true')
p.add_argument('--players', type=int, default=8)
p.add_argument('--games', type=int, default=1)
p.add_argument('--seed', type=int, default=1)
p.add_argument('--mirror', action='store_true')
p.add_argument('--threads')
p.add_argument('--label', default='duel')
p.add_argument('--out-dir', type=Path)
p.add_argument('--seat-opening', action='append', default=[])
p.add_argument('--opening-genome', action='append', default=[])
p.add_argument('--map-size')
p.add_argument('--difficulty')
p.add_argument('--barbarians')
p.add_argument('--factions')
p.add_argument('--safety-cap', type=int)
p.add_argument('--rules-experiment')
p.add_argument('--delay', type=float, default=0)
a = p.parse_args()
if a.opening_schema:
    print(json.dumps({'version': 2, 'openings': {n: {'genes': [{'name': 'strength', 'min': 0, 'max': 100}], 'genome': [50]} for n in NAMES[1:]}}))
    raise SystemExit()
time.sleep(a.delay)
assert a.players == 2 and a.mirror
seats = {int(value.split('=')[0]): value.split('=')[1] for value in a.seat_opening}
genomes = {name: json.loads(Path(path).read_text()) for name, path in (x.split('=', 1) for x in a.opening_genome)}
def turn(standard):
    return standard


def seat_record(seat, opening, winner, outcome):
    # Strong Rush genomes deliberately abandon their identity: harness tests must
    # distinguish winning from satisfying the behavioral constraint.
    rushing = opening == 'Rush' and genomes.get(opening, [50])[0] < 80
    return {'id': seat, 'faction': ['Japan', 'Celts'][seat], 'opening': opening,
            'vp': 10 if seat == winner else 5, 'won': seat == winner and outcome == 'vp',
            'cities_at_turns': {'25': 1}, 'phase_switches': [], 'rank': 1 if seat == winner else 2,
            'wars_declared': int(rushing), 'cities_captured': 0,
            'first_opponent_war_turn': turn(40) if rushing else None,
            'first_opponent_city_pressure_turn': turn(80) if rushing else None,
            'guarded_expansion_turns': []}


def empire_record(opening, turns, seed):
    return {'early_founded': 3, 'early_peak_cities': 3,
            'early_military_cost': 100, 'development_turn': min(turn(240), turns),
            'cities': 4, 'science': 24, 'production': 40, 'gold': 18, 'markets': 2}


games = []
for i in range(a.games * 2):
    seed, mirrored = a.seed + i // 2, bool(i % 2)
    openings = [seats[1], seats[0]] if mirrored else [seats[0], seats[1]]
    strengths = [genomes.get(o, [50])[0] for o in openings]
    chance = .5 + (strengths[0] - strengths[1]) / 150
    winner = int(random.Random(seed).random() >= chance)
    outcome = 'stalled' if a.safety_cap and a.safety_cap < 10 else 'vp'
    turns = turn(240) if outcome == 'vp' else a.safety_cap
    games.append({'index': i, 'seed': str(seed), 'mirrored': mirrored, 'turns': turns,
                  'outcome': outcome, 'victory_type': 'vp' if outcome == 'vp' else None,
                  'winner_players': [winner] if outcome == 'vp' else [], 'panic': None, 'wall_ms': 1,
                  'players': [seat_record(seat, opening, winner, outcome) for seat, opening in enumerate(openings)],
                  'telemetry': {'strategy': {'early_end_turn': turn(120), 'development_end_turn': turn(240),
                                'empires': {str(seat): empire_record(opening, turns, seed)
                                            for seat, opening in enumerate(openings)}}}})
a.out_dir.mkdir(parents=True, exist_ok=True)
(a.out_dir / ('test-' + a.label + '.json')).write_text(json.dumps({'games': games}))
