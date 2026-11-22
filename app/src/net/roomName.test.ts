import { expect, it } from 'vitest';
import { MAX_ROOM_NAME, cleanRoomName } from './roomName';

it('keeps letters of any script, digits and light punctuation', () => {
  expect(cleanRoomName("Ada's game #2, Zürich! 東京")).toBe("Ada's game #2, Zürich! 東京");
});

it('drops emoji and symbols and squeezes spaces', () => {
  expect(cleanRoomName('  Ada \u{1F600}  *of*   Rome ')).toBe('Ada of Rome');
});

it('is empty when nothing usable is left', () => {
  expect(cleanRoomName('\u{1F600}\u{1F600}')).toBe('');
});

it('keeps at most 40 visible characters, counting an emoji-free cluster once', () => {
  const long = 'x'.repeat(60);
  expect(cleanRoomName(long)).toBe('x'.repeat(MAX_ROOM_NAME));
  expect(cleanRoomName('e\u0301'.repeat(50))).toBe('\u00e9'.repeat(MAX_ROOM_NAME));
});
