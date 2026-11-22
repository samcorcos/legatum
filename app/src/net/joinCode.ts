// The join code a host can set on a game (plans/online-play.md): 4 to 12 letters or digits, typed once by each
// guest. The engine checks it (crates/wasm/src/host); this file only helps the text boxes. Not case sensitive.

export const JOIN_CODE_MIN = 4;
export const JOIN_CODE_MAX = 12;

/** What the host typed, with anything that is not a letter or digit dropped as it is typed. */
export function typedJoinCode(raw: string): string {
  return raw.replace(/[^A-Za-z0-9]/g, '').slice(0, JOIN_CODE_MAX);
}

/** A short plain note when the host's code cannot be used, else null. An empty box means an open game. */
export function joinCodeProblem(code: string): string | null {
  if (code === '') return null;
  return code.length < JOIN_CODE_MIN ? `A join code needs at least ${JOIN_CODE_MIN} letters or digits.` : null;
}
