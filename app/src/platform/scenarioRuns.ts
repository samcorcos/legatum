// A finished (or abandoned) scenario run is saved as a download of the run file, which the player
// can send back as training data.
import { runFileName, type ScenarioRun } from '../scenarios/scenarios';

export interface SavedRun {
  /** The downloaded file's name. */
  path: string;
}

function download(json: string, name: string): void {
  const url = URL.createObjectURL(new Blob([json], { type: 'application/json' }));
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  try { link.click(); } finally { setTimeout(() => URL.revokeObjectURL(url), 1000); }
}

export async function saveScenarioRun(run: ScenarioRun): Promise<SavedRun> {
  const name = runFileName(run);
  download(JSON.stringify(run), name);
  return { path: name };
}
