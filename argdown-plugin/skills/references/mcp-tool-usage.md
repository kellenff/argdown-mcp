# Calling the argdown-mcp tools

The plugin registers `argdown-mcp` via `.mcp.json`. Two tools are available:
`parse` and `export_json`. Both accept the same input shape.

## Input shape (both tools)

Flat object (not a discriminated union — MCP SDK serialisation limitation):

```jsonc
{
  "kind": "inline" | "file",
  "source": "...",   // when kind === "inline"
  "path":   "..."    // when kind === "file"; @import resolves relative to this file
}
```

The handler enforces exactly-one-of source/path based on `kind`; missing the required field returns `isError: true` with a recovery hint.

## `parse` — when to use it

Use `parse` for **go/no-go questions and summary counts**:

- "Does this argdown parse without errors?" → check `isError` + diagnostic block
- "How many statements / arguments / sections?" → first line of the text payload
- "(not valid Argdown)" hint flags inputs the parser tolerates but that lack titled statements, arguments, sections, or relations.

### Example: parse, inline, valid input

Request:

```json
{
  "name": "parse",
  "arguments": {
    "kind": "inline",
    "source": "# Section\n\n[Premise A]: First premise.\n\n<Argument>: The conclusion.\n  + [Premise A]\n"
  }
}
```

Response text payload:

```
Parsed 2 statements (by source-location), 1 arguments, 1 sections.
```

`isError` absent (success).

### Example: parse, inline, with errors

Malformed input (e.g. `"[a]: x\n  + "`) — incomplete relation marker:

```
Parsed 1 statements (by source-location), 0 arguments, 0 sections.
Errors:
  parse L2:5: ...
```

`isError: true`.

## `export_json` — when to use it

Use `export_json` for **any structural reasoning** that needs the model:

- Walk `arguments[].pcs[]` to find premises
- Traverse `relations[]` for support/attack chains
- Walk `sections[]` for document order
- Find unsupported statements; detect cycles; classify rebuttals

### Example: export_json, inline

Request:

```json
{
  "name": "export_json",
  "arguments": {
    "kind": "inline",
    "source": "# Section\n\n[Premise A]: First premise.\n\n[Premise B]: Second premise.\n  + [Premise A]\n\n[Some target]: The target statement.\n\n<Argument>: The conclusion.\n  + [Premise A]\n  + [Premise B]\n  -> [Some target]\n"
  }
}
```

Response text payload starts with the summary line, then a fenced JSON block:

````
Parsed 3 statements (by source-location), 1 arguments, 1 sections.
JSON:
```json
{
  "statements": { "Premise A": { ... }, "Premise B": { ... }, "Some target": { ... } },
  "arguments": { "Argument": { "pcs": [], "relations": [...], ... } },
  "relations": [
    { "relationType": "support", "from": "Premise A", "fromType": "equivalence-class", "to": "Premise B", "toType": "equivalence-class" },
    ...
  ],
  "sections": [{ "id": "s1", "level": 1, "title": "Section", ... }],
  "tags": {}
}
````

Extract the fenced JSON, `JSON.parse` it, and walk the structure. See `argdown-json-schema.md` for the keys.

## Decision matrix (which tool?)

| Query                          | Tool          |
| ------------------------------ | ------------- |
| Does this parse?               | `parse`       |
| How many statements?           | `parse`       |
| Locate parse error             | `parse`       |
| Find premises with no supports | `export_json` |
| Trace support chain            | `export_json` |
| Detect cycles                  | `export_json` |
| Generate counter-argument      | `export_json` |
| Walk document in section order | `export_json` |

Rule of thumb: **`parse` for the diagnostic; `export_json` for the structure**.

## When BOTH tools matter

`extract-argument` writes Argdown from prose, then calls `parse` to validate the output before showing the user. This is the only skill that produces Argdown (rather than reading it).

## Common mistakes

- Calling `parse` for a query that needs the AST — you'll have to call again with `export_json`.
- Forgetting `kind: "inline"` — handler returns `isError: true`.
- Calling `kind: "file"` with a relative path when CWD differs from the file's location — `@import` resolution may surprise.

## See also

- `argdown-json-schema.md` — top-level keys and types
- `.schema-probe.snapshot.json` — captured example response
