#!/usr/bin/env node
// Pins Parcel's inline <script type=importmap> (used for lazily loaded ADS
// theme chunks) into the CSP by SHA-256, so script-src stays 'self' + hash
// with no 'unsafe-inline'. Also writes a file manifest with hashes.
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dir = join(root, process.argv[2] ?? 'apps/browser-chrome/dist/app');
for (const f of readdirSync(dir).filter((x) => x.endsWith('.html'))) {
  let html = readFileSync(join(dir, f), 'utf8');
  const hashes = [...html.matchAll(/<script type=importmap>([\s\S]*?)<\/script>/g)].map((m) => `'sha256-${createHash('sha256').update(m[1]).digest('base64')}'`);
  html = html.replace(/script-src 'self'[^;"]*/, `script-src 'self'${hashes.length ? ' ' + hashes.join(' ') : ''}`);
  writeFileSync(join(dir, f), html);
  console.log(`${f}: pinned ${hashes.length} import map(s) in CSP`);
}
const files = readdirSync(dir).filter((f) => f !== 'manifest.json' && statSync(join(dir, f)).isFile()).sort();
writeFileSync(join(dir, 'manifest.json'), JSON.stringify({ generatedBy: 'scripts/finalize-chrome-bundle.mjs', files: files.map((p) => ({ path: p, bytes: statSync(join(dir, p)).size, sha256: createHash('sha256').update(readFileSync(join(dir, p))).digest('hex') })) }, null, 2) + '\n');
