import { describe, expect, it } from 'vitest';
import { ActivePlayClock } from './activePlayTime';

describe('active play clock', () => {
  it('counts focused time and caps an idle gap at three minutes', () => {
    const clock = new ActivePlayClock(true);
    clock.input(0);
    expect(clock.seconds(600_000)).toBe(180);
    clock.input(600_000);
    expect(clock.seconds(615_000)).toBe(195);
  });

  it('stops on blur and resumes on focus only while activity is recent', () => {
    const clock = new ActivePlayClock(true);
    clock.input(0);
    clock.setFocused(false, 10_000);
    clock.input(80_000);
    expect(clock.seconds(100_000)).toBe(10);
    clock.setFocused(true, 100_000);
    expect(clock.seconds(110_000)).toBe(20);
    clock.setFocused(false, 120_000);
    clock.setFocused(true, 400_000);
    expect(clock.seconds(410_000)).toBe(30);
  });

  it('starts a fresh turn total while keeping recent focused activity alive', () => {
    const clock = new ActivePlayClock(true);
    clock.input(0);
    expect(clock.seconds(35_000)).toBe(35);
    clock.reset(35_000);
    expect(clock.seconds(40_000)).toBe(5);
  });
});
