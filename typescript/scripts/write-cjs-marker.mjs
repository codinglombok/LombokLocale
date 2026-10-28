// Writes the CommonJS marker into dist/cjs so `require()` of the dual build
// resolves to CommonJS inside an otherwise ESM package. Run by `npm run build`
// after the two tsc passes (no arguments, no configuration).
import { writeFileSync } from 'node:fs';

writeFileSync('dist/cjs/package.json', '{"type":"commonjs"}\n');
