// The references a built web file makes, shared by check-dist-refs.mjs and carry-assets.mjs.

/** The references a file makes, as written. */
export function refsIn(text, ext) {
  const out = [];
  const add = (re, i = 1) => { for (const m of text.matchAll(re)) out.push(m[i]); };
  if (ext === '.js' || ext === '.mjs') {
    add(/new URL\(\s*(["'`])([^"'`$]+)\1\s*,\s*import\.meta\.url\s*\)/g, 2);
    add(/\bimport\(\s*(["'`])(\.{1,2}\/[^"'`$]+)\1\s*\)/g, 2);
    add(/\bfrom\s*(["'])(\.{1,2}\/[^"']+)\1/g, 2);
    // Vite's preload list (__vite__mapDeps) names chunks and styles from the site root: "assets/x.css".
    add(/(["'`])(assets\/[A-Za-z0-9_.-]+\.[a-z0-9]+)\1/g, 2);
  } else if (ext === '.css') {
    add(/url\(\s*(["']?)([^"')]+)\1\s*\)/g, 2);
  } else if (ext === '.html') {
    add(/\s(?:src|href)\s*=\s*(["'])([^"']+)\1/g, 2);
  }
  return out;
}

/** A reference resolved to a path from the site root, given the folder (from the root, '' or 'assets/') of the file that makes it. */
export function resolveRef(ref, dir) {
  const path = ref.split(/[?#]/)[0];
  if (path.startsWith('/')) return decodeURIComponent(path.slice(1));
  if (path.startsWith('assets/')) return decodeURIComponent(path);
  return decodeURIComponent(new URL(path, 'https://site/' + dir).pathname.slice(1));
}
