// Keep in sync with crates/wasm/src/json.rs. This includes the save envelope.
export const MAX_SAVE_BYTES = 64 * 1024 * 1024;

export function checkSaveBytes(bytes: number): void {
  if (bytes > MAX_SAVE_BYTES) throw new Error('Save exceeds the 64 MiB limit');
}

export function checkSaveText(text: string): void {
  checkSaveBytes(new TextEncoder().encode(text).byteLength);
}
