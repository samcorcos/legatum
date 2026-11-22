// When this build was made, for the title screen's "Last updated" line. vite.config.ts bakes the build's
// UTC time in (LEGATUM_BUILD_TIME overrides it); the player sees it in their own time zone. A save that
// opens in an older kept build (tools/release/keep-builds.mjs) shows that build's time, since each build
// carries its own. Dev builds show nothing.
declare const __BUILD_TIME__: string;

/** "22 Nov 2026 16:39" in the given (default: the player's) time zone; '' for a missing or bad time. */
export function formatBuildTime(iso: string, timeZone?: string): string {
  const t = Date.parse(iso);
  if (!iso || Number.isNaN(t)) return '';
  const d = new Date(t);
  const date = d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric', timeZone });
  const time = d.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit', hour12: false, timeZone });
  return `${date.replace('Sept', 'Sep')} ${time}`;
}

export const buildTime = import.meta.env.PROD ? formatBuildTime(__BUILD_TIME__) : '';
