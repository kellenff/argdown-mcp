#!/usr/bin/env node
// Argdown MCP plugin launcher — pure helpers.
//
// The download/verify/exec bootstrap is appended in a later step; this section
// is side-effect-free and unit-tested.

import { createHash } from 'node:crypto';
import { spawn, spawnSync } from 'node:child_process';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  renameSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';
import { pathToFileURL } from 'node:url';

export const VERSION = '0.1.1';
const REPO = 'kellenff/argdown-mcp';

export function targetTriple(platform, arch) {
  const map = {
    'darwin:arm64': 'aarch64-apple-darwin',
    'darwin:x64': 'x86_64-apple-darwin',
    'linux:x64': 'x86_64-unknown-linux-gnu',
    'win32:x64': 'x86_64-pc-windows-msvc',
  };
  return map[`${platform}:${arch}`] ?? null;
}

export function isWindowsTarget(target) {
  return target.includes('windows');
}

export function assetName(version, target) {
  const ext = isWindowsTarget(target) ? 'zip' : 'tar.gz';
  return `argdown-mcp-v${version}-${target}.${ext}`;
}

export function archiveUrl(version, target) {
  return `https://github.com/${REPO}/releases/download/v${version}/${assetName(version, target)}`;
}

export function sumsUrl(version) {
  return `https://github.com/${REPO}/releases/download/v${version}/SHA256SUMS`;
}

// `sha256sum -- *` emits "<64-hex>  <name>" (text mode, two spaces). The optional
// "*" binary-mode marker before the name is tolerated defensively.
export function parseSha256Sums(text, name) {
  for (const line of text.split(/\r?\n/)) {
    const m = line.match(/^([0-9a-f]{64})\s+\*?(.+)$/);
    if (m && m[2].trim() === name) return m[1];
  }
  return null;
}

export function sha256(buffer) {
  return createHash('sha256').update(buffer).digest('hex');
}

export function ghAvailable(cmd = 'gh') {
  try {
    return spawnSync(cmd, ['--version'], { stdio: 'ignore' }).status === 0;
  } catch {
    return false;
  }
}

// ---------- side-effecting bootstrap ----------

const SIGNER_WORKFLOW = 'kellenff/argdown-mcp/.github/workflows/release.yml';

function die(msg) {
  process.stderr.write(`argdown plugin launcher: ${msg}\n`);
  process.exit(1);
}

async function fetchBuffer(url) {
  const res = await fetch(url, { redirect: 'follow' });
  if (!res.ok) throw new Error(`download failed (${res.status}) for ${url}`);
  return Buffer.from(await res.arrayBuffer());
}

function cacheDir(version) {
  const base = process.env.XDG_CACHE_HOME || join(homedir(), '.cache');
  return join(base, 'argdown-mcp-plugin', `v${version}`);
}

async function ensureBinary() {
  const target = targetTriple(process.platform, process.arch);
  if (!target) {
    die(`unsupported platform ${process.platform}/${process.arch}; no prebuilt argdown-mcp binary is published for it.`);
  }
  const exe = isWindowsTarget(target) ? 'argdown-mcp.exe' : 'argdown-mcp';
  const dir = cacheDir(VERSION);
  const cachedBin = join(dir, exe);
  if (existsSync(cachedBin)) return cachedBin;

  const name = assetName(VERSION, target);
  const archive = await fetchBuffer(archiveUrl(VERSION, target));
  const sumsText = (await fetchBuffer(sumsUrl(VERSION))).toString('utf8');

  // Integrity: SHA-256 against SHA256SUMS (always).
  const expected = parseSha256Sums(sumsText, name);
  if (!expected) die(`no SHA256SUMS entry for ${name}`);
  const actual = sha256(archive);
  if (actual !== expected) {
    die(`SHA-256 mismatch for ${name}\n  expected ${expected}\n  got      ${actual}`);
  }

  // Stage on the same filesystem as the cache so the final publish rename is atomic.
  const cacheBase = dirname(dir);
  mkdirSync(cacheBase, { recursive: true });
  const work = mkdtempSync(join(cacheBase, '.tmp-'));
  try {
    const archivePath = join(work, name);
    writeFileSync(archivePath, archive);

    // Authenticity: keyless provenance attestation when `gh` is present.
    if (ghAvailable()) {
      const r = spawnSync(
        'gh',
        ['attestation', 'verify', archivePath, '--repo', REPO, '--signer-workflow', SIGNER_WORKFLOW],
        { stdio: ['ignore', 'ignore', 'inherit'] },
      );
      if (r.status !== 0) die(`provenance attestation verification failed for ${name}`);
    } else {
      process.stderr.write(
        'argdown plugin launcher: `gh` not found; proceeding on SHA-256 + TLS (provenance not cryptographically verified)\n',
      );
    }

    // Extract with system tar (bsdtar handles .tar.gz and .zip on macOS/Linux/Win10+).
    const extractDir = join(work, 'x');
    mkdirSync(extractDir);
    const ex = spawnSync('tar', ['-xf', archivePath, '-C', extractDir], {
      stdio: ['ignore', 'ignore', 'inherit'],
    });
    if (ex.status !== 0) die(`failed to extract ${name} (is \`tar\` available?)`);

    const extractedBin = join(extractDir, exe);
    if (!existsSync(extractedBin)) die(`archive ${name} did not contain ${exe}`);
    if (process.platform !== 'win32') chmodSync(extractedBin, 0o755);

    try {
      renameSync(extractDir, dir); // atomic publish into the version cache
    } catch {
      // Another session published first (rename onto a non-empty dir fails); use theirs.
      if (!existsSync(cachedBin)) throw new Error(`failed to publish binary to ${dir}`);
    }
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
  return cachedBin;
}

function run(binPath) {
  const child = spawn(binPath, [], { stdio: 'inherit' });
  const forward = (sig) => () => {
    try {
      child.kill(sig);
    } catch {
      /* already exited */
    }
  };
  process.on('SIGINT', forward('SIGINT'));
  process.on('SIGTERM', forward('SIGTERM'));
  child.on('error', (e) => die(`failed to exec argdown-mcp: ${e.message}`));
  child.on('exit', (code, signal) => {
    if (signal) process.kill(process.pid, signal);
    else process.exit(code ?? 0);
  });
}

// Only bootstrap when run as the entry point — never on `import` (keeps tests offline).
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  ensureBinary()
    .then(run)
    .catch((e) => die(e.message));
}
