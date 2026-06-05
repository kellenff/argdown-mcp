# GitHub Actions — PR Checks (CI) — Design

- **Date:** 2026-06-04
- **Status:** Approved
- **Scope:** Add a `ci` workflow that gates pull requests (and pushes to `main`)
  on formatting, lints, build, and tests. Sibling to the release workflow
  (`2026-06-04-github-actions-release-design.md`), which only tests at
  version-bump time; this gives per-PR signal.

## Context

The release workflow runs `cargo test` natively across four targets, but only
when the version bumps — so ordinary PRs got no automated checking. The tree
currently passes `cargo fmt --all --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, and `cargo test --workspace` locally, so all
gates can be enforced from the start without a remediation pass.

## Decisions

Settled with the operator (`ask-user-question`):

1. **Checks — fmt + clippy + build + test.** All four. `clippy` is enforced at
   `-D warnings` (any lint fails CI), matching the repo's clippy-clean discipline
   (e.g. the canonical-import-order commit) and the operator's "automatic
   formatters / one canonical style" preference. `cargo build` is included
   explicitly even though `cargo test` also compiles.

2. **Scope — full four-target matrix.** `clippy`, `build`, and `test` run on all
   four release targets natively:

   | OS runner        | Target                     |
   | ---------------- | -------------------------- |
   | `ubuntu-latest`  | `x86_64-unknown-linux-gnu` |
   | `macos-latest`   | `aarch64-apple-darwin`     |
   | `macos-15-intel` | `x86_64-apple-darwin`      |
   | `windows-latest` | `x86_64-pc-windows-msvc`   |

3. **`fmt` runs once, not per target.** rustfmt output is platform-independent;
   running it on four runners would add cost with zero added coverage. It lives
   in its own ubuntu `fmt` job. The four-target scope (decision 2) governs the
   compile/test checks, where platform differences are real.

## Workflow shape (`.github/workflows/ci.yml`)

- **Triggers:** `pull_request` into `main`, and `push` to `main` (keeps the
  default branch green and gives a status signal there too).
- **Concurrency:** `group: ci-<ref>`, `cancel-in-progress: true` — superseded
  runs on the same ref are cancelled (cheap, fast PR feedback).
- **Permissions:** `contents: read` (CI reads only).
- **`fmt`** (ubuntu): `cargo fmt --all --check`.
- **`check`** (matrix): `stable` toolchain + clippy component,
  `Swatinem/rust-cache`, then `clippy --all-targets -- -D warnings`, `build
  --workspace --locked`, `test --workspace --locked`. Native runner per target,
  so no `--target` flag is needed.

`fail-fast: false` so one platform's failure still reports the others.

## Out of scope

- Code coverage, MSRV matrix, doc builds, cargo-audit/deny — separate follow-ups.
- Required-status-check branch protection (a repo setting, not a workflow file).
