import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { cp, mkdtemp, mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
const execFileAsync = promisify(execFile);

const run = (program, args, options = {}) =>
  execFileAsync(program, args, {
    ...options,
    maxBuffer: 16 * 1024 * 1024,
  });

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const TOOL_ROOT = resolve(SCRIPT_DIR, '..');
const REPO_ROOT = resolve(TOOL_ROOT, '../..');
const ASSET_PATH = resolve(REPO_ROOT, 'crates/proto-ui-gpui/assets/proto-ui-bridge.js');
const MANIFEST_PATH = resolve(TOOL_ROOT, 'upstream.json');
const BRIDGE_ENTRY = resolve(TOOL_ROOT, 'src/index.ts');
const UPSTREAM_REPOSITORY = 'https://github.com/Proto-UI/Proto-UI.git';
const PACKAGE_MANAGER = 'pnpm@10.32.1';

function fail(message) {
  throw new Error(`[Proto UI bundle] ${message}`);
}

function assertSha(value) {
  if (!/^[0-9a-f]{40}$/.test(value)) {
    fail(`expected a lowercase 40-hex commit SHA, received ${JSON.stringify(value)}`);
  }
  return value;
}

function errorMessage(error) {
  if (error && typeof error === 'object' && 'stderr' in error && typeof error.stderr === 'string') {
    return error.stderr.trim() || String(error.message ?? error);
  }
  return error instanceof Error ? error.message : String(error);
}
async function sha256File(path) {
  return createHash('sha256').update(await readFile(path)).digest('hex');
}

async function checkoutSource(sha) {
  await mkdir(resolve(REPO_ROOT, 'target'), { recursive: true });
  const sourceRoot = await mkdtemp(join(resolve(REPO_ROOT, 'target'), 'sailbreak-proto-ui-'));
  try {
    await run('git', ['init', '--quiet', sourceRoot]);
    await run('git', ['-C', sourceRoot, 'remote', 'add', 'origin', UPSTREAM_REPOSITORY]);
    await run('git', ['-C', sourceRoot, 'fetch', '--depth', '1', 'origin', sha]);
    await run('git', ['-C', sourceRoot, 'checkout', '--quiet', '--detach', 'FETCH_HEAD']);
    return sourceRoot;
  } catch (error) {
    await rm(sourceRoot, { recursive: true, force: true });
    fail(`could not check out ${sha}: ${errorMessage(error)}`);
  }
}

async function packageJsonPaths(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const paths = [];
  for (const entry of entries) {
    if (entry.name === 'node_modules') continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      paths.push(...(await packageJsonPaths(path)));
    } else if (entry.isFile() && entry.name === 'package.json') {
      paths.push(path);
    }
  }
  return paths;
}

async function readPackageVersions(sourceRoot) {
  const packageVersions = [];
  const packagePaths = await packageJsonPaths(join(sourceRoot, 'packages'));
  for (const packagePath of packagePaths) {
    const packageJson = JSON.parse(await readFile(packagePath, 'utf8'));
    if (
      typeof packageJson.name !== 'string' ||
      !packageJson.name.startsWith('@proto.ui/') ||
      typeof packageJson.version !== 'string' ||
      !packageJson.version
    ) {
      continue;
    }
    packageVersions.push({ name: packageJson.name, version: packageJson.version });
  }
  packageVersions.sort((left, right) => left.name.localeCompare(right.name));
  if (packageVersions.length === 0) fail(`no Proto UI packages found in ${sourceRoot}`);
  return packageVersions;
}

async function buildFromSource(sourceRoot, sha, outputPath) {
  try {
    await run('corepack', ['pnpm@10.32.1', 'install', '--frozen-lockfile'], { cwd: sourceRoot });
    await run('corepack', ['pnpm@10.32.1', 'build:packages'], { cwd: sourceRoot });
  } catch (error) {
    fail(`Proto UI package build failed for ${sha}: ${errorMessage(error)}`);
  }

  const packageVersions = await readPackageVersions(sourceRoot);
  const lockfileDigest = await sha256File(join(sourceRoot, 'pnpm-lock.yaml'));
  const shadcnVersion = packageVersions.find(
    (entry) => entry.name === '@proto.ui/prototypes-shadcn'
  )?.version;
  if (!shadcnVersion) fail('Shadcn package version was not discovered');

  const scratchRoot = join(sourceRoot, '.sailbreak-proto-ui-bridge');
  const scratchEntry = join(scratchRoot, 'index.ts');
  await mkdir(scratchRoot, { recursive: true });
  await writeFile(
    join(scratchRoot, 'package.json'),
    '{"name":"sailbreak-proto-ui-bridge-scratch","private":true,"type":"module"}\n',
    'utf8'
  );
  const source = await readFile(BRIDGE_ENTRY, 'utf8');
  const metadata = JSON.stringify({ proto_ui_version: shadcnVersion, proto_ui_commit: sha });
  await writeFile(
    scratchEntry,
    `globalThis.__sailbreak_proto_ui_metadata = ${metadata};\n${source}`,
    'utf8'
  );

  try {
    await run(
      'bun',
      [
        'build',
        scratchEntry,
        '--bundle',
        '--format=iife',
        '--target=browser',
        '--outfile',
        outputPath,
      ],
      { cwd: scratchRoot }
    );
  } catch (error) {
    fail(`Bun bundle failed for ${sha}: ${errorMessage(error)}`);
  }

  const bytes = await readFile(outputPath);
  const digest = createHash('sha256').update(bytes).digest('hex');
  return { bytes, digest, lockfileDigest, packageVersions };
}
function manifestFor(sha, packageVersions, digest, lockfileDigest) {
  return {
    repository: UPSTREAM_REPOSITORY,
    ref: 'main',
    commit: sha,
    package_manager: PACKAGE_MANAGER,
    package_versions: packageVersions,
    lockfile_sha256: `sha256:${lockfileDigest}`,
    bundle_sha256: `sha256:${digest}`,
    license_sources: [
      `https://github.com/Proto-UI/Proto-UI/blob/${sha}/packages/prototypes/shadcn/THIRD_PARTY_NOTICES.md`,
      `https://github.com/Proto-UI/Proto-UI/blob/${sha}/packages/core/LICENSE`,
    ],
  };
}


async function syncHead(sha) {
  const sourceRoot = await checkoutSource(sha);
  const scratchOutput = join(sourceRoot, 'sailbreak-proto-ui-bridge.js');
  try {
    const result = await buildFromSource(sourceRoot, sha, scratchOutput);
    await cp(scratchOutput, ASSET_PATH);
    await writeFile(
      MANIFEST_PATH,
      `${JSON.stringify(manifestFor(sha, result.packageVersions, result.digest, result.lockfileDigest), null, 2)}\n`,
      'utf8'
    );
    process.stdout.write(`[Proto UI bundle] synced ${sha} sha256:${result.digest}\n`);
  } finally {
    await rm(sourceRoot, { recursive: true, force: true });
  }
}

async function checkBundle() {
  const manifest = JSON.parse(await readFile(MANIFEST_PATH, 'utf8'));
  const sha = assertSha(manifest.commit);
  if (manifest.repository !== UPSTREAM_REPOSITORY || manifest.ref !== 'main') {
    fail(`manifest source must be ${UPSTREAM_REPOSITORY}#main`);
  }
  const sourceRoot = await checkoutSource(sha);
  const scratchOutput = join(sourceRoot, 'sailbreak-proto-ui-bridge.js');
  try {
    const result = await buildFromSource(sourceRoot, sha, scratchOutput);
    const checkedIn = await readFile(ASSET_PATH);
    const expectedDigest = `sha256:${result.digest}`;
    const expectedLockfileDigest = `sha256:${result.lockfileDigest}`;
    if (!checkedIn.equals(result.bytes)) {
      fail(`generated bundle differs from ${ASSET_PATH} for ${sha}`);
    }
    if (manifest.bundle_sha256 !== expectedDigest) {
      fail(`manifest bundle digest ${manifest.bundle_sha256} does not match ${expectedDigest}`);
    }
    if (manifest.lockfile_sha256 !== expectedLockfileDigest) {
      fail(`manifest lockfile digest ${manifest.lockfile_sha256} does not match ${expectedLockfileDigest}`);
    }
    process.stdout.write(`[Proto UI bundle] check passed ${sha} ${expectedDigest}\n`);
  } finally {
    await rm(sourceRoot, { recursive: true, force: true });
  }
}

async function main() {
  const [mode, ...args] = process.argv.slice(2);
  if (mode === 'sync-head') {
    const shaIndex = args.indexOf('--sha');
    if (shaIndex < 0 || !args[shaIndex + 1]) fail('sync-head requires --sha <40-hex-sha>');
    await syncHead(assertSha(args[shaIndex + 1]));
    return;
  }
  if (mode === 'bundle:check') {
    await checkBundle();
    return;
  }
  fail(`unknown mode ${JSON.stringify(mode)}; use sync-head or bundle:check`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
