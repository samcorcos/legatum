// The title screen's build label, so the player always knows which playtest they're running.
// Release builds set LEGATUM_BUILD_LABEL (tools/release/check.sh, e.g. "Playtest 7a"); any other
// build describes itself against the last playtest-* tag (vite.config.ts injects both). A production
// build with no playtest tag behind it (a fresh clone has none) names app/package.json's version, so
// a cloner's build never reads "Dev".
declare const __BUILD_LABEL__: string;
declare const __BUILD_DESCRIBE__: string;
declare const __APP_VERSION__: string;

/** `version` is passed for production builds only. */
export function formatBuildLabel(label: string, describe: string, version = ''): string {
  if (label) return label;
  const m = /^playtest-([\w.]+?)(?:-(\d+)-g([0-9a-f]+))?$/.exec(describe);
  if (m && !m[2]) return `Playtest ${m[1]}`;
  if (m) return `Dev · after ${m[1]} (${m[3].slice(0, 7)})`;
  if (version) return describe ? `Version ${version} (${describe.slice(0, 7)})` : `Version ${version}`;
  return describe ? `Dev (${describe.slice(0, 7)})` : 'Dev build';
}

export const buildLabel = formatBuildLabel(__BUILD_LABEL__, __BUILD_DESCRIBE__, import.meta.env.PROD ? __APP_VERSION__ : '');
