// @vitest-environment happy-dom
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, expect, it } from 'vitest';
import { useGame } from '../store';
import { GameErrorToast } from './GameScreen';

beforeEach(() => {
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ error: 'Nothing to fire on there' });
});

afterEach(() => {
  cleanup();
  useGame.setState(useGame.getInitialState(), true);
});

it('announces the error and clears it with the keyboard-reachable Dismiss button', async () => {
  const user = userEvent.setup();
  render(<GameErrorToast />);

  const alert = screen.getByRole('alert');
  expect(alert.textContent).toContain('Nothing to fire on there');
  const dismiss = screen.getByRole('button', { name: 'Dismiss' });
  dismiss.focus();
  await user.keyboard('{Enter}');

  expect(screen.queryByRole('alert')).toBeNull();
  expect(useGame.getState().error).toBeNull();
});
