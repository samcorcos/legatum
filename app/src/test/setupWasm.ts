import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { initSync } from '../wasm-pkg/legatum.js';

initSync({ module: readFileSync(resolve(process.cwd(), 'src/wasm-pkg/legatum_bg.wasm')) });
