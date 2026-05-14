---
name: rebut-argument
description: This skill should be used when the user asks to "rebut this argument", "steelman a counter-argument to", "how would someone disagree with", or wants to generate a principled counter-position to a named Argdown argument. Calls the export_json MCP tool and applies Pollock's rebutting/undercutting frame. SKIP: when the user asks to find weak premises broadly — that's find-unsupported-premises. SKIP: when the user asks to validate syntax — that's validate-argdown.
---

# rebut-argument

Generate a principled counter to a named argument. Apply Pollock.

## Required MCP tools

- `mcp__argdown-mcp__export_json`

## Workflow

1. Identify the target argument by title (e.g., `<Argument>`).
2. Call `export_json`; extract the fenced JSON.
3. Locate the argument in `arguments[<title>]`. Read its `pcs[]` (premise-conclusion structure).
4. Apply Pollock's distinction (see [references/argumentation-theory.md](../references/argumentation-theory.md)):
   - **Rebutting defeater** — attack a premise directly. Best when one premise is structurally weakest (acceptability/grounding low, in the Govier sense) or factually contestable.
   - **Undercutting defeater** — attack the inferential warrant (the move from premises to conclusion). Best when premises are individually strong but the inference is loose, contextually limited, or assumes an unstated bridge.
5. Inspect `relations[]` for inbound `support` edges to the conclusion from elsewhere in the document. If the conclusion is heavily supported, prefer undercutting over rebutting — direct rebuttal is asymmetric warfare when the conclusion has independent backing.
6. Produce a counter-argument as ONE of:
   - **Rebuttal** — `[Counter-premise]: ...` plus an `attack` relation to the target premise.
   - **Undercutter** — `<Undercutter>: ...` framing the inferential leap as unwarranted; explain why the warrant fails (modal, scope, missing condition).
7. Output the counter as Argdown the user can paste, plus a short prose explanation of which Pollock move it is and why.

## See also

- [references/argumentation-theory.md](../references/argumentation-theory.md) — Pollock
- [references/argdown-json-schema.md](../references/argdown-json-schema.md)

## What this skill does NOT do

- Does not edit the user's document.
- Does not classify "weak" without a counter.
