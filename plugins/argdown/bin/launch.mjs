#!/usr/bin/env node
// Argdown MCP plugin launcher — pure helpers.
//
// The download/verify/exec bootstrap is appended in a later step; this section
// is side-effect-free and unit-tested.

import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';

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
