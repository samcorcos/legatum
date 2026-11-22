// Supporting tools read the app's brand source. Inline fonts keep exported reports offline.
import fs from "node:fs";
import { fontsourceFile } from "../film/lib/pw.mjs";

const asset = (name) => new URL(`../../app/src/${name}`, import.meta.url);
const tokens = fs.readFileSync(asset("brand/tokens.css"), "utf8");
const logo = fs.readFileSync(asset("assets/icons/logo-legatum.svg"), "utf8");

export function lookTokens(look) {
  const block = tokens.match(new RegExp(`:root\\[data-look='${look}'\\] \\{([\\s\\S]*?)\\n\\}`));
  if (!block) throw new Error(`Unknown look: ${look}`);
  return Object.fromEntries([...block[1].matchAll(/--([\w-]+):\s*([^;]+);/g)].map((m) => [m[1], m[2].trim()]));
}

const atlas = lookTokens("atlas"), court = lookTokens("court");
export const editorial = { ...court, display: atlas.display, sans: atlas.sans };
export const wordmark = logo.replace("<svg ", '<svg role="img" aria-label="Legatum" class="wordmark" ');
export const editorialWordmark = `data:image/svg+xml;base64,${Buffer.from(logo.replaceAll("currentColor", editorial.wordmark)).toString("base64")}`;

const families = {
  cormorant: ["Cormorant", [[500, "normal"], [600, "normal"], [700, "normal"], [500, "italic"]]],
  "libre-franklin": ["Libre Franklin", [[400, "normal"], [500, "normal"], [600, "normal"], [700, "normal"]]],
  "bodoni-moda": ["Bodoni Moda", [[500, "normal"], [600, "normal"]]],
  "josefin-sans": ["Josefin Sans", [[400, "normal"], [600, "normal"], [700, "normal"]]],
};

export function toolBrandCss(variant = "court") {
  const selected = variant === "editorial" ? ["cormorant", "libre-franklin"]
    : variant === "atlas" ? ["cormorant", "libre-franklin"] : ["bodoni-moda", "josefin-sans"];
  const fonts = selected.flatMap((pkg) => {
    const [family, faces] = families[pkg];
    return faces.map(([weight, style]) => {
      const file = fontsourceFile(pkg, `${pkg}-latin-${weight}-${style}.woff2`);
      if (!file) throw new Error(`Missing bundled ${family} font; install app dependencies first`);
      return `@font-face{font-family:'${family}';font-style:${style};font-weight:${weight};font-display:swap;src:url(data:font/woff2;base64,${fs.readFileSync(file).toString("base64")}) format('woff2');}`;
    });
  }).join("\n");
  return `${fonts}\n${tokens}
:root[data-variant='editorial'] { --display: ${atlas.display}; --sans: ${atlas.sans}; --display-weight: 600; }
:root { --line: var(--rule-2); --text: var(--ink); --gold: var(--accent); --gold-dim: var(--rule-2);
  --ok: var(--good); --warn: var(--y-gold); --idle: var(--muted); --blue: var(--y-science); }
body { font-feature-settings: 'lnum' 1, 'tnum' 1; }
.wordmark { display: block; height: 20px; width: auto; color: var(--wordmark); }
`;
}
