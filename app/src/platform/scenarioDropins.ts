// Scenario files the user dropped into ~/LegatumData/scenarios/lab/ (tools/scenarios/publish.sh), read by
// the desktop app's native command `scenario_dropins` (app/src-tauri/src/scenario_dropins.rs), which
// checks each file against this build's rules. The browser and phone builds have no such folder.
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { DropinFile } from '../scenarios/scenarios';

/** The folder's files, or none when this is not the desktop app or the folder cannot be read. Never throws. */
export async function loadDropins(): Promise<DropinFile[]> {
  if (!isTauri()) return [];
  try {
    const files = await invoke<DropinFile[]>('scenario_dropins');
    return Array.isArray(files) ? files : [];
  } catch {
    return [];
  }
}
