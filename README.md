# @casualtheorics/argdown-mcp

An MCP server that exposes [Argdown](https://argdown.org/) parsing, JSON export, and Dung grounded-extension reasoning to language models. Give Claude (or any MCP-capable model) the ability to parse, validate, inspect, and resolve attacks within Argdown documents — including files with `@import` directives. This server does not render HTML, SVG, dot, or PDF; it is strictly a parse/validate/export/extension tool.

## Install

**One-shot via npx:**

```bash
npx -y @casualtheorics/argdown-mcp
```

**One-shot via Yarn:**

```bash
yarn dlx @casualtheorics/argdown-mcp
```

**Claude Code (recommended for persistent access):**

```bash
claude mcp add --scope user argdown -- npx -y @casualtheorics/argdown-mcp
```

**Claude Desktop** — add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "argdown": {
      "command": "npx",
      "args": ["-y", "@casualtheorics/argdown-mcp"]
    }
  }
}
```

## What is Argdown?

[Argdown](https://argdown.org/) is a plain-text markup language for argument mapping. It lets you write complex argumentation structures — statements, arguments, attacks, supports — in a human-readable format. This server lets a language model parse and inspect those documents without needing the Argdown desktop application or CLI.

## Tools

### `parse`

Parses an Argdown document and returns a structural summary plus diagnostics.

**Input:**

```json
{ "kind": "inline" | "file", "source": "...", "path": "..." }
```

- `kind: "inline"` — pass the Argdown markup as `source`.
- `kind: "file"` — pass the filesystem path as `path`. `@import` directives resolve relative to the importer file, not CWD.

**Returns:**

- A summary line with statement, argument, and section counts.
- A diagnostic block listing any lexer/parser errors with `line:col` positions, and any plugin exceptions.
- A `(not valid Argdown)` hint when the input parses but contains only synthetic (`Untitled N`) statements with no relations, arguments, or sections — the Argdown parser is permissive, so this heuristic catches "definitely not Argdown" inputs.

**`isError: true`** when any of the following are true:

- Lexer or parser errors are present.
- A plugin threw an exception.
- The document parses to only synthetic anonymous statements with no relations, arguments, or sections.

### `export_json`

Same as `parse`, but also returns the full `IArgdownResponse.json` payload in a fenced JSON block.

**Input:** same shape as `parse`.

**Returns:** everything `parse` returns, plus a JSON block containing statements, arguments, relations, and sections as structured data.

### `dung_extensions`

Computes the grounded extension under [Dung's abstract argumentation framework](https://plato.stanford.edu/entries/argument/) — i.e., which arguments survive once every attack has been resolved. Uses Caminada's three-valued labelling algorithm.

**Input:** same shape as `parse` / `export_json`.

**Returns:**

- A summary line: `Grounded extension: N IN, M OUT, K UNDEC over A arguments and R attacks.`
- A fenced JSON block with the full partition:

```json
{
  "extension": { "in": ["..."], "out": ["..."], "undec": ["..."] },
  "argumentCount": 0,
  "attackCount": 0
}
```

**Semantics:**

- **IN** — accepted: every attacker of this argument is OUT.
- **OUT** — defeated: at least one attacker is IN.
- **UNDEC** — undecided: caught in an unresolved cycle.

**Scope:**

- Only **argument-to-argument** attack relations are considered (written `<X>\n  - <Y>` in Argdown). Statement-level attacks (`[s1]\n  - [s2]`) are intentionally ignored — Dung's framework is abstract over arguments; lifting statement attacks belongs to a structured-argumentation layer (ASPIC+, ABA), which is out of scope.
- Only the **grounded** semantics is computed. Preferred, stable, complete, semi-stable, and ideal semantics are out of scope for v0.2.
- Undercuts (`relationType: "undercut"`) target inference nodes, not arguments, and are filtered out.
- A self-attacker with no external defeater is UNDEC; with an IN external defeater, it is OUT.

## Architecture

**Plugin set:** The server runs three `@argdown/core` plugins: parse, model, and JSON export. Rendering plugins (html, svg, dot, pdf), selection/color/tag/group plugins, and `argdown.config.json` discovery are all intentionally excluded. The goal is a minimal, deterministic parse surface.

**Bundle:** The package ships as a single self-contained `dist/server.js` (~6 MB) with no runtime npm dependencies. It bundles `@argdown/core`, `@argdown/node`, the MCP SDK, and Zod via tsup, using a `createRequire` ESM shim to satisfy `@argdown/node`'s CommonJS expectations.

**Input schema:** The wire schema is a flat `{ kind, source?, path? }` object rather than a discriminated union, because the MCP SDK's `normalizeObjectSchema` does not accept `z.discriminatedUnion(...)`. Handler-side enforcement ensures the appropriate field (`source` or `path`) is present for each `kind` value.

## Troubleshooting

- **`@import` paths are relative to the importer file, not CWD.** If `main.argdown` imports `imports/sub.argdown`, the path resolves from the directory containing `main.argdown`.

- **`@import` cycles** are detected by `@argdown/node`'s IncludePlugin and surfaced in `response.exceptions`. The server will not hang.

- **`(not valid Argdown)` hint** appears when the input parses successfully but produces only synthetic `Untitled N` statements with no relations. This is the server's heuristic for catching non-Argdown content passed to `kind: "inline"`.

## Security

Path mode (`kind: "file"`) reads any file the server process can read. There is no sandboxing. Do not run this server as root, and treat its filesystem access scope as equivalent to any other tool the model can invoke (e.g., Bash).

## Platform support

Linux and macOS only. Windows is not supported (`package.json` sets `"os": ["!win32"]` and will refuse to install).

## Contributing

```bash
corepack enable            # one-time
git clone <repo>
yarn install               # no-op — zero-installs, PnP cache is committed
yarn typecheck
yarn test
yarn build
yarn smoke
```

For editor PnP integration:

```bash
yarn dlx @yarnpkg/sdks vscode   # or: idea, vim, etc.
```

## License

MIT. See the `LICENSE` file.
