#!/usr/bin/env python3
"""Mirrored 1v1 opening round robin, using only the Python standard library."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from contextlib import contextmanager
import fcntl
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import threading
import time

if __package__ in (None, ''):
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.duel.equilibrium import nash
from tools.duel.mathlib import cycles, direction_p, holm, interval
from tools.duel import identity

OPENINGS = ['Default', 'Rush', 'GreedyExpand', 'Snowball', 'Economic']
COUNTERS = [('Rush', 'GreedyExpand'), ('GreedyExpand', 'Snowball'),
            ('Snowball', 'Economic')]


def positive(value):
    value = int(value)
    if value < 1:
        raise argparse.ArgumentTypeError('must be positive')
    return value


def cpu_limit(workers=None):
    cap = max(1, (os.cpu_count() or 1) - 2)
    return min(cap, workers) if workers is not None else cap


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def file_hash(path):
    with open(path, 'rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text())


def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_suffix(path.suffix + '.tmp')
    temp.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n')
    temp.replace(path)


def append_json(path, value):
    with Path(path).open('a') as stream:
        stream.write(json.dumps(value, sort_keys=True, allow_nan=False) + '\n')
        stream.flush()
        os.fsync(stream.fileno())


@contextmanager
def locked(path):
    path.mkdir(parents=True, exist_ok=True)
    with (path / '.lock').open('w') as stream:
        try:
            fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ValueError(f'another writer owns {path}') from error
        yield


def ensure_config(path, config, *, defaults=None):
    if path.exists() and {**(defaults or {}), **read_json(path)} != config:
        raise ValueError(f'configuration changed: {path}; use a fresh run directory')
    write_json(path, config)


def canonical_genome(opening, genome, schema):
    if genome is None:
        return None
    if opening == 'Default':
        raise ValueError('Default has no opening genome')
    spec = schema['openings'][opening]
    genes, default = spec['genes'], spec['genome']
    if isinstance(genome, dict):
        unknown = set(genome) - {g['name'] for g in genes}
        if unknown:
            raise ValueError(f'unknown {opening} genes: {sorted(unknown)}')
        genome = [genome.get(g['name'], v) for g, v in zip(genes, default)]
    if not isinstance(genome, list) or len(genome) != len(genes):
        raise ValueError(f'wrong genome length for {opening}')
    if any(type(v) is not int or not g['min'] <= v <= g['max'] for g, v in zip(genes, genome)):
        raise ValueError(f'noninteger or out-of-bounds genome for {opening}')
    return genome[:]


def load_members(args, schema):
    raw = read_json(args.members) if args.members else [{'opening': o, 'genome': None} for o in args.openings]
    if not isinstance(raw, list) or not raw:
        raise ValueError('members must be a nonempty list')
    members = []
    for item in raw:
        opening, genome = item['opening'], item.get('genome')
        if opening not in OPENINGS or any(m['opening'] == opening for m in members):
            raise ValueError('unknown or duplicate opening')
        if isinstance(genome, str):
            genome = read_json(args.members.parent / genome)
        members.append({'opening': opening, 'genome': canonical_genome(opening, genome, schema)})
    seen = set()
    for assignment in args.opening_genome:
        opening, sep, path = assignment.partition('=')
        if not sep or opening in seen or opening not in {m['opening'] for m in members}:
            raise ValueError('expected one Opening=path override per member')
        seen.add(opening)
        next(m for m in members if m['opening'] == opening)['genome'] = canonical_genome(opening, read_json(path), schema)
    return members


def nice_command(command):
    # Never raise priority for a process already running at nice 10 or below.
    current = os.getpriority(os.PRIO_PROCESS, 0)
    return (['nice', '-n', str(10 - current)] if current < 10 else []) + command


class Simulator:
    def __init__(self, args, root):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        source = args.sim.expanduser().resolve()
        self.binary_hash = file_hash(source)
        frozen = self.root / ('simulator.py' if source.suffix == '.py' else 'simulator')
        if frozen.exists():
            if file_hash(frozen) != self.binary_hash:
                raise ValueError('simulator changed; use a fresh run directory')
        else:
            shutil.copy2(source, frozen)
        self.command = [sys.executable, str(frozen)] if frozen.suffix == '.py' else [str(frozen)]
        self.workers = cpu_limit(args.workers)
        self.timeout = args.timeout
        self.identity_rules = identity.load_rules()
        self.settings = {'map_size': args.map_size, 'barbarians': args.barbarians,
                         'difficulty': args.difficulty, 'factions': args.factions, 'safety_cap': args.safety_cap,
                         'rules_experiment': getattr(args, 'rules_experiment', None)}
        self._children = set()
        self._lock = threading.Lock()
        self._cancel = threading.Event()
        self.effective_nice = max(10, os.getpriority(os.PRIO_PROCESS, 0))
        self.schema = json.loads(self.process(['--opening-schema']))

    def cancel(self):
        self._cancel.set()
        with self._lock:
            for process in self._children:
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass

    def process(self, arguments):
        with self._lock:
            if self._cancel.is_set():
                raise RuntimeError('run cancelled')
            process = subprocess.Popen(nice_command(self.command + arguments), stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, text=True, start_new_session=True)
            self._children.add(process)
        try:
            stdout, stderr = process.communicate(timeout=self.timeout)
            if process.returncode:
                raise RuntimeError(f'simulator exit {process.returncode}: {stderr[-4000:]}')
            if 'setpriority: Operation not permitted' in stderr:
                self.effective_nice = os.getpriority(os.PRIO_PROCESS, 0)
            return stdout
        except BaseException:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.communicate()
            raise
        finally:
            with self._lock:
                self._children.discard(process)

    def batch(self, a, b, seed, count):
        if a['opening'] == b['opening'] and a['genome'] != b['genome']:
            raise ValueError('simulator keys genomes by opening; different versions need different opponents')
        key = digest([self.binary_hash, self.settings, a, b, seed, count])
        cache = self.root / 'cache' / (key + '.json.gz')
        if cache.exists():
            with gzip.open(cache, 'rt') as stream:
                return json.load(stream)
        with tempfile.TemporaryDirectory(prefix='duel-', dir=self.root) as temporary:
            directory = Path(temporary)
            args = ['--players', '2', '--mirror', '--games', str(count), '--seed', str(seed),
                    '--threads', '1', '--label', 'duel', '--out-dir', str(directory),
                    '--seat-opening', f"0={a['opening']}", '--seat-opening', f"1={b['opening']}"]
            for name, value in self.settings.items():
                if value is not None:
                    args += ['--' + name.replace('_', '-'), str(value)]
            for member in {m['opening']: m for m in [a, b]}.values():
                if member['genome'] is not None:
                    path = directory / (member['opening'] + '.json')
                    write_json(path, member['genome'])
                    args += ['--opening-genome', member['opening'] + '=' + str(path)]
            self.process(args)
            reports = list(directory.glob('*-duel.json'))
            if len(reports) != 1:
                raise ValueError('simulator did not write exactly one duel report')
            games = read_json(reports[0])['games']
            validate_games(games, a, b, seed, count)
            # Retain every original seat field. Large native telemetry remains in the cache.
            for game in games:
                game['duel_members'] = [a['opening'], b['opening']]
                game['genome_hashes'] = [digest(a['genome']), digest(b['genome'])]
        cache.parent.mkdir(exist_ok=True)
        with gzip.open(cache.with_suffix('.tmp'), 'wt') as stream:
            json.dump(games, stream)
        cache.with_suffix('.tmp').replace(cache)
        return games

    def evaluate(self, jobs):
        """Jobs are (member A, member B, first seed, seed count), all mirrored."""
        result = []
        with ThreadPoolExecutor(max_workers=self.workers) as executor:
            futures = [executor.submit(self.batch, *job) for job in jobs]
            try:
                for future in as_completed(futures):
                    result.extend(future.result())
            except BaseException:
                self.cancel()
                for future in futures:
                    future.cancel()
                raise
        return sorted(result, key=lambda g: (g['duel_members'], int(g['seed']), g.get('mirrored', False)))


def validate_games(games, a, b, first, count):
    if len(games) != 2 * count:
        raise ValueError('missing mirrored games')
    by_key = {}
    for game in games:
        key = (int(game['seed']), game.get('mirrored', False))
        if key in by_key:
            raise ValueError('duplicate seed/orientation')
        by_key[key] = game
        if game.get('panic') or game['outcome'] == 'panic':
            raise ValueError('panicked game is not evidence')
        if game['outcome'] not in ('vp', 'conquest', 'draw', 'stalled'):
            raise ValueError('unexpected outcome (turn limits are not duel evidence)')
        seats = sorted(game['players'], key=lambda p: p['id'])
        expected = [b['opening'], a['opening']] if key[1] else [a['opening'], b['opening']]
        if [p['id'] for p in seats] != [0, 1] or [p['opening'] for p in seats] != expected:
            raise ValueError('wrong seats/openings')
        if game['turns'] < 1 or 'victory_type' not in game:
            raise ValueError('missing turns/victory_type')
        for seat in seats:
            for field in ('vp', 'won', 'faction', 'cities_at_turns', 'phase_switches'):
                if field not in seat:
                    raise ValueError(f'missing seat field {field}')
        winners = [p['id'] for p in seats if p['won']]
        if winners != game['winner_players'] or len(winners) != (game['outcome'] in ('vp', 'conquest')):
            raise ValueError('inconsistent winner record')
        if game['victory_type'] != {'vp': 'vp', 'conquest': 'elimination'}.get(game['outcome']):
            raise ValueError('inconsistent victory type')
    expected_keys = {(seed, mirror) for seed in range(first, first + count) for mirror in (False, True)}
    if set(by_key) != expected_keys:
        raise ValueError('wrong seeds/orientations')
    for seed in range(first, first + count):
        factions = [[p['faction'] for p in sorted(by_key[seed, m]['players'], key=lambda p: p['id'])] for m in (False, True)]
        if factions[0] != factions[1]:
            raise ValueError('mirror changed the factions')


def focal_score(game, seat):
    if game['outcome'] in ('stalled', 'draw'):
        return 0.5
    return float(next(p for p in game['players'] if p['id'] == seat)['won'])


def map_scores(games):
    groups = {}
    for game in games:
        groups.setdefault(game['seed'], []).append(game)
    result, dropped = {}, 0
    for seed, pair in groups.items():
        if len(pair) != 2 or {g.get('mirrored', False) for g in pair} != {False, True}:
            raise ValueError('each map needs both mirror orientations')
        if any(g['outcome'] == 'stalled' for g in pair):
            dropped += 1
            continue
        result[seed] = statistics.mean(focal_score(g, int(g.get('mirrored', False))) for g in pair)
    return result, dropped


def validity_report(summary):
    """Apply the validity criteria to the existing Holm-significant edges."""
    names, cells = summary['openings'], summary['cells']
    openings, reasons = {}, []
    for name in names:
        others = [other for other in names if other != name]
        rates = [cells[name][other]['rate'] for other in others]
        mean = statistics.mean(rates) if rates and all(rate is not None for rate in rates) else None
        beats = [other for other in others if cells[name][other]['significant']]
        beaten_by = [other for other in others if cells[other][name]['significant']]
        criteria = {'beats_at_least_one': bool(beats),
                    'does_not_beat_all': bool(others) and len(beats) < len(others),
                    'mean_score_at_least_0_35': mean is not None and mean >= .35}
        failures = []
        if not criteria['beats_at_least_one']:
            failures.append('no significant wins against another opening')
        if not criteria['does_not_beat_all']:
            failures.append('significantly beats every other opening' if others else 'no other openings')
        if not criteria['mean_score_at_least_0_35']:
            failures.append('mean score unavailable (incomplete matchups)' if mean is None
                            else f'mean score {mean:.3f} is below 0.35')
        openings[name] = {'mean_score': mean, 'significantly_beats': beats,
                          'significantly_beaten_by': beaten_by, 'criteria': criteria,
                          'passed': all(criteria.values()), 'reasons': failures}
        reasons.extend(f'{name}: {failure}' for failure in failures)
    return {'openings': openings, 'mean_score_threshold': .35,
            'significant_cycles': summary['cycles'], 'passed': bool(names) and not reasons,
            'verdict': 'VALID' if names and not reasons else 'NOT VALID',
            'reasons': reasons if names else ['no openings']}


def summarize(games, names, identity_rules=None):
    cells = {a: {} for a in names}
    tests = []
    for i, a in enumerate(names):
        for b in names[i:]:
            pair = [g for g in games if g['duel_members'] == [a, b]]
            scores, dropped = map_scores(pair)
            values = list(scores.values())
            cell = {'rate': statistics.mean(values) if values else None, 'ci95': interval(values),
                    'maps': len(values), 'games': 2 * len(values), 'attempted_games': len(pair),
                    'stalled_maps_dropped': dropped,
                    'stalled_games': sum(g['outcome'] == 'stalled' for g in pair),
                    'draws': sum(g['outcome'] == 'draw' for g in pair),
                    'wins': sum(focal_score(g, int(g.get('mirrored', False))) == 1 for g in pair if g['seed'] in scores),
                    'significant': False}
            cells[a][b] = cell
            if a != b:
                cells[b][a] = {**cell, 'rate': 1 - cell['rate'] if values else None,
                               'ci95': [1 - cell['ci95'][1], 1 - cell['ci95'][0]] if values else [None, None],
                               'wins': sum(focal_score(g, int(g.get('mirrored', False))) == 0 for g in pair if g['seed'] in scores)}
                p = direction_p(values)
                tests.append((a, b, p))
    for (a, b, p), adjusted in zip(tests, holm([t[2] for t in tests])):
        for x, y in [(a, b), (b, a)]:
            cell = cells[x][y]
            cell.update(p_value=p, p_holm=adjusted)
            cell['significant'] = cell['maps'] >= 8 and cell['rate'] > .5 and adjusted < .05
    matrix = [[cells[a][b]['rate'] for b in names] for a in names]
    equilibrium = nash(matrix) if all(v is not None for row in matrix for v in row) else {'converged': False, 'reason': 'missing completed maps'}
    if equilibrium.get('converged'):
        equilibrium['mixture'] = dict(zip(names, equilibrium['weights']))
    counters = []
    for a, b in COUNTERS:
        if a in names and b in names:
            counters.append({'winner': a, 'loser': b, 'rate': cells[a][b]['rate'],
                             'status': 'supported' if cells[a][b]['significant'] else
                             'reversed' if cells[b][a]['significant'] else 'not_established'})
    summary = {'openings': names, 'cells': cells, 'matrix': matrix, 'equilibrium': equilibrium,
            'cycles': cycles(names, cells), 'intended_counters': counters, 'games': len(games),
            'stalled_games': sum(g['outcome'] == 'stalled' for g in games),
            'mean_turns': statistics.mean(g['turns'] for g in games) if games else None,
            'ci_method': '95% finite-sample Hoeffding bounds on independent mirrored map means; stalled maps dropped',
            'cycle_method': 'exact two-sided map sign tests (Hoeffding mean test for fractional draws), Holm correction across all unordered off-diagonal pairs; >=8 maps',
            'score': 'win=1, loss=0, completed draw=0.5; diagonal is self-play symmetry'}
    summary['validity'] = validity_report(summary)
    summary['identity'] = identity.report(games, names, identity_rules)
    return summary


def render(summary):
    names = summary['openings']
    lines = ['Row opening win percentage (95% CI); * = significant directed edge.', '',
             '| Opening | ' + ' | '.join(names) + ' |', '| --- | ' + ' | '.join(['---'] * len(names)) + ' |']
    for a in names:
        values = []
        for b in names:
            c = summary['cells'][a][b]
            values.append('unobserved' if c['rate'] is None else f"{100*c['rate']:.1f} [{100*c['ci95'][0]:.1f}, {100*c['ci95'][1]:.1f}]" + ('*' if c['significant'] else ''))
        lines.append('| ' + a + ' | ' + ' | '.join(values) + ' |')
    lines += ['', summary['ci_method'] + '.', summary['cycle_method'] + '.',
              f"Games: {summary['games']}; stalled games: {summary['stalled_games']}; mean turns: {summary['mean_turns']}.",
              '', 'Equilibrium: ' + json.dumps(summary['equilibrium'], sort_keys=True),
              '', 'Significant cycles: ' + (', '.join(' > '.join(c) for c in summary['cycles']) or 'none'), '']
    lines.extend(f"{c['winner']} > {c['loser']}: {c['status']}." for c in summary['intended_counters'])
    validity = summary['validity']
    lines += ['', '## Validity', '',
              validity['verdict'] + ': ' + ('; '.join(validity['reasons']) or 'all openings meet all three criteria') + '.', '',
              '| Opening | Mean vs others | Significantly beats | Significantly beaten by | Beats at least one | Does not beat all | Mean >= 0.35 |',
              '| --- | --- | --- | --- | --- | --- | --- |']
    for name, result in validity['openings'].items():
        mean = 'unobserved' if result['mean_score'] is None else f"{result['mean_score']:.3f}"
        lines.append('| ' + ' | '.join([name, mean, ', '.join(result['significantly_beats']) or 'none',
                     ', '.join(result['significantly_beaten_by']) or 'none',
                     *('PASS' if passed else 'FAIL' for passed in result['criteria'].values())]) + ' |')
    lines += identity.render(summary['identity'])
    return '\n'.join(lines) + '\n'


def round_robin(sim, members, seeds, first, directory):
    directory.mkdir(parents=True, exist_ok=True)
    names = [m['opening'] for m in members]
    jobs = [(a, b, first + offset, min(10, seeds-offset))
            for i, a in enumerate(members) for b in members[i:] for offset in range(0, seeds, 10)]
    start = time.monotonic()
    games = sim.evaluate(jobs)
    summary = summarize(games, names, sim.identity_rules)
    summary['wall_seconds'] = time.monotonic() - start
    summary['seeds_per_pair'] = seeds
    summary['first_seed'] = first
    write_json(directory / 'summary.json', summary)
    (directory / 'matrix.md').write_text(render(summary))
    with gzip.open(directory / 'games.jsonl.gz', 'wt') as stream:
        for game in games:
            stream.write(json.dumps(game, separators=(',', ':')) + '\n')
    return summary


def add_args(parser):
    parser.add_argument('--sim', type=Path, default=Path('target/release/legatum-sim'))
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--seeds', type=positive, default=100, help='maps per unordered pair; two games per seed')
    parser.add_argument('--seed', type=int, default=1930001)
    parser.add_argument('--workers', type=positive)
    parser.add_argument('--timeout', type=positive, default=900, help='seconds per native batch')
    parser.add_argument('--map-size', choices=['small', 'medium', 'large'], default='small')
    parser.add_argument('--barbarians', choices=['true', 'false'], default='true')
    parser.add_argument('--difficulty', choices=['easy', 'normal', 'hard'], default='hard')
    parser.add_argument('--factions', help='two factions in seat order; seeded draw by default')
    parser.add_argument('--safety-cap', type=positive)
    parser.add_argument('--rules-experiment', help='comma-separated legatum-sim rule levers, passed through')
    parser.add_argument('--openings', nargs='+', choices=OPENINGS, default=OPENINGS)
    parser.add_argument('--members', type=Path, help='active-members.json; null genomes use built-ins')
    parser.add_argument('--opening-genome', action='append', default=[], help='Opening=path, vector or named genes')


def terminate_signal(signum, frame):
    raise KeyboardInterrupt(f'interrupted by signal {signum}')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    add_args(parser)
    args = parser.parse_args(argv)
    if not 0 <= args.seed <= 2**64 - args.seeds:
        parser.error('seed range must fit u64 without wrapping')
    args.run = args.run.resolve()
    with locked(args.run):
        sim = Simulator(args, args.run)
        members = load_members(args, sim.schema)
        config = {'binary_sha256': sim.binary_hash, 'settings': sim.settings, 'members': members,
                  'identity_rules': sim.identity_rules,
                  'seed': args.seed, 'seeds': args.seeds, 'workers': sim.workers,
                  'nice': sim.effective_nice}
        ensure_config(args.run / 'config.json', config)
        write_json(args.run / 'members.json', members)
        summary = round_robin(sim, members, args.seeds, args.seed, args.run)
        append_json(args.run / 'matrix-history.jsonl', summary)
        print(render(summary), end='')
    return 0


if __name__ == '__main__':
    signal.signal(signal.SIGTERM, terminate_signal)
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
    except (ValueError, RuntimeError, OSError, subprocess.TimeoutExpired) as error:
        raise SystemExit(f'duel: {error}')
