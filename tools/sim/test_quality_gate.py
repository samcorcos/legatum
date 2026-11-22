"""Small statistical/provenance tests; no simulation or AWS access."""
import contextlib
from concurrent.futures import ThreadPoolExecutor
import copy
import io
import os
import importlib.util
from pathlib import Path
import random
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('quality_gate', Path(__file__).with_name('quality_gate.py'))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class GateTests(unittest.TestCase):
    def test_gene_arms_force_old_behavior_in_measured_seat(self):
        arms = gate.gene_arms(['coalition_push', 'walls_garrison'])['arms']
        self.assertEqual([(a['label'], a['mode']) for a in arms],
                         [('ctl', 'all'), ('rivals', 'seat'), ('all', 'all')])
        self.assertEqual(arms[0]['params']['decisions'],
                         {'coalition_push': 0, 'walls_garrison': 0})
        self.assertEqual(arms[1]['params']['decisions'], arms[0]['params']['decisions'])
        self.assertNotIn('decisions', arms[2]['params'])
        for faction in arms[0]['params']['faction_overrides']:
            self.assertEqual(arms[0]['params']['faction_overrides'][faction],
                             {'coalition_push': 0, 'walls_garrison': 0})
            self.assertEqual(arms[1]['params']['faction_overrides'][faction],
                             {'coalition_push': 0, 'walls_garrison': 0})
            self.assertEqual(arms[2]['params']['faction_overrides'][faction]['coalition_push'], 1)

    def test_builtin_fixture_matches_generated_arms(self):
        fixture = Path(__file__).with_name('fixtures') / 'gene-arms-builtins.json'
        import json
        expected = gate.gene_arms(['swiss_playbook', 'celts_playbook', 'aztecs_playbook',
                                   'india_playbook',
                                   'walls_garrison', 'army_war_extra'])
        self.assertEqual(json.loads(fixture.read_text()), expected)

    def setUp(self):
        self.p = {'maps':list('abcdefgh'), 'looks':[128,256,512],
                  'alpha':0.05, 'family_size':5, 'floor':-0.015}

    def record(self, wins):
        return {'protocol_id':gate.digest(gate.canonical(self.p)), 'maps':self.p['maps'],
                'salts':len(wins[0]), 'wins':wins,
                'game_hashes':[['0'*64]*len(wins[0]) for _ in wins],
                'binary_sha256':'d'*64, 'commit':'a'*40}

    def test_clear_harm_stops_early(self):
        b = self.record([[int(i%2 == 0) for i in range(128)] for _ in range(8)])
        c = self.record([[0]*128 for _ in range(8)])
        r = gate.compare(b,c,self.p)
        self.assertEqual(r['decision'],'clear-harm')
        self.assertLess(r['simultaneous_interval'][1],-.015)
        self.assertGreater(r['salt_cluster_se'], r['stratified_se'])

    def test_improvement_stops_early(self):
        b = self.record([[0]*128 for _ in range(8)])
        c = self.record([[int(i%2 == 0) for i in range(128)] for _ in range(8)])
        self.assertEqual(gate.compare(b,c,self.p)['decision'],'noninferior')

    def test_near_floor_continues_then_ships_with_uncertainty(self):
        rng = random.Random(12)
        b = [[int(rng.random()<.5) for _ in range(512)] for _ in range(8)]
        c = [[int(rng.random()<.485) for _ in range(512)] for _ in range(8)]
        control = self.record(b)
        for n in (128,256,512):
            candidate = self.record([r[:n] for r in c])
            result = gate.compare(control,candidate,self.p)
            self.assertEqual(result['decision'], 'inconclusive' if n==512 else 'continue')
            self.assertGreater(result['critical_z'],2.9)
            self.assertAlmostEqual(result['mde80']['1024']/result['mde80']['4096'],2)

    def test_designer_decided_rule_keeps_the_permissive_ship_monitor(self):
        self.p['policy'] = 'designer-rule-ship-unless-clear-harm'
        rng = random.Random(12)
        b = [[int(rng.random()<.5) for _ in range(512)] for _ in range(8)]
        c = [[int(rng.random()<.485) for _ in range(512)] for _ in range(8)]
        result = gate.compare(self.record(b), self.record(c), self.p)
        self.assertEqual(result['decision'], 'ship-monitor')

    def role_pair(self, direction):
        """Control 50% seat wins; candidate +-25 points (paired through shared salts)."""
        b = [[int(i%2 == 0) for i in range(128)] for _ in range(8)]
        c = [[1]*128 if direction > 0 else [0]*128 for _ in range(8)]
        return self.record(b), self.record(c)

    def test_seat_role_higher_wins_is_stronger(self):
        b, up = self.role_pair(+1)
        r = gate.compare(b, up, self.p, role='seat')
        self.assertGreater(r['delta'], 0)
        self.assertEqual(r['strength_effect'], r['delta'])
        self.assertEqual(r['decision'], 'noninferior')
        b, down = self.role_pair(-1)
        r = gate.compare(b, down, self.p, role='seat')
        self.assertEqual((r['decision'], r['strength_effect'] < 0), ('clear-harm', True))

    def test_opponent_role_lower_seat_wins_is_stronger(self):
        b, up = self.role_pair(+1)      # seat wins rise: weaker opponents
        r = gate.compare(b, up, self.p, role='opponents')
        self.assertGreater(r['delta'], 0)                 # raw seat delta is preserved
        self.assertEqual(r['strength_effect'], -r['delta'])
        self.assertEqual(r['decision'], 'clear-harm')     # the old label here was 'noninferior'
        self.assertEqual(r['strength_interval'], [-r['simultaneous_interval'][1], -r['simultaneous_interval'][0]])
        b, down = self.role_pair(-1)    # seat wins fall: stronger opponents
        r = gate.compare(b, down, self.p, role='opponents')
        self.assertLess(r['delta'], 0)
        self.assertGreater(r['strength_effect'], 0)
        self.assertEqual(r['decision'], 'noninferior')

    def test_all_seats_role_has_no_strength_verdict(self):
        b, up = self.role_pair(+1)
        r = gate.compare(b, up, self.p, role='all-seats')
        self.assertEqual((r['decision'], r['strength_effect']), ('role-mixed', None))
        with self.assertRaisesRegex(ValueError, 'unknown candidate role'):
            gate.compare(b, up, self.p, role='both')

    def test_placeholder_provenance_rejected(self):
        for field, value in (('commit', '0'*40), ('binary_sha256', '0'*64), ('commit', ''), ('commit', 'unknown'),
                             ('binary_sha256', 'z'*64), ('commit', 'a'*8)):
            b = self.record([[0]*128 for _ in range(8)])
            b[field] = value
            with self.subTest(field=field, value=value):
                with self.assertRaisesRegex(ValueError, 'placeholder'):
                    gate.compare(b, b, self.p)

    def test_wrong_protocol_rejected(self):
        b = self.record([[0]*128 for _ in range(8)])
        c = copy.deepcopy(b); c['protocol_id']='bad'
        with self.assertRaisesRegex(ValueError,'protocol mismatch'): gate.compare(b,c,self.p)

    def test_analysis_refuses_another_control_source(self):
        self.p['control_commit'] = 'b'*40
        b = self.record([[0]*128 for _ in range(8)])
        with self.assertRaisesRegex(ValueError,'wrong frozen control'):
            gate.compare(b,b,self.p)

    def test_incomplete_pairing_rejected(self):
        b = self.record([[0]*128 for _ in range(8)])
        c = copy.deepcopy(b); c['wins'][0].pop()
        with self.assertRaisesRegex(ValueError,'incomplete'): gate.compare(b,c,self.p)

    def test_different_salt_block_rejected(self):
        b = self.record([[0]*128 for _ in range(8)])
        c = copy.deepcopy(b); c['salt_start'] = 129
        with self.assertRaisesRegex(ValueError,'salt range'): gate.compare(b,c,self.p)

    def test_maps_and_unplanned_looks_rejected(self):
        b = self.record([[0]*128 for _ in range(8)])
        c = copy.deepcopy(b); c['maps']=list(reversed(c['maps']))
        with self.assertRaisesRegex(ValueError,'map order'): gate.compare(b,c,self.p)
        c = self.record([[0]*64 for _ in range(8)])
        with self.assertRaisesRegex(ValueError,'unplanned'): gate.compare(b,c,self.p)

    def test_identical_few_pairs_does_not_claim_zero_uncertainty(self):
        b = self.record([[0]*128 for _ in range(8)])
        r = gate.compare(b,b,self.p)
        self.assertEqual(r['decision'],'continue')
        self.assertEqual(r['simultaneous_interval'],[-1,1])
        self.assertEqual(r['ci95_descriptive'],[-1,1])
        self.assertIsNone(r['mde80'])

    def test_constant_complete_regression_cannot_ship_on_zero_variance(self):
        b = self.record([[1]*128 for _ in range(8)])
        c = self.record([[0]*128 for _ in range(8)])
        r = gate.compare(b,c,self.p)
        self.assertEqual(r['decision'],'clear-harm')
        self.assertEqual(r['simultaneous_interval'][0],-1)
        self.assertGreater(r['simultaneous_interval'][1],r['ci95_descriptive'][1])
        self.assertLess(r['simultaneous_interval'][1],self.p['floor'])
        self.assertIsNone(r['mde80'])
        self.assertEqual(gate.compare(c,b,self.p)['decision'],'noninferior')

    def test_map_strata_prevent_cancellation_of_uncertainty(self):
        b = self.record([[i%2 for i in range(128)] for _ in range(8)])
        c = self.record([[(i+j)%2 for i in range(128)] for j in range(8)])
        r = gate.compare(b,c,self.p)
        self.assertGreaterEqual(r['se'],r['stratified_se'])
        self.assertGreaterEqual(r['se'],r['salt_cluster_se'])


    def candidate_inputs(self, directory):
        self.p['control_commit'] = 'a'*40
        self.p['arms'] = gate.ARMS
        gate.write(Path(directory)/'protocol.json', self.p)
        gate.write(Path(directory)/'control/result.json', self.record([[0]*512 for _ in range(8)]))
        gate.write(Path(directory)/'repeat/result.json', self.record([[0]*128 for _ in range(8)]))
        return SimpleNamespace(directory=directory, commit='candidate', id='candidate-1', role='candidate')

    def test_preparation_refuses_failed_replication(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.candidate_inputs(directory)
            repeat = gate.read(Path(directory)/'repeat/result.json')
            repeat['game_hashes'][0][0] = 'f'*64
            gate.write(Path(directory)/'repeat/result.json', repeat)
            with patch.object(gate.subprocess, 'check_output', return_value='b'*40):
                with self.assertRaisesRegex(ValueError,'repeatability'):
                    gate.prepare(args)

    def test_preparation_refuses_another_control(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.candidate_inputs(directory)
            control = gate.read(Path(directory)/'control/result.json')
            control['commit'] = 'c'*40
            gate.write(Path(directory)/'control/result.json', control)
            with patch.object(gate.subprocess, 'check_output', return_value='b'*40):
                with self.assertRaisesRegex(ValueError,'complete frozen control'):
                    gate.prepare(args)

    def test_family_budget_cannot_silently_expand(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.candidate_inputs(directory)
            gate.write(Path(directory)/'candidates.json', {f'old-{i}':'b'*40 for i in range(5)})
            with patch.object(gate.subprocess, 'check_output', return_value='b'*40):
                with self.assertRaisesRegex(ValueError,'family exhausted'):
                    gate.prepare(args)

    def test_existing_job_cannot_be_repurposed(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.candidate_inputs(directory)
            gate.write(Path(directory)/'candidate-1.spec.json', {'different':'job'})
            with patch.object(gate.subprocess, 'check_output', return_value='b'*40):
                with self.assertRaisesRegex(ValueError,'different inputs'):
                    gate.prepare(args)


    def test_worker_appends_disjoint_fresh_salts_and_cleans_environment(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root/'src/target/sim/examples/hard_vs_human'
            binary.parent.mkdir(parents=True); binary.write_text('fake executable')
            maps = root/'src/crates/sim/data/hard-vs-human-maps.json'
            maps.parent.mkdir(parents=True); maps.write_text('[]')
            p = dict(self.p, maps_sha256=gate.digest(maps.read_bytes()), arms=gate.ARMS,
                     salt_start=1001, chunk_timeout_s=1)
            gate.write(root/'protocol.json', p)
            calls = []
            def run(command, **kwargs):
                calls.append((int(command[command.index('--seed')+1]),
                              int(command[command.index('--games')+1])))
                self.assertEqual(kwargs['env']['LEGATUM_HVH_RESEED_DICE'],'1')
                self.assertNotIn('LEGATUM_UNWANTED_EXPERIMENT',kwargs['env'])
                out = Path(command[command.index('--out-dir')+1])
                gate.write(out/'ctl-games.json', {'maps':p['maps'],
                           'games':[[{'win':bool((i+start)%2)} for i in range(count)]
                                    for start,count in [calls[-1]] for _ in range(8)]})
            old = Path.cwd()
            try:
                os.chdir(root)
                with patch.object(gate.platform,'system',return_value='Linux'), \
                     patch.object(gate.subprocess,'check_output',return_value='a'*40), \
                     patch.object(gate.subprocess,'run',side_effect=run), \
                     patch.dict(os.environ, {'LEGATUM_UNWANTED_EXPERIMENT':'1'}), \
                     contextlib.redirect_stdout(io.StringIO()):
                    gate.worker(SimpleNamespace(binary=str(binary),commit='a'*40,role='control'))
                result=gate.read('result.json')
                gate.validate_records(result,p)
                self.assertEqual(calls,[(1001,128),(1129,128),(1257,256)])
                self.assertEqual(result['salts'],512)
            finally:
                os.chdir(old)


    def test_concurrent_registration_cannot_exceed_family_budget(self):
        with tempfile.TemporaryDirectory() as directory:
            self.candidate_inputs(directory)
            def register(i):
                args = SimpleNamespace(directory=directory, commit='candidate',
                                       id=f'candidate-{i}', role='candidate')
                try:
                    gate.prepare(args)
                    return True
                except ValueError as error:
                    self.assertIn('family exhausted', str(error))
                    return False
            with patch.object(gate.subprocess,'check_output',return_value='b'*40), \
                 contextlib.redirect_stdout(io.StringIO()), ThreadPoolExecutor(max_workers=6) as pool:
                registered = list(pool.map(register, range(6)))
            self.assertEqual(sum(registered),5)
            self.assertEqual(len(gate.read(Path(directory)/'candidates.json')),5)


    def spot_inputs(self, directory):
        root = Path(directory)
        p = dict(self.p, salt_start=1, maps_sha256='c'*64, arms=gate.ARMS)
        manifest = {'commit':'b'*40, 'binary_sha256':'d'*64, 'maps_sha256':'c'*64, 'profile':'sim'}
        params = root/'arms.json'; gate.write(params, gate.ARMS)
        remote = '/tmp/ai-params-' + gate.digest(params.read_bytes())[:16]
        chunks = []
        for start in (1,65):
            folder=root/str(start);folder.mkdir()
            chunk={'salt_start':start,'salts':64,'games_file':str(folder/'games.json.gz'),
                   'report_file':str(folder/'report.json.gz'),'log_file':str(folder/'log'),
                   'done_file':str(folder/'DONE')}
            gate.write(chunk['games_file'], {'maps':p['maps'], 'games':[
                [{'win':bool(i%2),'salt':start+i} for i in range(64)] for _ in range(8)]})
            gate.write(chunk['report_file'], {'ctl':{'salt_start':start,'salts_per_map':64,
                'brain':'params','seat_mode':'all-seats','rules':[],'candidate_params':'builtin'}})
            Path(chunk['log_file']).write_text('+ aws s3 cp s3://bucket/bin/'+('d'*16)+'/hard_vs_human.gz /tmp/bin\n'
                +f'+ env LEGATUM_HVH_RESEED_DICE=1 RAYON_NUM_THREADS=64 /tmp/hard_vs_human --arms --ai-params {remote} --games 64 --seed {start}\n')
            Path(chunk['done_file']).write_text('rc=0\nstart=0\nend=1\n')
            chunks.append(chunk)
        return p,manifest,chunks,params

    def test_spot_ingest_orders_and_validates_two_completed_slices(self):
        with tempfile.TemporaryDirectory() as directory:
            p,m,chunks,params=self.spot_inputs(directory)
            result,games=gate.ingest(p,m,list(reversed(chunks)),params)
            self.assertEqual(result['salts'],128)
            self.assertEqual(sum(map(sum,result['wins'])),512)
            self.assertEqual(games['games'][0][-1]['salt'],128)
            self.assertEqual(result['rayon_threads'],'64')

    def test_spot_ingest_rejects_duplicate_or_failed_slices(self):
        with tempfile.TemporaryDirectory() as directory:
            p,m,chunks,params=self.spot_inputs(directory)
            with self.assertRaisesRegex(ValueError,'gap, overlap'):
                gate.ingest(p,m,[chunks[0],chunks[0]],params)
            Path(chunks[1]['done_file']).write_text('rc=1\n')
            with self.assertRaisesRegex(ValueError,'failed cloud slice'):
                gate.ingest(p,m,chunks,params)

    def test_spot_ingest_rejects_wrong_binary_and_hidden_experiments(self):
        with tempfile.TemporaryDirectory() as directory:
            p,m,chunks,params=self.spot_inputs(directory)
            with self.assertRaisesRegex(ValueError,'another binary'):
                gate.ingest(p,dict(m,binary_sha256='e'*64),chunks,params)
            log=Path(chunks[0]['log_file']); original=log.read_text()
            log.write_text(original.replace('RAYON_NUM_THREADS=64','LEGATUM_EXPERIMENT=1 RAYON_NUM_THREADS=64'))
            with self.assertRaisesRegex(ValueError,'unexpected simulation environment'):
                gate.ingest(p,m,chunks,params)
            log.write_text(original.rstrip()+' --until-turn 100\n')
            with self.assertRaisesRegex(ValueError,'unexpected or duplicate'):
                gate.ingest(p,m,chunks,params)

    def test_spot_ingest_rejects_missing_reseed_or_wrong_params(self):
        with tempfile.TemporaryDirectory() as directory:
            p,m,chunks,params=self.spot_inputs(directory)
            log=Path(chunks[0]['log_file']); original=log.read_text()
            log.write_text(original.replace('LEGATUM_HVH_RESEED_DICE=1 ',' '))
            with self.assertRaisesRegex(ValueError,'differs from frozen protocol'):
                gate.ingest(p,m,chunks,params)
            log.write_text(original)
            gate.write(params,{'arms':[]})
            with self.assertRaisesRegex(ValueError,'parameter file differs'):
                gate.ingest(p,m,chunks,params)


if __name__ == '__main__': unittest.main()
