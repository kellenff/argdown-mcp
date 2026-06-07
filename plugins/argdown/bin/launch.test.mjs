import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  targetTriple,
  isWindowsTarget,
  assetName,
  archiveUrl,
  sumsUrl,
  parseSha256Sums,
  sha256,
  ghAvailable,
} from './launch.mjs';

test('targetTriple maps the four supported hosts', () => {
  assert.equal(targetTriple('darwin', 'arm64'), 'aarch64-apple-darwin');
  assert.equal(targetTriple('darwin', 'x64'), 'x86_64-apple-darwin');
  assert.equal(targetTriple('linux', 'x64'), 'x86_64-unknown-linux-gnu');
  assert.equal(targetTriple('win32', 'x64'), 'x86_64-pc-windows-msvc');
});

test('targetTriple returns null for unsupported hosts', () => {
  assert.equal(targetTriple('linux', 'arm64'), null);
  assert.equal(targetTriple('freebsd', 'x64'), null);
});

test('assetName / archiveUrl / sumsUrl construction', () => {
  assert.equal(isWindowsTarget('x86_64-pc-windows-msvc'), true);
  assert.equal(isWindowsTarget('x86_64-unknown-linux-gnu'), false);
  assert.equal(
    assetName('0.1.1', 'x86_64-unknown-linux-gnu'),
    'argdown-mcp-v0.1.1-x86_64-unknown-linux-gnu.tar.gz',
  );
  assert.equal(
    assetName('0.1.1', 'x86_64-pc-windows-msvc'),
    'argdown-mcp-v0.1.1-x86_64-pc-windows-msvc.zip',
  );
  assert.equal(
    archiveUrl('0.1.1', 'aarch64-apple-darwin'),
    'https://github.com/kellenff/argdown-mcp/releases/download/v0.1.1/argdown-mcp-v0.1.1-aarch64-apple-darwin.tar.gz',
  );
  assert.equal(
    sumsUrl('0.1.1'),
    'https://github.com/kellenff/argdown-mcp/releases/download/v0.1.1/SHA256SUMS',
  );
});

test('parseSha256Sums picks the matching line, tolerates the binary "*" marker', () => {
  const a = '1'.repeat(64);
  const b = '2'.repeat(64);
  const sums =
    `${a}  argdown-mcp-v0.1.1-x86_64-unknown-linux-gnu.tar.gz\n` +
    `${b} *argdown-mcp-v0.1.1-aarch64-apple-darwin.tar.gz\n`;
  assert.equal(parseSha256Sums(sums, 'argdown-mcp-v0.1.1-aarch64-apple-darwin.tar.gz'), b);
  assert.equal(parseSha256Sums(sums, 'argdown-mcp-v0.1.1-x86_64-unknown-linux-gnu.tar.gz'), a);
  assert.equal(parseSha256Sums(sums, 'missing.tar.gz'), null);
});

test('sha256 matches known vectors', () => {
  assert.equal(
    sha256(Buffer.from('')),
    'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
  );
  assert.equal(
    sha256(Buffer.from('abc')),
    'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
  );
});

test('parseSha256Sums handles CRLF line endings', () => {
  const hash = '1'.repeat(64);
  const sums = `${hash}  file.tar.gz\r\n`;
  assert.equal(parseSha256Sums(sums, 'file.tar.gz'), hash);
});

test('ghAvailable returns false for a missing command without throwing', () => {
  assert.equal(ghAvailable('definitely-not-a-real-binary-xyz-9999'), false);
});
