/**
 * Game uploads (infra/games): the hosted web build sends each game's record to a private Cloudflare R2 bucket,
 * after each of the first five turns, then every five turns, and when the game ends, so the AI can be measured and bugs found. Every seat goes under its
 * default name ("Player 1", "AI 2", from the engine's `public_archive`), and the server refuses any other name.
 * scripts/build-web.sh bakes the address in as VITE_LEGATUM_GAMES_URL; other builds leave it empty and upload nothing.
 */
export const GAMES_URL: string = String(import.meta.env.VITE_LEGATUM_GAMES_URL ?? '').replace(/\/+$/, '');

/** README section that says what is saved and why; the title screen links to it. */
export const PRIVACY_URL = 'https://github.com/samcorcos/legatum#privacy';

export const uploadsGames = (): boolean => GAMES_URL !== '';

/**
 * Whether the record is captured after this many ended turns: at setup (0), after each of the first five turns,
 * then every fifth. Most games that stop do so in the first five turns, and one checkpoint at setup and one at
 * turn 5 cannot say which of those turns a player left on.
 */
export const isHistoryCheckpoint = (turnsEnded: number): boolean => turnsEnded <= 5 || turnsEnded % 5 === 0;

/**
 * The record to keep and upload: names removed when uploads are on, the player's coach-question answers added as
 * `coach_answers`, and the times the player went back to an earlier turn as `rewinds` (store.ts Rewind: the tape
 * holds only the line kept, so analysis learns nothing from the turns discarded). Only this upload copy carries
 * them: the engine's `Recording` refuses unknown keys, so exports and anything fed back to the engine strip them
 * (archive.ts).
 */
export function historyToRecord(client: { archive(): string; publicArchive?(): string }, answers: readonly unknown[] = [], rewinds: readonly unknown[] = []): string {
  if (!uploadsGames()) return client.archive();
  const json = client.publicArchive ? client.publicArchive() : client.archive();
  if (!answers.length && !rewinds.length) return json;
  return JSON.stringify({
    ...(JSON.parse(json) as object),
    ...(answers.length ? { coach_answers: answers } : {}),
    ...(rewinds.length ? { rewinds } : {}),
  });
}

const KEYS = 'legatum.uploadKeys.v1';
const MAX_KEYS = 200;

/**
 * The upload key of one game: a random secret made in this browser the first time the game is uploaded and
 * sent with each of its checkpoints. The server remembers its hash and refuses a checkpoint for that game id
 * under any other key, so someone who learns a game's id (it is inside every exported save) cannot replace the
 * record. It names a game, never a player: each game gets its own. Null where the browser keeps no storage;
 * the upload then goes without a key, as older builds do. The list keeps the MAX_KEYS games uploaded most
 * recently, so a game still being played keeps its key however many games start after it.
 */
export function uploadKey(id: string): string | null {
  try {
    const kept = JSON.parse(localStorage.getItem(KEYS) ?? '[]') as [string, string][];
    const found = kept.find(([game]) => game === id)?.[1];
    const key = found ?? Array.from(crypto.getRandomValues(new Uint8Array(16)), (b) => b.toString(16).padStart(2, '0')).join('');
    try {
      localStorage.setItem(KEYS, JSON.stringify([...kept.filter(([game]) => game !== id), [id, key]].slice(-MAX_KEYS)));
    } catch {
      // Storage full: a kept key still goes with the upload; a new one that cannot be kept is not sent.
      if (!found) return null;
    }
    return key;
  } catch {
    return null;
  }
}

/**
 * Upload one record. True once the server has it, or when it never will (refused as invalid), so the local
 * queue moves on; false to keep it queued and retry later (offline, server down).
 */
export async function uploadGameHistory(json: string): Promise<boolean> {
  if (!uploadsGames()) return false;
  const id = (JSON.parse(json) as { metadata: { game_id: string } }).metadata.game_id;
  if (!id.startsWith('web-')) return true;
  const key = uploadKey(id);
  let body: Blob = new Blob([json], { type: 'application/json' });
  if (typeof CompressionStream === 'undefined') return false;
  body = await new Response(body.stream().pipeThrough(new CompressionStream('gzip'))).blob();
  try {
    const response = await fetch(`${GAMES_URL}/v1/games/${encodeURIComponent(id)}`, {
      method: 'PUT', body, signal: AbortSignal.timeout(30000),
      headers: { 'Content-Type': 'application/octet-stream', ...(key ? { 'X-Upload-Key': key } : {}) },
    });
    // The answer is not needed. Release its body now; left unread, the timeout above would abort it 30 s later.
    await response.body?.cancel().catch(() => {});
    if (response.ok) return true;
    // 400/404/409/413/422: this record will never be accepted; retrying cannot help (409: the game is stored
    // under another upload key, for example after this browser's storage was cleared). A 403 means this site is
    // missing from the Worker's ALLOWED_ORIGINS, a setup mistake that a fix can cure, so the game stays queued.
    return [400, 404, 409, 413, 422].includes(response.status);
  } catch {
    return false;
  }
}
