import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { compile } from 'json-schema-to-typescript';

const root = fileURLToPath(new URL('../../', import.meta.url));
const schema = JSON.parse(
  execFileSync('uv', ['run', 'python', 'shell/scripts/export_contracts.py'], {
    cwd: root,
    encoding: 'utf8',
  }),
);
const result = await compile(schema, 'BloomMessage', {
  bannerComment: '/* Generated from Python/Pydantic. Run npm run contracts; do not edit. */',
  style: { singleQuote: true },
});
const output = new URL('../src/contracts.generated.ts', import.meta.url);
if (process.argv.includes('--check')) {
  if ((await readFile(output, 'utf8')) !== result) {
    throw new Error('Generated contracts are stale. Run npm run contracts.');
  }
} else {
  await writeFile(output, result);
}
