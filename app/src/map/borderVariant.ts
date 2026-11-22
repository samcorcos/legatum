// Dev-only switch for the territory border look, so the variants can be compared in the real map
// until one is picked. Choose with `?borders=current|a|b|c` in the URL (remembered in this browser
// under `legatum.dev.borders`), or at runtime with `window.legatumMap.setBorderVariant(v)`.
export const BORDER_VARIANTS = ['current', 'a', 'b', 'c'] as const;
export type BorderVariant = (typeof BORDER_VARIANTS)[number];

/** What a player sees when nothing has been chosen. */
export const DEFAULT_BORDER_VARIANT: BorderVariant = 'c';

const KEY = 'legatum.dev.borders';
const parse = (v: string | null | undefined): BorderVariant | null => (BORDER_VARIANTS as readonly string[]).includes(v ?? '') ? (v as BorderVariant) : null;

function initial(): BorderVariant {
  const fromUrl = parse(new URLSearchParams(globalThis.location?.search ?? '').get('borders'));
  if (fromUrl) {
    try { globalThis.localStorage?.setItem(KEY, fromUrl); } catch { /* storage may be blocked */ }
    return fromUrl;
  }
  try { return parse(globalThis.localStorage?.getItem(KEY)) ?? DEFAULT_BORDER_VARIANT; } catch { return DEFAULT_BORDER_VARIANT; }
}

let active: BorderVariant = initial();
export const getBorderVariant = (): BorderVariant => active;
export function setBorderVariant(v: BorderVariant) {
  active = v;
  try { globalThis.localStorage?.setItem(KEY, v); } catch { /* storage may be blocked */ }
}
