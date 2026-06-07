---
description: Analyze an Argdown document — validate it, show its structure, and compute the grounded extension (which arguments survive).
argument-hint: "[argdown text | path to a .argdown file]"
---

Produce a dialectical analysis of an Argdown document using the `argdown` MCP tools.

Input is `$ARGUMENTS` — either inline Argdown source or a path to a file.

1. If `$ARGUMENTS` is a path to an existing file, read it; otherwise treat `$ARGUMENTS` as the Argdown source text itself.
2. Call the `parse` tool with the source. Report whether it parses and the block summary (headings / statements / arguments / relations / PCS). If it fails, show the diagnostic message and byte offset, then stop.
3. On success, call `export_model` (resolved model) and `dung_extensions` (grounded extension).
4. Present a concise analysis:
   - the block summary;
   - the grounded partition — which arguments are IN (accepted), OUT (defeated), UNDEC (undecided);
   - one line per argument explaining why it survives or is defeated (e.g. "B is unattacked → IN; A is attacked by B → OUT").

Pass the document inline as `source` to the tools.
