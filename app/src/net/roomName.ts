// The name a game shows in the game list. The game server shortens names with the same rule (kept in step with
// infra/relay/src/shared.ts `sanitizeName`, minus its word blocklist, which only the server applies).

/** Most characters of a name the game list shows. */
export const MAX_ROOM_NAME = 40;

/** Letters of any script, digits, spaces and a little punctuation survive; everything else is dropped. */
const DROP = /[^\p{L}\p{M}\p{N}\s'’.,!?&()#:_-]/gu;

type Segmenter = new (locale?: string, options?: { granularity: 'grapheme' }) => { segment(s: string): Iterable<{ segment: string }> };

function visibleCharacters(s: string): string[] {
  const Seg = (Intl as unknown as { Segmenter?: Segmenter }).Segmenter;
  return Seg ? Array.from(new Seg(undefined, { granularity: 'grapheme' }).segment(s), (x) => x.segment) : Array.from(s);
}

/** The name as the game list will show it; '' when nothing usable is left (the list then numbers the game). */
export function cleanRoomName(raw: string): string {
  let s = raw.normalize('NFC').replace(DROP, '');
  s = s.replace(/(^|[^\p{L}\p{M}])\p{M}+/gu, '$1');
  s = s.replace(/(\p{M}{3})\p{M}+/gu, '$1');
  s = s.replace(/\s+/gu, ' ').trim();
  const g = visibleCharacters(s);
  return g.length > MAX_ROOM_NAME ? g.slice(0, MAX_ROOM_NAME).join('').trimEnd() : s;
}
