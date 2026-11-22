// @vitest-environment happy-dom
// Coach questions: the card shows one question, sends his words, hides on Esc (Skip skips), and the store keeps
// at most one waiting (a new one counts the old as skipped) and remembers what was asked.
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { useGame, saveFile } from '../store';
import { MAX_SAVED_ANSWERS, NO_WHY, answerLine, remember, unwrittenAnswers, withAnswer, type WhyQuestion } from '../why';
import { WhyCard } from './WhyCard';
import { CoachLine } from './TurnReport';

const q = (over: Partial<WhyQuestion> = {}): WhyQuestion => ({
  turn: 17, category: 'war_peace', key: 'warpeace:declare:1', materiality: 1_000_000,
  text: 'Turn 17: you declared war on Athens. Hard would not have declared war. What made the difference?',
  evidence: { his: ['DeclareWar'], hard: [] }, ...over,
});

beforeEach(() => { useGame.setState({ reportOpen: false, report: null, whyQuestion: null, whyHidden: false, whyRecent: null, why: { ...NO_WHY, enabled: true }, client: null }); });
afterEach(cleanup);

it('shows nothing without a question and the question text with one', () => {
  render(<WhyCard />);
  expect(screen.queryByTestId('why-card')).toBeNull();
  useGame.setState({ whyQuestion: q() });
  const { container } = render(<WhyCard />);
  expect(container.querySelector('[data-testid=why-text]')!.textContent).toContain('you declared war on Athens');
});

it('Send is off until he has written something, then saves his words and closes', () => {
  useGame.setState({ whyQuestion: q() });
  render(<WhyCard />);
  const send = screen.getByTestId('why-send') as HTMLButtonElement;
  expect(send.disabled).toBe(true);
  fireEvent.change(screen.getByTestId('why-input'), { target: { value: 'Their Settler was alone.' } });
  expect(send.disabled).toBe(false);
  const before = unwrittenAnswers().length;
  fireEvent.click(send);
  expect(useGame.getState().whyQuestion).toBeNull();
  expect(JSON.parse(unwrittenAnswers()[before]).answer).toBe('Their Settler was alone.');
});

it('Cmd+Return sends and the Skip button skips, keeping the card out of the game keys', () => {
  useGame.setState({ whyQuestion: q() });
  render(<WhyCard />);
  fireEvent.change(screen.getByTestId('why-input'), { target: { value: 'reason' } });
  const before = unwrittenAnswers().length;
  fireEvent.keyDown(screen.getByTestId('why-input'), { key: 'Enter', metaKey: true });
  expect(useGame.getState().whyQuestion).toBeNull();
  expect(JSON.parse(unwrittenAnswers()[before]).answer).toBe('reason');
  cleanup();
  useGame.setState({ whyQuestion: q({ turn: 20, key: 'k2' }) });
  render(<WhyCard />);
  fireEvent.click(screen.getByTestId('why-skip'));
  expect(useGame.getState().whyQuestion).toBeNull();
  const last = JSON.parse(unwrittenAnswers()[unwrittenAnswers().length - 1]);
  expect(last).toMatchObject({ type: 'answer', turn: 20, answer: 'skipped', category: 'war_peace' });
  expect(useGame.getState().why.skipped).toEqual(['war_peace']);
});

it('Esc only hides the card: the question stays, the draft is kept, and the button brings it back', () => {
  useGame.setState({ whyQuestion: q({ turn: 21, key: 'k3' }) });
  render(<WhyCard />);
  fireEvent.change(screen.getByTestId('why-input'), { target: { value: 'half an answer' } });
  const before = unwrittenAnswers().length;
  // false: the card took the Escape (default prevented), so the map does not also act on it.
  expect(fireEvent.keyDown(screen.getByTestId('why-input'), { key: 'Escape' })).toBe(false);
  expect(screen.queryByTestId('why-card')).toBeNull();
  expect(useGame.getState().whyQuestion).not.toBeNull();
  expect(unwrittenAnswers().length).toBe(before);
  expect(useGame.getState().why.skipped).toEqual([]);
  const reopen = screen.getByTestId('why-reopen');
  expect(reopen.textContent).toContain('Coach question');
  expect(reopen.textContent).toContain('draft kept');
  fireEvent.click(reopen);
  expect((screen.getByTestId('why-input') as HTMLTextAreaElement).value).toBe('half an answer');
  fireEvent.click(screen.getByTestId('why-hide'));
  expect(screen.getByTestId('why-reopen')).toBeTruthy();
  expect(useGame.getState().whyQuestion).not.toBeNull();
});

it('the turn report lists the question: Answer reopens a hidden card, and a skipped one can still be answered', () => {
  useGame.setState({ whyQuestion: q({ turn: 22, key: 'k4' }), whyHidden: true, reportOpen: true, report: { turn: 22 } as never });
  render(<><WhyCard /><CoachLine /></>);
  // With the report open its own line is the way back: no floating button over its header.
  expect(screen.queryByTestId('why-reopen')).toBeNull();
  expect(screen.getByTestId('report-coach').textContent).toContain('Waiting for your answer');
  fireEvent.click(screen.getByTestId('report-coach-open'));
  expect(screen.getByTestId('why-card')).toBeTruthy();
  expect(useGame.getState().reportOpen).toBe(false);
  fireEvent.click(screen.getByTestId('why-skip'));
  expect(useGame.getState().why.skipped).toEqual(['war_peace']);
  expect(screen.getByTestId('report-coach').textContent).toContain('You skipped it');
  fireEvent.click(screen.getByTestId('report-coach-open'));
  expect(useGame.getState().whyQuestion?.key).toBe('k4');
  expect(useGame.getState().why.skipped).toEqual([]);
  fireEvent.change(screen.getByTestId('why-input'), { target: { value: 'late reason' } });
  const before = unwrittenAnswers().length;
  fireEvent.click(screen.getByTestId('why-send'));
  expect(JSON.parse(unwrittenAnswers()[before])).toMatchObject({ turn: 22, answer: 'late reason' });
  expect(screen.getByTestId('report-coach').textContent).toContain('You answered it');
  expect(screen.queryByTestId('report-coach-open')).toBeNull();
});

it('remembers each asked key once and the last category, and the answer line carries the evidence', () => {
  const a = remember(NO_WHY, q());
  const b = remember(a, q());
  expect(b.asked).toEqual(['warpeace:declare:1']);
  expect(b.last).toEqual([17, 'war_peace']);
  const line = answerLine('game-1', q(), '  ');
  expect(line.answer).toBe('skipped');
  expect(line.evidence).toEqual({ his: ['DeclareWar'], hard: [] });
  vi.fn();
});

it('answers are kept in the saved coach memory, a late answer adds a second line, and the list is capped', () => {
  useGame.setState({ whyQuestion: q({ turn: 30, key: 'k9' }) });
  useGame.getState().answerWhy(null);
  useGame.getState().showWhy();
  useGame.getState().answerWhy('late reason');
  const answers = useGame.getState().why.answers!;
  expect(answers.map((a) => a.answer)).toEqual(['skipped', 'late reason']);
  expect(answers[1]).toMatchObject({ turn: 30, question: q().text });
  const file = saveFile({ kind: 'wasm', save: () => 'x' } as never, [{ turn: 30, me: 0, players: [] }] as never);
  expect(file.why?.answers).toHaveLength(2);
  expect(JSON.parse(JSON.stringify(file)).why.answers[1].answer).toBe('late reason');
  let memory = NO_WHY;
  for (let i = 0; i < MAX_SAVED_ANSWERS + 5; i++) memory = withAnswer(memory, answerLine('g', q({ turn: i }), 'a'));
  expect(memory.answers).toHaveLength(MAX_SAVED_ANSWERS);
  expect(memory.answers![MAX_SAVED_ANSWERS - 1].turn).toBe(MAX_SAVED_ANSWERS + 4);
});
