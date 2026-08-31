import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import assert from 'node:assert/strict';

const manifestPath = new URL('../upstream.json', import.meta.url);
const assetPath = new URL('../../../crates/proto-ui-gpui/assets/proto-ui-bridge.js', import.meta.url);

await test('upstream.json records one exact Proto UI main commit and bundle identity', async () => {
  const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  assert.match(manifest.repository, /^https:\/\/github\.com\/Proto-UI\/Proto-UI\.git$/);
  assert.equal(manifest.ref, 'main');
  assert.match(manifest.commit, /^[0-9a-f]{40}$/);
  assert.equal(manifest.package_manager, 'pnpm@10.32.1');
  assert.match(manifest.bundle_sha256, /^sha256:[0-9a-f]{64}$/);
  assert.match(manifest.lockfile_sha256, /^sha256:[0-9a-f]{64}$/);
  assert.ok(Array.isArray(manifest.package_versions));
  assert.ok(manifest.package_versions.every((entry) => typeof entry.name === 'string'));
  assert.ok(manifest.package_versions.some((entry) => entry.name === '@proto.ui/prototypes-shadcn'));
  assert.ok(Array.isArray(manifest.license_sources));
});

await test('checked-in bundle exists and is non-empty', async () => {
  const bytes = await readFile(assetPath);
  assert.ok(bytes.length > 0, 'proto-ui-bridge.js must be a non-empty asset');
});