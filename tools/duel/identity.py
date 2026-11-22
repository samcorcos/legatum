"""Opening identity from native observations; no policy labels count as evidence."""
from __future__ import annotations

from collections import Counter
import json
from pathlib import Path


def load_rules():
    return json.loads(Path(__file__).with_name('identity.json').read_text())


def standard_turn(turn):
    return turn


def seat_gate(game, seat, rules=None):
    """Return True/False/None (unknown). Null event turns mean never, absent fields unknown."""
    rules = load_rules() if rules is None else rules
    opening = seat['opening']
    gate = rules['openings'][opening]
    if opening == 'Default':
        return {'passed': None, 'checks': {}, 'missing': []}
    checks = {}
    strategy = (game.get('telemetry') or {}).get('strategy') or {}
    empire = strategy.get('empires', {}).get(str(seat['id']), {})

    def minimum(name, value, threshold):
        checks[name] = None if value is None else value >= threshold

    def maximum(name, value, threshold):
        checks[name] = None if value is None else value <= threshold

    def early():
        # Native counters stop at this exact Standard horizon. Never silently
        # interpret a longer collection window as proof of an earlier deadline.
        checks['early_horizon'] = (True if strategy.get('early_end_turn') ==
                                   standard_turn(rules['early_turn']) else None)
        return empire if checks['early_horizon'] else {}

    def development():
        horizon = standard_turn(rules['development_turn'])
        turn = empire.get('development_turn')
        checks['development_snapshot'] = (True if strategy.get('development_end_turn') == horizon
                                          and turn is not None and 0 < turn <= min(horizon, game['turns'])
                                          else None)
        return empire if checks['development_snapshot'] else {}

    if opening == 'Rush':
        deadline = standard_turn(gate['by_turn'])
        for field in ('first_opponent_war_turn', 'first_opponent_city_pressure_turn'):
            checks[field] = (None if field not in seat else
                             seat[field] is not None and 0 < seat[field] <= deadline)
        # This establishes failure even in old reports that lack the two timestamps.
        minimum('declared_war', seat.get('wars_declared'), 1)
    elif opening == 'GreedyExpand':
        observed = early()
        minimum('early_founded', observed.get('early_founded'), gate['min_founded'])
        maximum('early_military_cost', observed.get('early_military_cost'), gate['max_military_cost'])
    elif opening == 'Snowball':
        observed = early()
        minimum('early_founded', observed.get('early_founded'), gate['min_founded'])
        observed = development()
        minimum('cities', observed.get('cities'), gate['min_cities'])
        minimum('production', observed.get('production'), gate['min_production'])
    elif opening == 'Economic':
        observed = development()
        minimum('markets', observed.get('markets'), gate['min_markets'])
        minimum('gold', observed.get('gold'), gate['min_gold'])
    missing = [name for name, passed in checks.items() if passed is None]
    passed = False if False in checks.values() else None if missing else True
    return {'passed': passed, 'checks': checks, 'missing': missing}


def report(games, names, rules=None):
    rules = load_rules() if rules is None else rules
    outcomes = {name: [] for name in names}
    for game in games:
        for seat in game['players']:
            if seat['opening'] in outcomes:
                outcomes[seat['opening']].append(seat_gate(game, seat, rules))
    openings = {}
    for name, rows in outcomes.items():
        total = len(rows)
        applicable = name != 'Default'
        passes = sum(row['passed'] is True for row in rows)
        unknown = sum(row['passed'] is None for row in rows) if applicable else 0
        rate = passes / total if total and applicable else None
        required = rules['required_share'] if applicable else None
        passed = rate is not None and rate >= required if applicable else None
        openings[name] = {
            'games': total, 'passed_games': passes,
            'failed_games': total - passes - unknown if applicable else 0,
            'unknown_games': unknown,
            'missing_telemetry_games': sum(bool(row['missing']) for row in rows),
            'missing_fields': dict(Counter(field for row in rows for field in row['missing'])),
            'failed_checks': dict(Counter(field for row in rows for field, ok in row['checks'].items()
                                          if ok is False)),
            'pass_rate': rate,
            'pass_rate_upper_bound': (passes + unknown) / total if total and applicable else None,
            'required_share': required, 'passed': passed,
            'status': ('PASS' if passed else 'FAIL') if applicable else 'N/A',
            'comment': rules['openings'][name]['comment'],
        }
    return {'version': rules['version'], 'rules': rules, 'openings': openings,
            'denominator': 'seat-games, including stalls, losses, draws and both seats of self-play; '
                           'missing evidence never passes; Default is an unconstrained anchor'}


def render(report):
    lines = ['', '## Identity', '', report['denominator'] + '.', '',
             '| Opening | Passes / games | Pass rate | Required | Missing telemetry | Gate |',
             '| --- | --- | --- | --- | --- | --- |']
    for name, row in report['openings'].items():
        percent = lambda value: 'N/A' if value is None else f'{100 * value:.1f}%'
        lines.append(f"| {name} | {row['passed_games']} / {row['games']} | {percent(row['pass_rate'])} | "
                     f"{percent(row['required_share'])} | {row['missing_telemetry_games']} | {row['status']} |")
    lines += ['', 'Rates count proven passes only; missing telemetry can make FAIL inconclusive. '
              'JSON includes upper bounds, missing fields and individual failed checks.', '']
    lines.extend(f"{name}: {row['comment']}" for name, row in report['openings'].items())
    return lines
