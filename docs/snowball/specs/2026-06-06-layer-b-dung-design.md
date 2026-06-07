# Layer B — Dung AF + Grounded Extension (B6b) — Design

- **Date:** 2026-06-06
- **Status:** Draft (for review)
- **Scope:** The second half of Layer B slice B6 and the project's end goal:
  project the `Model` into a **Dung abstract argumentation framework (AF)** and
  compute its **grounded extension** ("Dung extensions in Rust"). Two pure,
  **total** functions in a new `argdown_model::dung` module. Representation-only;
  `argdown-mcp` stays a placeholder.

## Context

B6 was split: **B6a** is the tag registry (shipped); **B6b** is the AF + grounded
extension. The representation was settled by a **chorus brain-jam** over
`@argdown/core` probes, then the one contested point was resolved by a decisive
reference probe (below).

### Reference semantics (probed via `dung_extensions`)

`@argdown/core`'s Dung framework is a **sharp projection**:
- **Nodes = arguments only.**
- **Attacks = argument→argument relations of kind `Attack` only.** Supports,
  undercuts, contradictories, and **all statement-level attacks are ignored**.
- The **grounded extension** is the unique grounded labelling: `in` (survive all
  attackers), `out` (defeated by an `in` attacker), `undec` (cycles / self-attack).

Probes that pin this:
- `<a>: A` / `  - <b>` / `<b>: B` → **1 attack**, grounded `{b in, a out}`. So an
  argument-level `-` is **b attacks a** (the target attacks the parent), and our
  B5 already emits this as an `Argument→Argument` `Attack` edge.
- The *same* attack written at the statement level (a `- <b>` nested under a
  PCS premise of `<a>`) → **0 attacks**, both `in`. Our B5 emits that with a
  `Node::Statement` endpoint, which the projection drops — matching the
  reference exactly. (This is why no statement→argument "lifting" / `StatementMapper`
  is done: it would over-count and diverge from the reference. If lifting is ever
  wanted, it is a separate slice that first probes the reference for the rule.)
- A 3-argument attack chain → `{A, C} in, {B} out`.

## Decisions (chorus-settled, probe-confirmed)

1. **Standalone projection, not new `Model` fields.** The AF is a *derived view*
   of `Model.edges` for one consumer (the solver), not document data. So:
   `fn dung_framework(&Model) -> ArgumentationFramework` and
   `fn grounded_extension(&ArgumentationFramework) -> GroundedLabelling`. The
   `Model` stays the parsed-document model; the AF/extension is a computed query.
   (Compose, don't mutate.)
2. **Sharp projection.** `dung_framework` keeps exactly the edges with
   `from = Node::Argument`, `to = Node::Argument`, `kind == Attack`; deduped, in
   first-occurrence (source) order. Everything else is excluded — matching the
   reference.
3. **All arguments are AF nodes**, named and anonymous (a standalone PCS is an
   anonymous argument), in arena/source order. Isolated nodes (no attacks) are
   `in`.
4. **Grounded extension via the characteristic-function least-fixpoint.** Total
   (the grounded labelling is unique and always exists — no `Result`).
   Deterministic: outputs follow `af.arguments` order.

## Data types

```rust
use crate::model::{ArgumentId, Model, Node, RelationKind};

/// A Dung abstract argumentation framework: arguments + a binary attack
/// relation. A sharp projection of the `Model` (argument→argument `Attack`
/// edges only); see module docs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArgumentationFramework {
    /// All Model arguments (named + anonymous), in arena/source order.
    pub arguments: Vec<ArgumentId>,
    /// Deduped directed attacks `(attacker, attacked)`, first-occurrence order.
    pub attacks: Vec<(ArgumentId, ArgumentId)>,
}

/// The unique grounded labelling. Each argument appears in exactly one list;
/// lists follow `ArgumentationFramework::arguments` order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroundedLabelling {
    /// Accepted: every attacker is `out` (vacuously true if unattacked).
    pub in_: Vec<ArgumentId>,
    /// Defeated: attacked by some `in` argument.
    pub out: Vec<ArgumentId>,
    /// Undecided: in cycles / self-attacks the fixpoint cannot resolve.
    pub undec: Vec<ArgumentId>,
}

pub fn dung_framework(model: &Model) -> ArgumentationFramework
pub fn grounded_extension(af: &ArgumentationFramework) -> GroundedLabelling
```

## Algorithm

**`dung_framework`:** `arguments = model.arguments.iter().map(|a| a.id)`. For each
`edge` in `model.edges` with `kind == Attack` and both endpoints
`Node::Argument`, push `(from, to)` if not already present (dedup on the pair).

**`grounded_extension`:** build `attackers_of: HashMap<ArgumentId, Vec<ArgumentId>>`
from `af.attacks`. Label all `undec`. Iterate to a fixpoint: an `undec` argument
becomes `in` if **all** its attackers are `out` (vacuously, if none); `out` if
**any** attacker is `in`. Repeat until a pass makes no change; remaining `undec`
stay `undec`. Partition `af.arguments` (preserving order) into the three lists.
(Indices resolve via an `ArgumentId → position` map so the function is correct
for any AF, not only the full dense arena.)

Worked: unattacked → `in`; `a→b→c` → `{a,c} in, {b} out`; self-attack `a→a` →
`undec`; 2-cycle `a↔b` → both `undec`.

## Error handling

None — both functions total. No `Result`.

## Testing (TDD)

Gated by `cargo test` / `clippy -D warnings` / `fmt`.

**`dung_framework`** (via `argdown_parser::parse` + `build_model`):
1. Empty model → empty AF.
2. Argument-level attack (`<a>: A` / `  - <b>` / `<b>: B`) → `arguments` has both,
   `attacks == [(b, a)]` (matches the reference direction).
3. Statement-level attack (`- <b>` under a PCS premise of `<a>`) → **0 attacks**.
4. Support / undercut excluded (`<a>: A` / `  + <b>` ...) → 0 attacks.
5. Dedup: the same attack written twice → one attack.
6. Anonymous argument (a standalone PCS) appears in `arguments`.

**`grounded_extension`** (directly-constructed `ArgumentationFramework`s — the
function is independent of parsing):
7. Single unattacked argument → `in`.
8. `a` attacks `b` → `a in, b out`.
9. Chain `(a,b),(b,c)` → `{a,c} in, {b} out`.
10. Self-attack `(a,a)` → `undec`.
11. Two-cycle `(a,b),(b,a)` → both `undec`.

**End-to-end:** `<a>: A` / `  - <b>` / `<b>: B` → `grounded_extension(dung_framework(&m))`
== `{ in: [b], out: [a], undec: [] }`, matching the `dung_extensions` probe.

## Out of scope

- **Statement→argument lifting** (counting statement-level attacks as argument
  attacks) — the reference ignores them; a future slice would probe + pin a rule.
- **Undercut→inference auxiliary nodes**, preferred/stable/complete extensions,
  edge ids, per-node adjacency on the public type.
- **Wiring into the `argdown-mcp` server** — the protocol layer remains deferred.

## Summary

B6b is the project's end goal: `dung_framework(&Model)` sharply projects the
argument-level attack graph, and `grounded_extension(&AF)` computes the unique
grounded labelling in Rust — both total, standalone, reference-faithful (probe-
confirmed). It completes Layer B (B1–B6).
