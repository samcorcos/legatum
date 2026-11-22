// Bundled faces (OFL), served with the game under its CSP (no font CDN). Latin subsets only.
// Atlas: Cormorant (display, italics) + Libre Franklin (labels, numerals).
import '@fontsource/cormorant/latin-400.css';
import '@fontsource/cormorant/latin-500.css';
import '@fontsource/cormorant/latin-600.css';
import '@fontsource/cormorant/latin-700.css';
import '@fontsource/cormorant/latin-400-italic.css';
import '@fontsource/cormorant/latin-500-italic.css';
import '@fontsource/cormorant/latin-600-italic.css';
import '@fontsource/libre-franklin/latin-400.css';
import '@fontsource/libre-franklin/latin-500.css';
import '@fontsource/libre-franklin/latin-600.css';
import '@fontsource/libre-franklin/latin-700.css';

// Canvas text does not trigger @font-face loads; ask for the faces the map draws with.
if (typeof document !== 'undefined' && document.fonts?.load) {
  for (const f of [
    '700 15px "Cormorant"',
    '600 15px "Cormorant"',
    'italic 500 20px "Cormorant"',
    '600 13px "Libre Franklin"',
    '700 11px "Libre Franklin"',
  ])
    void document.fonts.load(f).catch(() => undefined);
}
