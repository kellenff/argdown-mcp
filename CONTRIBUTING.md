# Contributing

## Prerequisites

- Node.js >=24
- `corepack enable` (one-time, picks up the pinned Yarn version)

## Setup

```sh
git clone https://github.com/casualtheorics/argdown-mcp.git
yarn install        # no-op on clean clone (zero-installs)
yarn dlx @yarnpkg/sdks vscode  # or vim, idea — for editor PnP integration
```

## Workflow

| Command              | Purpose                                                                                                        |
| -------------------- | -------------------------------------------------------------------------------------------------------------- |
| `yarn typecheck`     | Strict TS type-check (no emit)                                                                                 |
| `yarn test`          | Vitest suite (in-process via InMemoryTransport)                                                                |
| `yarn build`         | tsup ESM bundle into `dist/server.js`                                                                          |
| `yarn bundle-sanity` | Assert build is shippable (size, no native deps, no jsdom/puppeteer leakage, smoke-runs a JSON-RPC initialize) |
| `yarn smoke`         | Composite end-to-end smoke (Inspector + tarball-install + SIGPIPE)                                             |
| `yarn dev`           | `tsup --watch`                                                                                                 |

## Releasing

1. Bump `version` in `package.json` (semver — start at 0.1.0).
2. Update CHANGELOG if you keep one (we don't yet).
3. `yarn build && yarn typecheck && yarn test && yarn smoke` — all green.
4. `yarn npm publish --dry-run --access public` — verify the would-be tarball.
5. `yarn release` — actually publishes (= `yarn npm publish --access public`).
6. `git tag v$(node -p "require('./package.json').version")` and push tags.

## Code Style

- ESM only (`.js` extensions on TS imports per NodeNext).
- No emojis in code or commit messages.
- Bug fixes prefer minimal-diff over refactor.

## Out of Scope

- Windows is not supported (`package.json#os: ["!win32"]`).
- No HTML/SVG/PDF rendering — see README's Architecture section.
