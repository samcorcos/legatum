/** Synthetic fixture edits start a partial history. Keep opaque u64 state tokens exact. */
export function withoutArchive(save: string): string {
  const protectedSave = save.replace(/([:[,]\s*)(\d{16,})(?=\s*[,}\]])/g, '$1"u64:$2"');
  const state = JSON.parse(protectedSave);
  delete state.archive;
  return JSON.stringify(state).replace(/"u64:(\d+)"/g, '$1');
}
