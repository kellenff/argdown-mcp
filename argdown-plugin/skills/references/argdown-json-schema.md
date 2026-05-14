# Argdown JSON schema reference

> Captured from @argdown/core 2.0.1 via `yarn schema-probe`.
> Snapshot: `argdown-plugin/.schema-probe.snapshot.json`.

## Top-level keys

The `json` field on an `export_json` MCP response parses to an object with these keys (order as observed in snapshot):

- `arguments` — `Record<title, IArgument>`
- `statements` — `Record<title, IEquivalenceClass>`
- `relations` — `IRelation[]`
- `sections` — `ISection[]` (preserves document order)
- `tags` — `Record<string, unknown>` (empty in probe; shape not captured)

## `statements` (Record<title, IEquivalenceClass>)

Keyed by statement title. Each value:

| Field                       | Type                  | Notes                               |
| --------------------------- | --------------------- | ----------------------------------- |
| `type`                      | `"equivalence-class"` | literal                             |
| `title`                     | string                | same as key                         |
| `relations`                 | `IRelation[]`         | relations involving this statement  |
| `members`                   | `IStatement[]`        | one entry per occurrence in source  |
| `section`                   | string (id)           | section containing first occurrence |
| `isUsedAsTopLevelStatement` | boolean               |                                     |
| `isUsedAsRelationStatement` | boolean               |                                     |

Each `members` entry: `{ type, role, title, text?, isTopLevel?, isReference?, startLine, startColumn, endLine, endColumn, section }`. `role` observed values: `"top-level-statement"`, `"relation-statement"`.

## `arguments` (Record<title, IArgument>)

Keyed by argument title. Each value:

| Field       | Type           | Notes                                                                   |
| ----------- | -------------- | ----------------------------------------------------------------------- |
| `type`      | `"argument"`   | literal                                                                 |
| `title`     | string         | same as key                                                             |
| `relations` | `IRelation[]`  |                                                                         |
| `members`   | `IStatement[]` | description occurrences; `role: "argument-description"`                 |
| `pcs`       | array          | premise-conclusion structure; empty in probe (no `#+ / #-` syntax used) |
| `section`   | string (id)    |                                                                         |

`pcs` entries (not shown in probe) are statement-with-role rows where `role` is `"premise"` or `"conclusion"`.

## `relations` (IRelation[])

Top-level flat array of all relations. Each entry:

```
{
  type: "relation",
  relationType: string,
  from: string,       // title of source node
  fromType: string,   // "equivalence-class" | "argument"
  to: string,
  toType: string
}
```

**`relationType` values observed in probe:** `"support"`, `"attack"`. Other values (`entails`, `contrary`, `contradictory`, `undercut`) may appear in larger documents with those relation types declared.

Note: relations also appear denormalized on each `statement` and `argument` entry.

## `sections` (ISection[])

Array in document order. Each entry:

| Field                                                 | Type                           |
| ----------------------------------------------------- | ------------------------------ |
| `type`                                                | `"section"`                    |
| `id`                                                  | string (e.g. `"s1"`)           |
| `level`                                               | number (heading depth)         |
| `title`                                               | string                         |
| `children`                                            | `ISection[]` (nested sections) |
| `ranges`                                              | array (empty in probe)         |
| `startLine` / `startColumn` / `endLine` / `endColumn` | number                         |

## `tags`

`Record<string, unknown>`. Empty object in the probe fixture (no tags used in probe input). Consult `@argdown/core/dist/model/` for the full type definition.

## What is NOT in `response.json`

`JSONExportPlugin` serialises only the model graph. Absent:

- `tokens`
- `lexerErrors`, `parserErrors`, `exceptions` — live on the raw `IArgdownResponse`, not serialised
- Raw source text or line comments
- Per-statement source-file attribution (`@import` resolves textually before parsing)

**What does survive:** `startLine`, `startColumn`, `endLine`, `endColumn` on `IStatement` members (from `HasLocation`).

## See also

- Source: `argdown-plugin/.schema-probe.snapshot.json` (verbatim probe output)
- Upstream types: `@argdown/core/dist/model/*` (regenerate via `yarn schema-probe`)
