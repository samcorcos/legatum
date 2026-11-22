// The copy lint for scenario files that are not bundled yet: the pre-merge gate (tools/gates/scenario.py)
// runs it on a file with LEGATUM_LINT_SCENARIOS=<path>[:<path>...] so a Lab scenario
// can ship without the full test suite. The same blocks the bundled scenarios are held to in
// ui/textArtifacts.test.tsx: title, briefing, objective, card text, opponent and what the scenario tests.
// Without the variable it checks nothing and passes.
import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import { auditBlocks, auditCopy } from '../test/textArtifacts';
import type { ScenarioFile } from './scenarios';

const paths = (process.env.LEGATUM_LINT_SCENARIOS ?? '').split(':').filter(Boolean);

it('reads every named scenario cleanly (none named: nothing to do)', () => {
  const problems = paths.flatMap((path) => {
    const f = JSON.parse(readFileSync(path, 'utf8')) as ScenarioFile;
    const text = [f.title, f.briefing, f.objective?.summary, f.lab?.question, f.lab?.blurb, f.lab?.ask, f.lab?.opponent, f.tests].filter((b): b is string => !!b);
    return [...auditBlocks(path, text), ...auditCopy(path, text)];
  });
  expect(problems).toEqual([]);
});
