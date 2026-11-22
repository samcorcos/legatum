// The scenario copy lint alone (src/scenarios/lintFiles.test.ts): no WASM setup, so it starts in about a second.
// Used by tools/gates/scenario.py.
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: { environment: 'node', include: ['src/scenarios/lintFiles.test.ts'] },
});
