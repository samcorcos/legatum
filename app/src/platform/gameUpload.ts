/**
 * Game uploads (infra/games): the hosted web build sends each game's record to a private Cloudflare R2 bucket,
 * every five turns and when the game ends, so the AI can be measured and bugs found. Every seat goes under its
 * default name ("Player 1", "AI 2", from the engine's `public_archive`), and the server refuses any other name.
 * scripts/build-web.sh bakes the address in as VITE_LEGATUM_GAMES_URL; other builds leave it empty and upload nothing.
 */
export const GAMES_URL: string = String(import.meta.env.VITE_LEGATUM_GAMES_URL ?? '').replace(/\/+$/, '');

/** README section that says what is saved and why; the title screen links to it. */
export const PRIVACY_URL = 'https://github.com/samcorcos/legatum#privacy';

export const uploadsGames = (): boolean => GAMES_URL !== '';

/** The record to keep and upload: names removed when uploads are on. */
export function historyToRecord(client: { archive(): string; publicArchive?(): string }): string {
  return uploadsGames() && client.publicArchive ? client.publicArchive() : client.archive();
}

/**
 * Upload one record. True once the server has it, or when it never will (refused as invalid), so the local
 * queue moves on; false to keep it queued and retry later (offline, server down).
 */
export async function uploadGameHistory(json: string): Promise<boolean> {
  if (!uploadsGames()) return false;
  const id = (JSON.parse(json) as { metadata: { game_id: string } }).metadata.game_id;
  if (!id.startsWith('web-')) return true;
  let body: Blob = new Blob([json], { type: 'application/json' });
  if (typeof CompressionStream === 'undefined') return false;
  body = await new Response(body.stream().pipeThrough(new CompressionStream('gzip'))).blob();
  try {
    const response = await fetch(`${GAMES_URL}/v1/games/${encodeURIComponent(id)}`, {
      method: 'PUT', body, headers: { 'Content-Type': 'application/octet-stream' }, signal: AbortSignal.timeout(30000),
    });
    if (response.ok) return true;
    // 400/403/404/413/422: this record will never be accepted; retrying cannot help.
    return response.status >= 400 && response.status < 500 && response.status !== 408 && response.status !== 429;
  } catch {
    return false;
  }
}
