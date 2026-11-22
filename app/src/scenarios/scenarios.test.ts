import { afterEach, describe, expect, it } from 'vitest';
import { SOON_SLOTS, heldBackCatalog, aiResultLine, bestResultLine, compareToAi, leadTarget, newRun, nextRecords, parseEntries, runFileName, scenarioCatalog, targetText, turnsText, versusBaseline, type ScenarioFile, type ScenarioRecord, type ScenarioStatus, type ScenarioTarget } from './scenarios';

const status = (p: Partial<ScenarioStatus> = {}): ScenarioStatus => ({
  turn: 21, start_turn: 21, end_turn: 36, turns_left: 15, outcome: 'Playing', reason: '', score: 100,
  conditions: [], score_lines: [], ...p,
});

describe('the bundled scenarios', () => {
  const entries = scenarioCatalog();
  it('lists the war scenarios cut for human play, and no simulator-only file', () => {
    const ids = entries.map((e) => e.file.id);
    expect(ids).toContain('war-sparta-nicaea');
    expect(ids).not.toContain('war-golden-horde');
    expect(ids).not.toContain('rush-offense');
  });
  it('lists nothing that waits for a change, and still loads what is held back', () => {
    for (const e of entries) expect(e.file.lab?.requires).toBeUndefined();
    for (const e of heldBackCatalog()) {
      expect(entries.map((x) => x.file.id)).not.toContain(e.file.id);
      expect(e.file.lab?.requires).toMatch(/\w/);
    }
  });
  it('keeps the soon slots out of the playable list', () => {
    for (const slot of SOON_SLOTS) expect(entries.map((e) => e.file.id)).not.toContain(slot.id);
  });
  it.each(entries.map((e) => [e.file.id, e] as const))('%s has everything the card, the bar and the comparison need', (_, e) => {
    const f = e.file;
    expect(f.format).toBe('legatum-scenario');
    expect(f.version).toBe(1);
    expect(f.title).not.toMatch(/TODO/);
    expect(f.briefing.length).toBeGreaterThan(120);
    expect(f.objective.summary).not.toMatch(/TODO/);
    expect(f.objective.turns).toBeGreaterThan(0);
    expect(f.objective.score.length).toBeGreaterThan(0);
    expect(f.minutes).toBeGreaterThanOrEqual(5);
    expect(f.state.encoding).toBe('gzip-base64');
    expect(f.state.hash).toMatch(/^\d+$/);
    // The card: a question that is a question, two lines of scene, a painting.
    expect(f.lab?.question).toMatch(/\?$/);
    expect(f.lab?.blurb.length).toBeGreaterThan(40);
    expect(f.lab?.era).toBeGreaterThanOrEqual(0);
    // The result card compares with the Hard AI from the same start.
    expect(f.baseline, `${f.id} needs a Hard AI baseline (tools/scenarios/cut.sh records one)`).toBeTruthy();
  });
});

describe('test overrides', () => {
  afterEach(() => { delete (globalThis as { legatumScenarioOverrides?: unknown }).legatumScenarioOverrides; });
  it('shorten a scenario before it loads, and leave the others alone', () => {
    const text = JSON.stringify({ format: 'legatum-scenario', id: 'a', title: 'A', lab: { question: 'q?', blurb: 'b' }, objective: { summary: 's', turns: 12, score: [] } });
    const other = JSON.stringify({ format: 'legatum-scenario', id: 'b', title: 'B', lab: { question: 'q?', blurb: 'b' }, objective: { summary: 's', turns: 9, score: [] } });
    (globalThis as { legatumScenarioOverrides?: unknown }).legatumScenarioOverrides = { a: { objective: { turns: 1 } } };
    const [a, b] = parseEntries([text, other]);
    expect(a.file.objective).toEqual({ summary: 's', turns: 1, score: [] });
    expect(JSON.parse(a.text).objective.turns).toBe(1);
    expect(b.text).toBe(other);
  });
});

describe('records', () => {
  it('keep the best score per revision and count plays', () => {
    let r = nextRecords({}, 'x', 1, status({ score: 80, outcome: 'Lost' }), 't1');
    r = nextRecords(r, 'x', 1, status({ score: 120, outcome: 'Won', turn: 27 }), 't2');
    r = nextRecords(r, 'x', 1, status({ score: 90, outcome: 'Won', turn: 26 }), 't3');
    // The best score stays 120, and the fewest turns of any win is 5.
    expect(r.x).toEqual({ revision: 1, best: 120, bestOutcome: 'Won', bestTurns: 5, plays: 3, lastAt: 't3' });
    // A new revision is a new scenario.
    expect(nextRecords(r, 'x', 2, status({ score: 10, outcome: 'Lost' }), 't4').x).toEqual({ revision: 2, best: 10, bestOutcome: 'Lost', plays: 1, lastAt: 't4' });
  });
});

describe('runs', () => {
  const entry = scenarioCatalog()[1];
  it('start empty and name their file by scenario and time', () => {
    const run = newRun(entry, status(), 'wasm', '2026-10-03T20:25:09.123Z');
    expect(run).toMatchObject({ format: 'legatum-scenario-run', version: 1, player: 'human', turns: [], outcome: null, finished_at: null });
    expect(run.scenario).toMatchObject({ id: entry.file.id, seat: entry.file.seat, state_hash: entry.file.state.hash, turns: entry.file.objective.turns });
    expect(run.initial_state).toBe(entry.file.state);
    expect(runFileName(run)).toBe(`${entry.file.id}-2026-10-03T20-25-09Z.json`);
    // The same name when it ends, so the finished run replaces its last checkpoint.
    expect(runFileName({ ...run, finished_at: '2026-10-03T21:00:00Z' })).toBe(`${entry.file.id}-2026-10-03T20-25-09Z.json`);
  });
});

describe('words', () => {
  it('turns left, and the score against the AI baseline', () => {
    expect(turnsText(status({ turns_left: 1 }))).toBe('1 turn left');
    expect(turnsText(status({ turns_left: 4 }))).toBe('4 turns left');
    expect(turnsText(status({ outcome: 'Won', turn: 30 }))).toBe('Ended on turn 30');
    const b = { runs: 50, difficulty: 'Hard', median: 90, p10: 80, p90: 100, win_percent: 0, measured_at: '' };
    expect(versusBaseline(150, b)).toBe("Better than 90% of the Hard AI's 50 tries");
    expect(versusBaseline(95, b)).toBe("Better than the Hard AI's median of 90");
    expect(versusBaseline(90, b)).toBe("Level with the Hard AI's median of 90");
    expect(versusBaseline(85, b)).toBe("Below the Hard AI's median of 90");
    expect(versusBaseline(10, b)).toBe("Below 90% of the Hard AI's 50 tries");
  });
});

describe('objective targets', () => {
  const t = (kind: ScenarioTarget['kind'], name: string, distance: number, bearing = ''): ScenarioTarget => ({
    kind, name, distance, bearing, city: name.length, owner: 1, pos: { q: distance, r: 0 },
  });
  it('points the bar at a city to take, else the nearest city to contain, else home', () => {
    const take = t('Take', 'Last Bridge', 4, '4 tiles north of Promise');
    const near = t('Contain', 'Resonator', 6, '6 tiles north-east of Tessel Market');
    const far = t('Contain', 'Waystation', 9, '9 tiles north of Tessel Market');
    const home = t('Hold', 'Promise', 0);
    expect(leadTarget([home, near, take])).toBe(take);
    expect(leadTarget([far, home, near])).toBe(near);
    expect(leadTarget([home])).toBe(home);
    expect(leadTarget([])).toBeUndefined();
    expect(leadTarget(undefined)).toBeUndefined();
  });
  it('points a hold-every-city objective at the city the briefing names, not the capital', () => {
    const capital = { ...t('Hold', 'Lugano', 0), city: 1 };
    const biel = { ...t('Hold', 'Biel', 0), city: 9 };
    const bern = { ...t('Hold', 'Bern', 0), city: 4 };
    const text = 'Hold all 11 of your cities through turn 140. Ten Indian units stand within 5 tiles of Biel. Bern is quiet.';
    expect(leadTarget([capital, bern, biel], text)).toBe(biel);
    expect(leadTarget([capital, bern, biel], 'Hold all your cities.')).toBe(capital);
    expect(leadTarget([capital, { ...t('Hold', 'Bielefeld', 0), city: 2 }], 'Hold Biel.')).toBe(capital);
  });
  it('names the target and where it is', () => {
    expect(targetText(t('Contain', 'Resonator', 6, '6 tiles north-east of Tessel Market'))).toBe('Contain Resonator: 6 tiles north-east of Tessel Market');
    expect(targetText(t('Hold', 'Hymnal', 0))).toBe('Hold Hymnal');
  });
});

describe('the comparison with the Hard AI', () => {
  const baseline = { runs: 64, difficulty: 'Hard', median: 500, p10: 400, p90: 520, win_percent: 100, median_turns: 4, measured_at: '' };
  const file = (over: Partial<ScenarioFile['objective']>, b: Partial<typeof baseline> | null = {}): ScenarioFile => ({
    format: 'legatum-scenario', version: 1, id: 'x', revision: 1, title: 'X', faction: 'Sparta', seat: 0, difficulty: 'Hard', minutes: 10,
    briefing: '', objective: { summary: '', turns: 8, score: [], ...over },
    predictions: { good_play: '', ai_play: '', ai_score: { median: 0, low: 0, high: 0, win_percent: 0 } },
    baseline: b === null ? null : { ...baseline, ...b }, state: { encoding: 'gzip-base64', hash: '1', data: '' },
  });
  const race = file({ end_on_win: true, win: [{ OwnCity: 1 }] });
  const hold = file({ win: ['HoldStartingCities'], keep: ['HoldStartingCities'] }, { win_percent: 100, median_turns: null as unknown as number });

  it('says how many turns you took against the AI from the same start', () => {
    const faster = compareToAi(race, status({ outcome: 'Won', turn: 21 + 3 }));
    expect(faster.you).toBe('You took 3 turns.');
    expect(faster.ai).toBe('The Hard AI took 4 turns from the same start, and did it in 64 of 64 tries.');
    expect(faster.verdict).toBe('That is 1 turn faster than the Hard AI.');
    expect(compareToAi(race, status({ outcome: 'Won', turn: 21 + 6 })).verdict).toBe('That is 2 turns slower than the Hard AI.');
    expect(compareToAi(race, status({ outcome: 'Won', turn: 21 + 4 })).verdict).toBe('That matches the Hard AI.');
  });
  it('words a miss and an AI that never managed it', () => {
    const miss = compareToAi(race, status({ outcome: 'Lost', turn: 29 }));
    expect(miss.you).toBe('You did not meet the objective in 8 turns.');
    expect(miss.verdict).toBe('The Hard AI managed it where you did not.');
    const hard = file({ end_on_win: true, win: [{ OwnCity: 1 }] }, { win_percent: 0 });
    expect(compareToAi(hard, status({ outcome: 'Won', turn: 24 })).verdict).toBe('You did what the Hard AI could not.');
    expect(compareToAi(hard, status({ outcome: 'Won', turn: 24 })).ai).toBe('The Hard AI failed all 64 tries from the same start.');
  });
  it('words a defence by the cities held, not by turns', () => {
    const held = compareToAi(hold, status({ outcome: 'Won', turn: 29 }));
    expect(held.you).toBe('You held every city for all 8 turns.');
    expect(held.ai).toBe('The Hard AI held every city in 64 of 64 tries from the same start.');
    const fell = file({ win: ['HoldStartingCities'], keep: ['HoldStartingCities'] }, { win_percent: 10 });
    expect(compareToAi(fell, status({ outcome: 'Won', turn: 29 })).verdict).toBe('You held where the Hard AI usually fell.');
  });
  it('falls back to the score for score-only scenarios, and to a plain line with no baseline', () => {
    const score = file({}, { win_percent: null as unknown as number });
    expect(compareToAi(score, status({ outcome: 'Complete', score: 700 })).ai).toBe("Better than 90% of the Hard AI's 64 tries (median 500).");
    expect(compareToAi(race, status({ outcome: 'Won' })).you).toMatch(/^You took/);
    expect(compareToAi(file({ end_on_win: true }, null), status({ outcome: 'Won', turn: 24 })).ai).toBe('The Hard AI has not played this start yet.');
  });
  it('gives the card one line each for you and the AI', () => {
    expect(aiResultLine(race)).toBe('Hard AI: did it in 4 turns, 64 of 64 tries');
    expect(aiResultLine(hold)).toBe('Hard AI: held every city in 64 of 64 tries');
    expect(aiResultLine(file({ end_on_win: true }, null))).toBe('Hard AI: not measured yet');
    const rec = (p: Partial<ScenarioRecord>): ScenarioRecord => ({ revision: 1, best: 300, bestOutcome: 'Won', plays: 1, lastAt: '', ...p });
    expect(bestResultLine(race, undefined)).toBe('Not played yet');
    expect(bestResultLine(race, rec({ bestTurns: 3 }))).toBe('Your best: 3 turns');
    expect(bestResultLine(hold, rec({}))).toBe('Your best: objective met, score 300');
    expect(bestResultLine(hold, rec({ bestOutcome: 'Lost', best: 120 }))).toBe('Your best: failed, score 120');
    expect(bestResultLine(race, rec({ revision: 0 }))).toBe('Not played yet');
  });
});
