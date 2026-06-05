# GitHub Actions — Version-Gated Binary Releases — Design

- **Date:** 2026-06-04
- **Status:** Approved
- **Scope:** Add a single GitHub Actions workflow that publishes a GitHub
  Release with cross-compiled `argdown-mcp` binaries. Triggered on every push to
  `main`, but only *releases* when the workspace version in `Cargo.toml` has not
  yet been released. No crates.io publish, no general PR-CI workflow (both are
  easy follow-ups).

## Context

Rust workspace with three crates (`argdown-core`, `argdown-parser`,
`argdown-mcp`); `argdown-mcp` is the binary (currently a placeholder). Version
is workspace-wide in the root `Cargo.toml` under `[workspace.package]`
(`0.1.0`). `Cargo.lock` is committed (so builds use `--locked`). No git tags
yet. Edition 2024 → needs a recent `stable` Rust (fine on current stable).

## Decisions

Settled with the operator (`ask-user-question` + design review):

1. **Artifact — GitHub Release + binaries.** Cross-compile the `argdown-mcp`
   binary for the common desktop targets and attach archives to a GitHub
   Release with auto-generated notes. Chosen over crates.io publish (crates
   aren't publish-ready; path-only deps, no metadata) and over a source-only
   release. The placeholder binary still exercises the whole pipeline so it's
   ready when the MCP layer lands.

2. **Trigger — version-bump gate.** The workflow runs on every push to `main`,
   but a `guard` job stops early unless the workspace version has no existing
   `v<version>` release. This gives "release on push to main" semantics without
   release spam: ordinary pushes compute an already-released version and stop.
   The tag *is* the version (`v0.1.0`), created at the pushed commit.

3. **Build strategy — native-runner matrix.** One runner per OS/arch builds its
   own binary natively; `cargo test` runs natively on each and is the release
   gate. Rejected single-runner cross-compilation (cannot produce working
   macOS/Windows binaries from Linux) and `cargo-dist`/`release-plz`
   (heavyweight external tooling, overkill for a placeholder; easy to migrate to
   later).

4. **Targets — four.** Covering the common desktop platforms:

   | OS runner       | Target                       |
   | --------------- | ---------------------------- |
   | `ubuntu-latest` | `x86_64-unknown-linux-gnu`   |
   | `macos-latest`  | `aarch64-apple-darwin`       |
   | `macos-15-intel`| `x86_64-apple-darwin` (Intel)|
   | `windows-latest`| `x86_64-pc-windows-msvc`     |

   `aarch64-unknown-linux-gnu` deferred (would need `cross`).

5. **Version read — real TOML parse, no Rust in the guard.** The guard uses
   preinstalled Python 3.11+ `tomllib`:
   `tomllib.load(open("Cargo.toml","rb"))["workspace"]["package"]["version"]`.
   Robust parse, fast, no toolchain install on the hot path (every push to
   `main`).

6. **Packaging — centralized in the release job.** Build matrix jobs only build
   + test + upload the raw binary. One Ubuntu `release` job downloads all
   binaries and does the messy packaging once: `.tar.gz` for unix targets,
   `.zip` for Windows, plus a `SHA256SUMS` file. Keeps platform-specific shell
   in one place rather than duplicated across runners.

## Workflow shape (`.github/workflows/release.yml`)

- **Trigger:** `push` to `main`. **Concurrency:** `group: release`,
  `cancel-in-progress: false` (never interrupt a release). **Permissions:**
  `contents: write` (create tags + releases).
- **`guard`** (ubuntu): read version, `gh release view v$VERSION` →
  `should_release`. Outputs `version`, `should_release`.
- **`build`** (matrix, `needs: guard`, `if: should_release`): `stable` toolchain
  + target, `Swatinem/rust-cache`, `cargo test --workspace --locked`,
  `cargo build --release --locked -p argdown-mcp --target <t>`, upload the raw
  binary as an artifact named for its target.
- **`release`** (ubuntu, `needs: [guard, build]`, `if: should_release`):
  download all artifacts, package per-target, write `SHA256SUMS`,
  `gh release create v$VERSION --generate-notes --target $GITHUB_SHA dist/*`.

**Idempotency:** re-pushes don't re-release — `guard` sees the existing tag and
stops. **Safety:** a release is only cut after `cargo test` passes natively on
all four platforms.

## Out of scope (deliberate)

- crates.io publishing.
- A general PR/push CI workflow (test-on-PR). Tests here run only at release
  time; broad CI is a separate, easy follow-up.
- `aarch64-unknown-linux-gnu` and other extra targets.
- A pinned `rust-toolchain.toml` (uses `stable`; pin is a trivial add later).
