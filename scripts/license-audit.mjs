#!/usr/bin/env node
// Licence inventory for the chrome's production dependency closure (npm) and
// the Rust workspace (cargo metadata). Output: docs/evidence/test-results/licenses.json
import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const nm = join(root, 'node_modules');
const seen = new Map();
function walk(name) {
  if (seen.has(name)) return;
  const pj = join(nm, name, 'package.json');
  if (!existsSync(pj)) { seen.set(name, { version: '?', license: 'NOT-INSTALLED' }); return; }
  const p = JSON.parse(readFileSync(pj, 'utf8'));
  seen.set(name, { version: p.version, license: typeof p.license === 'string' ? p.license : JSON.stringify(p.license ?? p.licenses ?? 'UNKNOWN') });
  for (const d of Object.keys({ ...(p.dependencies || {}), ...(p.peerDependencies || {}) })) walk(d);
}
const app = JSON.parse(readFileSync(join(root, 'apps/browser-chrome/package.json'), 'utf8'));
for (const d of Object.keys(app.dependencies)) walk(d);
const npm = [...seen].map(([name, v]) => ({ name, ...v })).sort((a, b) => a.name.localeCompare(b.name));
const meta = JSON.parse(execSync('cargo metadata --format-version 1 --locked', { cwd: root, maxBuffer: 64 << 20 }).toString());
const cargo = meta.packages.filter((p) => !p.source?.startsWith?.('path') && p.source).map((p) => ({ name: p.name, version: p.version, license: p.license ?? 'UNKNOWN' }));
const tally = (xs) => xs.reduce((m, x) => ((m[x.license] = (m[x.license] || 0) + 1), m), {});
const out = { generatedAt: new Date().toISOString(), npmProductionClosure: { count: npm.length, byLicense: tally(npm), packages: npm }, cargoThirdParty: { count: cargo.length, byLicense: tally(cargo), packages: cargo } };
writeFileSync(join(root, 'docs/evidence/test-results/licenses.json'), JSON.stringify(out, null, 2) + '\n');
console.log('npm', out.npmProductionClosure.count, out.npmProductionClosure.byLicense);
console.log('cargo', out.cargoThirdParty.count, out.cargoThirdParty.byLicense);
const atlaskit = npm.filter((p) => p.name.startsWith('@atlaskit/') && p.license !== 'Apache-2.0');
if (atlaskit.length) console.log('NON-APACHE @atlaskit:', atlaskit.map((p) => `${p.name}@${p.version}:${p.license}`).join(', '));
