import type { StagingServerMsg } from '../staging';
import type { ServerMsg, SocketLike } from '../remote';

export class FakeSocket implements SocketLike {
  readyState = 0;
  sent: Record<string, unknown>[] = [];
  onopen: ((ev: unknown) => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onclose: ((ev: unknown) => void) | null = null;
  onerror: ((ev: unknown) => void) | null = null;
  constructor(public url: string) {}
  send(data: string) {
    if (this.readyState !== 1) throw new Error('socket is not open');
    this.sent.push(JSON.parse(data));
  }
  close() {
    this.readyState = 3;
    this.onclose?.({});
  }
  open() {
    this.readyState = 1;
    this.onopen?.({});
  }
  push(msg: ServerMsg | StagingServerMsg) {
    this.onmessage?.({ data: JSON.stringify(msg) });
  }
  drop(code = 1006, reason = '') {
    this.readyState = 3;
    this.onclose?.({ code, reason });
  }
  types() {
    return this.sent.map((m) => m.type);
  }
}
