# Argdown Claude Code Plugin & Marketplace — Design

- **Date:** 2026-06-07
- **Status:** Approved
- **Scope:** Package the `argdown-mcp` server as a Claude Code **plugin**, distributed through a **marketplace** hosted in this same repo. The plugin registers the MCP server (launched from a prebuilt, attested release binary), plus a usage skill and one slash command. Almost entirely additive non-Rust files; the only changes outside the plugin/marketplace are a scoped `release.yml` provenance-attestation step and a `0.1.0 → 0.1.1` workspace version bump (to cut the first attested release the plugin pins).

## Context

`argdown-mcp` is a Rust stdio MCP server exposing three tools (`parse` / `export_model` / `dung_extensions`). `release.yml` already publishes, on every version bump, per-target archives to a GitHub Release: `argdown-mcp-v<version>-<target>.tar.gz` (`.zip` for Windows) plus a `SHA256SUMS`, for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`. `v0.1.0` is already published with assets. Those release binaries are exactly what this plugin distributes — the plugin is the consumer the release pipeline was built for.

A Claude Code plugin launches an MCP server by running a `command` on the user's machine. Since the server is a compiled Rust binary, the plugin must *obtain* that binary without assuming a Rust toolchain. This design does so by downloading the matching, checksum-verified release archive on first run.

## Locked decisions

| Decision | Choice |
| --- | --- |
| Launch mechanism | **Download the prebuilt release binary** on first run, verify against `SHA256SUMS`, cache, then exec — over building from source (needs a toolchain) or expecting the binary on `PATH` (install burden). |
| Launcher runtime | A **Node** script (`launch.mjs`) invoked as the MCP `command` — Node is guaranteed in the Claude Code runtime; `sh` is not, on Windows. The launcher is a self-bootstrapping stdio proxy. |
| Version selection | **Pinned** to the plugin's version (a constant kept in sync with `plugin.json`), not "latest" — reproducible installs; the plugin version names exactly which server it ships. |
| Marketplace location | **Same repo**, single plugin in a `plugins/argdown/` subdir, over a separate marketplace repo — idiomatic for one plugin, keeps the release the single source of truth. |
| Plugin scope | MCP server **+ a usage skill + one slash command** (`/argdown:analyze`). |
| Extraction tool | System `tar` (bsdtar handles both `.tar.gz` and `.zip` on macOS / Linux / Windows 10+) — no JS archive dependency. |
| Binary verification | **SHA-256 always** (zero-dep integrity floor, checked against `SHA256SUMS`) **+ keyless provenance attestations** produced in CI (`actions/attest-build-provenance`). The launcher additionally runs `gh attestation verify` **when `gh` is on PATH** (hard-fail on mismatch) and falls back to SHA-256+TLS with a one-line note when it isn't — over requiring `gh` (re-introduces a toolchain dep) or vendoring `sigstore-js` (heavy dependency tree). Not HMAC, not a hand-managed RSA key. |

## Repository & marketplace layout

All new files are non-Rust and additive; `cargo` never sees them.

```
.claude-plugin/
  marketplace.json                  # marketplace manifest: one plugin "argdown", source ./plugins/argdown
plugins/argdown/
  .claude-plugin/plugin.json        # plugin manifest (name, version, description, author)
  .mcp.json                         # MCP server registration -> node launcher
  bin/launch.mjs                    # cross-platform download + verify + exec launcher
  commands/analyze.md               # /argdown:analyze
  skills/argdown-analysis/SKILL.md  # usage skill
```

`${CLAUDE_PLUGIN_ROOT}` resolves to `plugins/argdown/` at runtime.

`marketplace.json` lists one plugin whose `source` points at `./plugins/argdown`. `plugin.json` carries the plugin name (`argdown`), a version equal to the server release it pins, a description, and author/repo metadata.

## MCP server registration

`plugins/argdown/.mcp.json` declares one stdio server:

```json
{
  "mcpServers": {
    "argdown": {
      "command": "node",
      "args": ["${CLAUDE_PLUGIN_ROOT}/bin/launch.mjs"]
    }
  }
}
```

## The launcher (`bin/launch.mjs`)

A dependency-free Node ESM script. On invocation it bootstraps the binary if needed, then becomes a transparent stdio proxy so the MCP protocol passes straight through.

1. **Resolve target.** Map `process.platform` + `process.arch` → target triple:
   - `darwin`+`arm64` → `aarch64-apple-darwin`
   - `darwin`+`x64` → `x86_64-apple-darwin`
   - `linux`+`x64` → `x86_64-unknown-linux-gnu`
   - `win32`+`x64` → `x86_64-pc-windows-msvc`
   - anything else → write a clear message to **stderr** and exit `1`.
2. **Cache check.** Cache dir is a user location, XDG-aware: `${XDG_CACHE_HOME:-~/.cache}/argdown-mcp-plugin/v<VERSION>/`. If `argdown-mcp[.exe]` already exists there, skip to step 6 (its presence implies a prior successful verify). The cache survives plugin reinstalls/updates.
3. **Download** `https://github.com/kellenff/argdown-mcp/releases/download/v<VERSION>/argdown-mcp-v<VERSION>-<target>.<ext>` (`ext` = `zip` on Windows, else `tar.gz`) to a temp file via `fetch`/streamed write.
4. **Verify (two layers).**
   - *Integrity (always, zero-dep):* fetch `SHA256SUMS` from the same release, locate the line for this archive name, compute the downloaded archive's SHA-256, **abort on mismatch** (stderr + exit 1).
   - *Authenticity (best-effort keyless):* if `gh` is on `PATH`, run `gh attestation verify <archive> --repo kellenff/argdown-mcp --signer-workflow kellenff/argdown-mcp/.github/workflows/release.yml`; **hard-fail (exit 1) on any non-success** (bad digest, wrong identity, or no attestation found). This is unconditional because the plugin only ever pins an **attested** release (see below), so "no attestation found" never arises in normal operation — if it does, it's a signal worth failing on. If `gh` is absent, proceed and print one line to stderr noting provenance was not cryptographically verified (integrity still enforced via SHA-256 over TLS). The `--signer-workflow` pin ensures only the release workflow's identity satisfies verification, not just any workflow in the repo.
5. **Extract.** Extract with system `tar -xf` into a temp dir, `chmod +x` the binary (POSIX), then **atomically rename** the temp dir into the versioned cache dir (so concurrent session starts don't corrupt the cache).
6. **Exec.** `spawn(binaryPath, { stdio: 'inherit' })`; forward `SIGINT`/`SIGTERM`; exit with the child's exit code.

`<VERSION>` is a single pinned constant at the top of the file (initially `0.1.1` — the first attested release), kept equal to `plugin.json`'s version.

**Pure, unit-testable helpers** (exported for the test, no side effects): `targetTriple(platform, arch)`, `assetName(version, target)` / `archiveUrl`, `parseSha256Sums(text, assetName)`, and a `sha256(buffer)` wrapper. The `gh` invocation and `gh`-presence probe are isolated behind a thin function so the pure helpers stay testable offline.

## Release pipeline change (provenance attestation)

A scoped addition to `release.yml` — the one in-repo change outside the plugin dir. In the existing `release` job (which downloads artifacts, packages `dist/`, and creates the Release), after packaging and before/after `gh release create`, attest the archives:

```yaml
permissions:
  contents: write
  id-token: write        # added: OIDC for keyless signing
  attestations: write    # added: write the attestation
# ...
- uses: actions/attest-build-provenance@v4
  with:
    subject-path: 'dist/*.tar.gz,dist/*.zip'
```

This produces a Sigstore-backed, transparency-logged provenance attestation bound to each archive's digest and to the release workflow's identity. Attestations live in GitHub's attestation store (not as release assets); `gh attestation verify` fetches them by digest. No new workflow file; permissions are added to the existing `release` job only.

**The plugin pins an attested release.** `v0.1.0` predates attestation, so the rollout is sequenced: (1) land the `release.yml` attestation step; (2) bump the workspace version `0.1.0 → 0.1.1`; (3) the push to `main` triggers `release.yml`, which publishes an **attested** `v0.1.1`; (4) pin the plugin (`plugin.json` version + launcher `<VERSION>`) to `0.1.1`. The launcher therefore always targets an attested release, which is what lets the `gh`-present path hard-fail unconditionally. Because step 3 runs on GitHub Actions, the end-to-end `gh`-verification test depends on `v0.1.1` being published; the offline launcher unit tests and the SHA-256 path do not.

## Usage skill (`skills/argdown-analysis/SKILL.md`)

One skill whose `description` triggers on *analyzing argument structure, building/inspecting dialectical maps, asking which arguments survive/"win", argumentation, or working with Argdown documents*. The body teaches the workflow over the three MCP tools:
- `parse` — validate syntax and get block counts (a quick well-formedness check; a parse failure returns a byte offset).
- `export_model` — the resolved Layer B model (statements, arguments, PCS roles, dialectical edges, conflicts) as JSON, for structural reasoning.
- `dung_extensions` — the grounded extension (IN / OUT / UNDEC) under Dung's abstract argumentation framework: which arguments survive once all attacks resolve.

It states *when* to reach for each and to prefer inline `source`.

## Slash command (`commands/analyze.md`)

`/argdown:analyze $ARGUMENTS` where `$ARGUMENTS` is inline Argdown **or** a file path. The command prompt instructs Claude to: read the file if given a path; call `parse` and report validity + block summary; on success call `export_model` and `dung_extensions`; then present a concise dialectical analysis — block counts and the grounded IN/OUT/UNDEC partition, each argument tagged with a one-line reason it survives or is defeated. The tools take inline `source`, so Claude passes the document text.

## Versioning & release integration

The plugin pins the server release it ships, so two version strings move together: `plugin.json`'s `version` and the launcher's `<VERSION>` constant. When the workspace version bumps and `release.yml` publishes a new `v<version>`, bump both — a one-line addition to the release checklist. Every release from now on carries provenance attestations (the `release.yml` step above), so any pinned version is verifiable. No new CI *workflow* file is required; the only CI change is the attestation step + permissions on the existing `release` job.

## Testing & verification

- **Manifest validation:** run `plugin-dev:plugin-validator` against `marketplace.json` + `plugin.json` (schema, required fields, source path resolves).
- **Launcher unit tests** (`node --test`, no network): `targetTriple` for each supported/unsupported platform-arch pair; `assetName`/`archiveUrl` string construction; `parseSha256Sums` (picks the right line, handles the `<hash>  <name>` ordering actually emitted by `sha256sum -- *`); `sha256` against a known vector; the `gh`-presence probe returns false cleanly when `gh` is absent (no throw, soft path taken).
- **Launcher integration test** (network; runnable on this machine, darwin/arm64): invoke the bootstrap path against the real attested `v0.1.1` release, assert SHA-256 verifies, `gh attestation verify` succeeds (this machine has `gh`), and the cached `argdown-mcp` is present and executable. Also assert the negative: a tampered archive byte makes SHA-256 abort, and a wrong `--signer-workflow` makes the `gh` step hard-fail. Marked/ignored so the unit suite stays offline; depends on `v0.1.1` being published.
- **Manual e2e:** add the marketplace from the local repo path, install the plugin, and confirm in a Claude session that the three tools, the `argdown-analysis` skill, and `/argdown:analyze` all appear and work on a sample document.

## Out of scope (YAGNI / future)

- MCPB bundling; committing prebuilt binaries into the repo; an auto-"latest" channel; a `cargo install` / build-from-source fallback.
- HMAC or a hand-managed RSA/Ed25519 signing key in repo secrets (rejected in favor of keyless attestations); **requiring** `gh`; vendoring `sigstore-js` for always-on pure-Node attestation verification.
- More than one slash command; richer command output formatting.
- Targets `release.yml` does not build (Windows arm64, linux-musl, linux arm64) — the launcher errors clearly on any unsupported platform rather than guessing.
- Beyond the scoped `release.yml` attestation step and the `0.1.0 → 0.1.1` version bump, no change to the Rust workspace, the MCP server/parser/model, or other CI.

## Success criteria

- A user can add this repo as a marketplace, install the `argdown` plugin, and — with no Rust toolchain — have the three MCP tools available in a session, served by the verified `v0.1.1` binary the launcher downloaded.
- `release.yml` publishes an attested `v0.1.1`; `gh attestation verify` against a downloaded archive succeeds, and the launcher (on a machine with `gh`) enforces it; on a machine without `gh` the launcher proceeds on SHA-256 with a one-line provenance note.
- `/argdown:analyze` on a sample Argdown document reports the block summary and the grounded IN/OUT/UNDEC partition.
- The launcher refuses to run a binary whose SHA-256 doesn't match `SHA256SUMS` or whose attestation fails (when `gh` is present), and prints a clear error on an unsupported platform.
- Launcher unit tests pass offline; manifest validation is clean.
