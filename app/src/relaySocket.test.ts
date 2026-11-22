import { describe, expect, it } from 'vitest';
import { FakeSocket } from './test/fakeSocket';
import { relaySocketFactory, wrapRelaySocket } from './relaySocket';

const chunk = (id: number, i: number, n: number, data: string) => JSON.stringify({ type: 'chunk', id, i, n, data });

function setup() {
  const raw = new FakeSocket('wss://r/rooms/x/ws');
  const sock = wrapRelaySocket(raw);
  const got: string[] = [];
  sock.onmessage = (ev) => got.push(String(ev.data));
  const feed = (text: string) => raw.onmessage?.({ data: text });
  return { raw, sock, got, feed };
}

describe('relaySocket', () => {
  it('passes normal messages through untouched', () => {
    const { got, feed } = setup();
    feed('{"type":"pong"}');
    feed('not json at all');
    feed('{"type":"chunky"}');
    expect(got).toEqual(['{"type":"pong"}', 'not json at all', '{"type":"chunky"}']);
  });

  it('reassembles pieces, with normal messages before and after', () => {
    const { got, feed } = setup();
    const whole = JSON.stringify({ type: 'view', view: { name: 'Ada' } });
    feed('{"type":"seats","seats":[]}');
    feed(chunk(1, 0, 3, whole.slice(0, 10)));
    feed(chunk(1, 1, 3, whole.slice(10, 20)));
    expect(got).toHaveLength(1);
    feed(chunk(1, 2, 3, whole.slice(20)));
    feed('{"type":"pong"}');
    expect(got).toEqual(['{"type":"seats","seats":[]}', whole, '{"type":"pong"}']);
  });

  it('keeps multi-byte characters intact across piece boundaries', () => {
    const { got, feed } = setup();
    const whole = JSON.stringify({ type: 'view', name: 'Zürich 🏔️ 東京 ⚔️' });
    const chars = [...whole];
    const pieces: string[] = [];
    for (let i = 0; i < chars.length; i += 3) pieces.push(chars.slice(i, i + 3).join(''));
    pieces.forEach((p, i) => feed(chunk(7, i, pieces.length, p)));
    expect(got).toEqual([whole]);
    expect(JSON.parse(got[0]).name).toBe('Zürich 🏔️ 東京 ⚔️');
  });

  it('handles two chunked messages in a row', () => {
    const { got, feed } = setup();
    feed(chunk(1, 0, 2, 'ab'));
    feed(chunk(1, 1, 2, 'cd'));
    feed(chunk(2, 0, 2, 'ef'));
    feed(chunk(2, 1, 2, 'gh'));
    expect(got).toEqual(['abcd', 'efgh']);
  });

  it('drops a partial message on a gap, a wrong id or a bad piece, without throwing', () => {
    const { got, feed } = setup();
    feed(chunk(1, 0, 3, 'a'));
    feed(chunk(1, 2, 3, 'c')); // gap
    feed(chunk(1, 1, 3, 'b')); // late, nothing to join
    expect(got).toEqual([]);
    feed(chunk(2, 0, 2, 'a'));
    feed(chunk(3, 1, 2, 'b')); // other message
    expect(got).toEqual([]);
    feed(chunk(4, 0, 2, 'a'));
    feed('{"type":"chunk","id":4,"i":"1","n":2,"data":"b"}');
    feed('{"type":"chunk","id":4}');
    feed('{"type":"chunk","id":4,"i":1,"n":0,"data":"b"}');
    expect(got).toEqual([]);
    feed(chunk(5, 0, 2, 'ok'));
    feed(chunk(5, 1, 2, '!'));
    expect(got).toEqual(['ok!']);
  });

  it('drops a message that grows past 16 MiB of characters, and keeps working after', () => {
    const { got, feed } = setup();
    const piece = 'x'.repeat(256 * 1024);
    for (let i = 0; i < 64; i++) feed(chunk(1, i, 80, piece)); // exactly 16 MiB so far
    feed(chunk(1, 64, 80, 'y')); // one character over
    for (let i = 65; i < 80; i++) feed(chunk(1, i, 80, piece));
    expect(got).toEqual([]);
    feed(chunk(2, 0, 2, 'ok'));
    feed(chunk(2, 1, 2, '!'));
    expect(got).toEqual(['ok!']);
  });

  it('delivers a message just under the 16 MiB cap', () => {
    const { got, feed } = setup();
    const piece = 'x'.repeat(256 * 1024);
    for (let i = 0; i < 64; i++) feed(chunk(1, i, 64, piece));
    expect(got).toHaveLength(1);
    expect(got[0]).toHaveLength(16 * 1024 * 1024);
  });

  it('drops a partial message when a normal message interrupts it', () => {
    const { got, feed } = setup();
    feed(chunk(1, 0, 2, 'a'));
    feed('{"type":"pong"}');
    feed(chunk(1, 1, 2, 'b'));
    expect(got).toEqual(['{"type":"pong"}']);
  });

  it('forwards state, handlers, send and close, and forgets a partial on close', () => {
    const { raw, sock, got, feed } = setup();
    const events: string[] = [];
    sock.onopen = () => events.push('open');
    sock.onclose = () => events.push('close');
    raw.open();
    expect(sock.readyState).toBe(1);
    sock.send('{"type":"ping"}');
    expect(raw.types()).toEqual(['ping']);
    feed(chunk(1, 0, 2, 'a'));
    raw.drop();
    feed(chunk(1, 1, 2, 'b'));
    expect(got).toEqual([]);
    expect(events).toEqual(['open', 'close']);
    sock.close();
    expect(sock.readyState).toBe(3);
  });

  it('wraps every socket a factory makes', () => {
    const made: FakeSocket[] = [];
    const make = relaySocketFactory((u) => { const s = new FakeSocket(u); made.push(s); return s; });
    const sock = make('wss://r/ws');
    const got: string[] = [];
    sock.onmessage = (ev) => got.push(String(ev.data));
    made[0].onmessage?.({ data: chunk(1, 0, 1, 'whole') });
    expect(got).toEqual(['whole']);
  });
});
