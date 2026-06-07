---
description: Analyze an Argdown document — validate it, show its structure, project the attack framework, and compute extensions (which arguments survive).
argument-hint: "[argdown text | path to a .argdown file]"
---

Produce a dialectical analysis of an Argdown document using the `argdown` MCP tools.

Input is `$ARGUMENTS` — either inline Argdown source or a path to a file.

1. If `$ARGUMENTS` is a path to an existing file, read it; otherwise treat `$ARGUMENTS` as the Argdown source text itself.
2. Call the `parse` tool with the source. Report whether it parses and the block summary (headings / statements / arguments / relations / PCS). If it fails, show the diagnostic message and byte offset, then stop.
3. On success, call:
   - `export_model` — resolved Layer B model (use `format: "yaml"` when a compact human-readable dump helps);
   - `inspect_af` — projected Dung AF (arguments, attacks, SCC metadata, acyclicity);
   - `extensions` — extension labellings under `semantics: "preferred"` (default). Do **not** call `dung_extensions` (deprecated; removed in v2).
4. Present a concise analysis:
   - the block summary;
   - AF shape — argument count, attack count, whether the graph is acyclic, and any non-trivial SCCs from `inspect_af`;
   - the preferred extension labelling — which arguments are IN (accepted), OUT (defeated), UNDEC (undecided) in the primary labelling;
   - one line per argument explaining why it survives or is defeated (e.g. "B is unattacked → IN; A is attacked by B → OUT").
   - If the user asked about a specific argument or credulous vs skeptical acceptance, also call `accepts` with the relevant `argument_id` and `mode`.

Pass the document inline as `source` to the tools.
