import { beforeEach, describe, expect, it, vi } from 'vitest';
import { newGame, onSocket } from './fake-lan-server.mjs';

class FakeSocket {
  sent = [];
  onmessage = null;
  onclose = null;
  send(message) {
    this.sent.push(message);
  }
  close() {
    this.onclose?.();
  }
  push(message) {
    this.onmessage(JSON.stringify(message));
  }
}

describe('fake LAN server frame handling', () => {
  let socket;
  let validate;
  let forecast;

  beforeEach(() => {
    validate = vi.fn(() => '[]');
    forecast = vi.fn(() => 'null');
    newGame(JSON.stringify({ players: [
      { name: 'Ana', faction: 'Japan', is_human: true },
      { name: 'Ben', faction: 'Byzantium', is_human: true },
    ] }), class {
      planningTurn = 1;
      turn() { return this.planningTurn; }
      view() { return JSON.stringify({ turn: this.planningTurn }); }
      end_turn() {
        return JSON.stringify({ turn: this.planningTurn++, events: [], hash: '123', finished: false, winner: null });
      }
      validate(...args) { return validate(...args); }
      forecast(...args) { return forecast(...args); }
    });
    socket = new FakeSocket();
    onSocket(socket);
  });

  const expectErrorAndRecovery = () => {
    expect(socket.sent.at(-1)).toEqual({ type: 'error', message: expect.any(String) });
    socket.push({ type: 'ping' });
    expect(socket.sent.at(-1)).toEqual({ type: 'pong' });
  };

  it('sends the new planning turn on the wire, matching the Rust server', () => {
    const teammate = new FakeSocket();
    onSocket(teammate);
    socket.push({ type: 'join', seat: 0, name: 'Ana' });
    teammate.push({ type: 'join', seat: 1, name: 'Ben' });
    socket.push({ type: 'ready', ready: true });
    teammate.push({ type: 'ready', ready: true });
    for (const peer of [socket, teammate]) {
      const report = peer.sent.findIndex((m) => m.type === 'turn');
      expect(peer.sent[report]).toMatchObject({ type: 'turn', turn: 2 });
      expect(peer.sent[report + 1]).toEqual({ type: 'view', view: { turn: 2 } });
    }
  });

  it.each([null, false, 7, 'ping', [], {}, { type: null }, { type: 1 }, { type: {} }, { type: '' }, { type: ' ' }].map((message) => [message]))(
    'rejects invalid message shape %j before joining and keeps processing frames',
    (message) => {
      expect(() => socket.push(message)).not.toThrow();
      expect(socket.sent.at(-1)).toEqual({ type: 'error', message: 'expected an object with a non-empty string type' });
      expectErrorAndRecovery();
      socket.push({ type: 'join', seat: 0, name: 'Ana' });
      expect(socket.sent.some((m) => m.type === 'welcome')).toBe(true);
    },
  );

  it('rejects invalid JSON and continues serving the same socket', () => {
    expect(() => socket.onmessage('{')).not.toThrow();
    expect(socket.sent.at(-1)).toEqual({ type: 'error', message: 'bad json' });
    expectErrorAndRecovery();
  });

  it('catches a malformed forecast dispatch and accepts the next valid forecast', () => {
    socket.push({ type: 'join', seat: 0, name: 'Ana' });
    expect(() => socket.push({ type: 'forecast', req_id: 1, unit: 4 })).not.toThrow();
    expectErrorAndRecovery();
    socket.push({ type: 'forecast', req_id: 2, unit: 4, target: { q: 1, r: 2 }, mode: 'Melee' });
    expect(forecast).toHaveBeenLastCalledWith(0, 4, 1, 2, 'Melee');
    expect(socket.sent.at(-1)).toEqual({ type: 'forecast', req_id: 2, forecast: null });
    socket.push({ type: 'forecast', req_id: 3, unit: 4, target: { q: 1, r: 2 }, mode: 'Ranged' });
    expect(forecast).toHaveBeenLastCalledWith(0, 4, 1, 2, 'Ranged');
  });

  it('catches engine dispatch exceptions without replacing the last valid orders', () => {
    socket.push({ type: 'join', seat: 0, name: 'Ana' });
    socket.push({ type: 'orders', orders: [{ type: 'HoldUnit', unit: 4 }] });
    validate.mockImplementationOnce(() => { throw new Error('invalid order'); });
    expect(() => socket.push({ type: 'orders', orders: [{ type: 'bogus' }, {}] })).not.toThrow();
    expect(socket.sent.at(-1)).toEqual({ type: 'error', message: 'bad message: invalid order' });
    expectErrorAndRecovery();
    socket.push({ type: 'ready', ready: false });
    expect(socket.sent.at(-1).seats[0].orders).toBe(1);
    socket.push({ type: 'orders', orders: [] });
    expect(socket.sent.at(-1).seats[0].orders).toBe(0);
  });

  it.each([null, {}, 'orders', 1])('rejects a non-array orders list %j before changing seat state', (orders) => {
    socket.push({ type: 'join', seat: 0, name: 'Ana' });
    expect(() => socket.push({ type: 'orders', orders })).not.toThrow();
    expect(socket.sent.at(-1)).toEqual({ type: 'error', message: 'orders must be an array' });
    expect(validate).not.toHaveBeenCalled();
    expectErrorAndRecovery();
  });
});
