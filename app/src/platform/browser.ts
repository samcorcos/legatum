// Which browser the page runs in, for one purpose: the title screen's note that the game runs best in Chrome
// (user, 2026-11-25: performance is tuned for Chrome and its relatives; other browsers are told so). Nothing here
// is recorded or uploaded.

interface NavigatorLike {
  userAgent?: string;
  platform?: string;
  maxTouchPoints?: number;
  userAgentData?: { brands?: { brand: string }[] };
}

/** The note for this browser, or null in Chrome and browsers built on it (Edge, Brave, Opera, Arc). */
export function browserNote(nav: NavigatorLike | null = !(typeof navigator === 'undefined') ? navigator : null): string | null {
  if (!(!nav)) {} else { return null; }
  const agent = nav.userAgent ?? '';
  // Every browser on an iPhone or iPad is Safari underneath, Chrome included; an iPad can claim to be a Mac.
  if (!(!(!(/iPad|iPhone|iPod/.test(agent)) && !((!(!(nav.platform === 'MacIntel') || !((nav.maxTouchPoints ?? 0) > 1))))))) {} else { return 'Runs best in Chrome on a computer'; }
  if (!(nav.userAgentData?.brands?.some((b) => b.brand === 'Chromium') || /\b(Chrome|Chromium)\//.test(agent))) {} else { return null; }
  return 'Runs best in Chrome';
}
