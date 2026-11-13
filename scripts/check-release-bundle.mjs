#!/usr/bin/env node
// Guards the production chrome bundle (apps/browser-chrome/dist/app):
//  - no development fixture data or fixture-only brand favicons
//  - no external font/CDN downloads, no inline <script>
//  - lists assets whose licence is still "needs-review" (fails with --release)
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dist = join(root, 'apps/browser-chrome/dist/app');
const release = process.argv.includes('--release');
const files = readdirSync(dist).filter((f) => statSync(join(dist, f)).isFile());
const errors = [], warnings = [];
const fixtureOnly = new Set([...readdirSync(join(root, 'assets/icons/favicons')), ...readdirSync(join(root, 'assets/icons/favorites'))].map((f) => f.replace(/\.svg$/, '')));
const needsReview = ['codex', 'claude-code', 'claude-code-activity', 'chatgpt'];
for (const f of files) {
  const base = f.replace(/\.[0-9a-f]{8}\.svg$/, '');
  if (f.endsWith('.svg') && fixtureOnly.has(base) && !needsReview.includes(base)) errors.push(`fixture-only brand asset in app bundle: ${f}`);
  if (f.endsWith('.svg') && needsReview.includes(base)) (release ? errors : warnings).push(`asset needs owner/legal review before release: ${f}`);
  if (/\.(js|css|html)$/.test(f)) {
    const t = readFileSync(join(dist, f), 'utf8');
    for (const bad of ['DESIGN FIXTURE', 'slide-fixture', 'createFixtureBridge', 'FIXTURE_FAVICONS', 'grant_fixture']) if (t.includes(bad)) errors.push(`${f} contains development-only marker "${bad}"`);
    for (const url of ['fonts.googleapis.com', 'fonts.gstatic.com', 'use.typekit', 'cdn.jsdelivr', 'unpkg.com']) if (t.includes(url)) errors.push(`${f} references external host ${url}`);
    if (f.endsWith('.html')) {
      for (const m of t.matchAll(/<script(?![^>]*\bsrc=)([^>]*)>([\s\S]*?)<\/script>/g)) {
        const h = `'sha256-${createHash('sha256').update(m[2]).digest('base64')}'`;
        if (!/type=importmap/.test(m[1]) || !t.includes(h)) errors.push(`${f} contains an inline <script> not pinned by hash in the CSP`);
      }
      if (/unsafe-eval|script-src[^;]*unsafe-inline/.test(t)) errors.push(`${f} CSP allows unsafe script execution`);
      if (!/connect-src 'none'/.test(t)) errors.push(`${f} CSP must forbid network connections from the chrome`);
    }
  }
}
const html = files.filter((f) => f.endsWith('.html'));
if (!html.length) errors.push('no HTML entry in dist/app');
for (const w of warnings) console.warn(`warning: ${w}`);
if (errors.length) { for (const e of errors) console.error(`error: ${e}`); process.exit(1); }
console.log(`release bundle check passed (${files.length} files${warnings.length ? `, ${warnings.length} licence warnings` : ''})`);
