---
name: rebut-argument
description: This skill should be used when the user asks to "rebut this argument", "steelman a counter-argument to", "how would someone disagree with", or wants to generate a principled counter-position to a named Argdown argument. Calls the export_json MCP tool and applies Pollock's rebutting/undercutting frame plus Walton-scheme classification. SKIP: when the user asks to find weak premises broadly — that's find-unsupported-premises. SKIP: when the user asks to validate syntax — that's validate-argdown.
---

# rebut-argument

Generate a principled counter to a named argument. Apply Pollock + Walton.

## Required MCP tools

- `mcp__argdown-mcp__export_json`

## Workflow

1. Identify the target argument by title (e.g., `<Argument>`).
2. Call `export_json`; extract the fenced JSON.
3. Locate the argument in `arguments[<title>]`. Read its `pcs[]`.
4. Classify the argument's scheme (see "Scheme-aware rebuttals" below) — this picks the critical questions to press.
5. Apply Pollock (see [references/argumentation-theory.md](../references/argumentation-theory.md)):
   - **Rebutting defeater** — attack a premise directly. Best when one premise is structurally weakest or factually contestable.
   - **Undercutting defeater** — attack the inferential warrant. Best when premises are strong but the inference is loose.
6. Inspect `relations[]` for inbound `support` to the conclusion. If support is heavy, prefer undercutting.
7. Output the counter as ONE of:
   - **Rebuttal** — `[Counter-premise]: ...` plus an `attack` relation to the target premise.
   - **Undercutter** — `<Undercutter>: ...` framing the inferential leap as unwarranted.
8. Produce Argdown the user can paste, plus a short note on which Pollock move + Walton scheme + which critical question it answers.

## Scheme-aware rebuttals (Walton)

Classify the target, then press the matching critical question:

- **Expert opinion** — "Is the expert qualified, unbiased, and asserting this explicitly?"
- **Popular opinion** — "Is wide acceptance evidence, or mere convergence?"
- **Cause-to-effect** — "Are there confounders or counter-instances?"
- **Analogy** — "Where does the analogy break down?"
- **Sign** — "Could the sign have a different cause?"
- **Consequences** — "Are the predicted consequences likely, and do they actually favour the conclusion?"
- **Example** — "Is the example representative or cherry-picked?"
- **Verbal classification** — "Does the case actually fit the definition?"

## See also

- [references/argumentation-theory.md](../references/argumentation-theory.md) — Pollock + Walton
- [references/argdown-json-schema.md](../references/argdown-json-schema.md)
