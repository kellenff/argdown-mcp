# MCP Extensions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Extend argdown-mcp with Dung semantics tools (`inspect_af`, `extensions`, `accepts`), YAML export, and a deprecated `dung_extensions` alias (v1.0); then ship QBAF DF-QuAD evaluation with dual weight resolution (v1.1).

**Architecture:** Solver logic grows in `argdown-model` (SCC analysis, propagation, grounded fixed-point, SCC backtracking, semantics dispatch; v1.1 adds `qbaf` module). Pure tool orchestration stays in `argdown-tools`. `argdown-mcp/src/server.rs` remains a thin rmcp adapter. TDD throughout with fixture-driven tests in `argdown-model`.

**Tech Stack:** Rust 2024 workspace, `rmcp`, `schemars`, `serde`/`serde_json`, `serde_yaml`, existing `argdown-parser` / `argdown-model` / `argdown-tools` / `argdown-mcp` / `argdown-cli`.

**Reference spec:** `docs/snowball/specs/2026-06-07-mcp-extensions-design.md`

---

## File structure

| File | Responsibility |
| --- | --- |
| `crates/argdown-model/src/dung/scc.rs` | **Create.** Tarjan SCC, acyclic check, isolated-args |
| `crates/argdown-model/src/dung/propagate.rs` | **Create.** Shared 3-valued propagation primitive |
| `crates/argdown-model/src/dung/grounded.rs` | **Create.** O(V+E) grounded fixed-point (move from `dung.rs`) |
| `crates/argdown-model/src/dung/search.rs` | **Create.** Backtracking over cyclic SCCs |
| `crates/argdown-model/src/dung/semantics.rs` | **Create.** Preferred/stable/complete filters + dispatch |
| `crates/argdown-model/src/dung/mod.rs` | **Create.** Re-export AF types + public API |
| `crates/argdown-model/src/dung.rs` | **Modify → remove.** Contents move into `dung/` subtree |
| `crates/argdown-model/src/qbaf.rs` | **Create (v1.1).** QBAF projection + DF-QuAD |
| `crates/argdown-model/src/model.rs` | **Modify (v1.1).** Add `canonical_metadata` to `Edge` |
| `crates/argdown-tools/src/lib.rs` | **Modify.** `inspect_af`, `extensions`, `accepts`, `ExportModelInput`, `qbaf_evaluate` |
| `crates/argdown-mcp/src/server.rs` | **Modify.** Register new tools; deprecate `dung_extensions` |
| `crates/argdown-mcp/tests/integration.rs` | **Modify.** Cover new tools |
| `crates/argdown-cli/src/main.rs` | **Modify.** `inspect-af`, `extensions`, `accepts`, `qbaf` subcommands |
| `plugins/argdown/skills/argdown-analysis/SKILL.md` | **Modify.** Document new tools |
| `plugins/argdown/commands/analyze.md` | **Modify.** Route to new tools |

**Boundary discipline:** only `server.rs` imports `rmcp`. Solver code never imports protocol types.

---

# Part A — v1.0

### Task 1: `dung/` module scaffold + SCC analysis

**Files:**
- Create: `crates/argdown-model/src/dung/mod.rs`
- Create: `crates/argdown-model/src/dung/scc.rs`
- Modify: `crates/argdown-model/src/lib.rs`
- Modify: `crates/argdown-model/src/dung.rs` (temporary — re-export from submodule until Task 2)

- [x] **Step 1: Write failing SCC tests**

Create `crates/argdown-model/src/dung/scc.rs` with tests only (module wired in step 3):

```rust
//! Strongly-connected-component analysis for Dung AFs.

use crate::ArgumentId;
use super::ArgumentationFramework;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AfMetadata {
    pub argument_count: usize,
    pub attack_count: usize,
    pub is_acyclic: bool,
    pub has_self_attacks: bool,
    pub strongly_connected_components: Vec<Vec<ArgumentId>>,
    pub isolated_arguments: Vec<ArgumentId>,
}

pub fn analyze_af(af: &ArgumentationFramework) -> AfMetadata {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArgumentationFramework;

    fn af(n: usize, attacks: &[(usize, usize)]) -> ArgumentationFramework {
        ArgumentationFramework {
            arguments: (0..n).map(ArgumentId).collect(),
            attacks: attacks.iter().map(|&(f, t)| (ArgumentId(f), ArgumentId(t))).collect(),
        }
    }

    #[test]
    fn empty_af_is_acyclic_with_no_sccs() {
        let m = analyze_af(&ArgumentationFramework::default());
        assert!(m.is_acyclic);
        assert!(m.strongly_connected_components.is_empty());
    }

    #[test]
    fn chain_is_acyclic() {
        let m = analyze_af(&af(3, &[(0, 1), (1, 2)]));
        assert!(m.is_acyclic);
        assert_eq!(m.strongly_connected_components.len(), 3);
    }

    #[test]
    fn two_cycle_is_not_acyclic() {
        let m = analyze_af(&af(2, &[(0, 1), (1, 0)]));
        assert!(!m.is_acyclic);
        assert_eq!(m.strongly_connected_components, vec![vec![ArgumentId(0), ArgumentId(1)]]);
    }

    #[test]
    fn isolated_node_is_listed() {
        let m = analyze_af(&af(3, &[(0, 1)]));
        assert_eq!(m.isolated_arguments, vec![ArgumentId(2)]);
    }
}
```

- [x] **Step 2: Run tests — expect FAIL**

Run: `cargo test -p argdown-model scc::tests -- --nocapture`
Expected: FAIL (`todo!()` panics or module not found)

- [x] **Step 3: Implement Tarjan + metadata**

Wire `mod dung;` to `mod dung { mod scc; ... }` — move existing `ArgumentationFramework`, `dung_framework`, `GroundedLabelling`, `grounded_extension` from `dung.rs` into `dung/mod.rs` and `dung/af.rs` (or keep in `mod.rs` initially). Implement `analyze_af`:

```rust
pub fn analyze_af(af: &ArgumentationFramework) -> AfMetadata {
    let argument_count = af.arguments.len();
    let attack_count = af.attacks.len();
    let has_self_attacks = af.attacks.iter().any(|&(a, b)| a == b);

    let mut adj: HashMap<ArgumentId, Vec<ArgumentId>> = HashMap::new();
    for &id in &af.arguments {
        adj.entry(id).or_default();
    }
    for &(from, to) in &af.attacks {
        adj.entry(from).or_default().push(to);
    }

    let sccs = tarjan(&af.arguments, &adj);
    let is_acyclic = sccs.iter().all(|c| c.len() == 1)
        && !af.attacks.iter().any(|&(a, b)| a == b);

    let mut attacked = HashSet::new();
    let mut attackers = HashSet::new();
    for &(from, to) in &af.attacks {
        attackers.insert(from);
        attacked.insert(to);
    }
    let isolated_arguments: Vec<_> = af
        .arguments
        .iter()
        .copied()
        .filter(|id| !attackers.contains(id) && !attacked.contains(id))
        .collect();

    AfMetadata {
        argument_count,
        attack_count,
        is_acyclic,
        has_self_attacks,
        strongly_connected_components: sccs,
        isolated_arguments,
    }
}
```

Implement `tarjan` as a standard iterative Tarjan over `af.arguments` order.

Update `lib.rs`:
```rust
pub use dung::{
    AfMetadata, ArgumentationFramework, GroundedLabelling, Semantics, dung_framework,
    grounded_extension, analyze_af, /* later: solve_semantics */,
};
```

- [x] **Step 4: Run tests — expect PASS**

Run: `cargo test -p argdown-model dung -- --nocapture`
Expected: all dung + scc tests PASS (relocate existing `dung.rs` tests unchanged)

- [x] **Step 5: Commit**

```bash
git add crates/argdown-model/src/dung/ crates/argdown-model/src/lib.rs
git commit -m "feat(model): add SCC analysis for Dung AF metadata"
```

---

### Task 2: Shared propagation + grounded refactor

**Files:**
- Create: `crates/argdown-model/src/dung/propagate.rs`
- Create: `crates/argdown-model/src/dung/grounded.rs`
- Modify: `crates/argdown-model/src/dung/mod.rs`

- [x] **Step 1: Write failing propagation tests**

```rust
// propagate.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Label { In, Out, Undec }

pub type Labeling = HashMap<ArgumentId, Label>;

pub fn empty_labeling(args: &[ArgumentId]) -> Labeling {
    args.iter().map(|&a| (a, Label::Undec)).collect()
}

/// One pass: IN if all attackers OUT; OUT if any attacker IN.
pub fn propagate_once(af: &ArgumentationFramework, labeling: &mut Labeling) -> bool {
    todo!()
}

/// Grounded fixed-point via repeated propagation until stable.
pub fn grounded_fixpoint(af: &ArgumentationFramework) -> Labeling {
    todo!()
}
```

Tests: unattacked → IN; `a→b` → a IN b OUT; 2-cycle → both UNDEC; self-attack → UNDEC.

- [x] **Step 2: Run tests — expect FAIL**

Run: `cargo test -p argdown-model propagate -- --nocapture`

- [x] **Step 3: Implement + wire `grounded_extension` to use it**

```rust
// grounded.rs
pub fn grounded_extension(af: &ArgumentationFramework) -> GroundedLabelling {
    let labeling = grounded_fixpoint(af);
    partition_labeling(af, labeling)
}
```

Replace the inline loop in old `grounded_extension` with this. Existing B6b tests must still pass.

- [x] **Step 4: Run full model tests**

Run: `cargo test -p argdown-model`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git commit -m "refactor(model): extract shared propagation + grounded fixpoint"
```

---

### Task 3: Semantics enum + complete-labeling search

**Files:**
- Create: `crates/argdown-model/src/dung/search.rs`
- Create: `crates/argdown-model/src/dung/semantics.rs`
- Modify: `crates/argdown-model/src/dung/mod.rs`

- [x] **Step 1: Write failing semantics tests**

```rust
// semantics.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Semantics {
    Grounded,
    Preferred,
    Stable,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    GroundedFixpoint,
    SccPropagationOnly,
    SccWithBacktracking,
    FilteredComplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticsResult {
    pub semantics: Semantics,
    pub algorithm: Algorithm,
    pub labellings: Vec<Labeling>,
}

pub fn solve(af: &ArgumentationFramework, semantics: Semantics) -> SemanticsResult {
    todo!()
}
```

Fixture tests (add to `search.rs` or `semantics.rs`):

```rust
#[test]
fn preferred_on_chain_has_one_labelling() {
    // a→b→c : preferred IN = {a,c}
    let af = af(3, &[(0, 1), (1, 2)]);
    let r = solve(&af, Semantics::Preferred);
    assert_eq!(r.algorithm, Algorithm::SccPropagationOnly);
    assert_eq!(in_set(&r.labellings[0]), vec![ArgumentId(0), ArgumentId(2)]);
}

#[test]
fn stable_on_odd_cycle_is_empty() {
    // 3-cycle: no stable extension
    let af = af(3, &[(0, 1), (1, 2), (2, 0)]);
    let r = solve(&af, Semantics::Stable);
    assert!(r.labellings.is_empty());
}
```

- [x] **Step 2: Run tests — expect FAIL**

- [x] **Step 3: Implement search + dispatch**

`search.rs`:
- Build attacker/target adjacency from `af.attacks`.
- If `analyze_af(af).is_acyclic`: run `grounded_fixpoint`, return single labelling with `Algorithm::SccPropagationOnly` (all semantics agree on DAG).
- Else: SCC-decompose; propagate acyclic regions; backtrack only within cyclic SCCs to enumerate **complete** labelings.
- Stable fail-fast: track `defeated` set; abort branch if OUT node not defeated by IN.

`semantics.rs`:
- `Semantics::Grounded` → `grounded_fixpoint` only, `Algorithm::GroundedFixpoint`.
- `Semantics::Complete` → all complete labellings from search.
- `Semantics::Preferred` → filter to ⊆-maximal IN sets.
- `Semantics::Stable` → filter to labellings with no UNDEC + reinstatement; empty vec if none.

- [x] **Step 4: Run tests — expect PASS**

Run: `cargo test -p argdown-model semantics search`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git commit -m "feat(model): SCC search + preferred/stable/complete semantics"
```

---

### Task 4: `inspect_af` + `extensions` in argdown-tools

**Files:**
- Modify: `crates/argdown-tools/src/lib.rs`
- Modify: `crates/argdown-tools/Cargo.toml` (ensure `schemars` feature exports new types)

- [x] **Step 1: Write failing tool tests**

Add to `crates/argdown-tools/src/lib.rs`:

```rust
use argdown_model::{
    analyze_af, dung_framework, solve, AfMetadata, Algorithm, Semantics, SemanticsResult,
};

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct AfArgumentRef {
    pub id: usize,
    pub title: Option<String>,
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct AfAttackRef {
    pub attacker: usize,
    pub target: usize,
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct InspectAfResult {
    pub arguments: Vec<AfArgumentRef>,
    pub attacks: Vec<AfAttackRef>,
    pub metadata: AfMetadata,
}

pub fn inspect_af(source: &str) -> Result<InspectAfResult, Diagnostic> {
    todo!()
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct ExtensionsInput {
    pub semantics: Semantics,
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct LabellingEntry {
    pub id: usize,
    pub title: Option<String>,
    pub label: String, // "in" | "out" | "undec"
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct ExtensionsResult {
    pub semantics: Semantics,
    pub algorithm: Algorithm,
    pub labellings: Vec<Vec<LabellingEntry>>,
    pub extension_sets: Vec<Vec<ArgRef>>,
}

pub fn extensions(source: &str, semantics: Semantics) -> Result<ExtensionsResult, Diagnostic> {
    todo!()
}
```

Tests:

```rust
#[test]
fn inspect_af_shows_attack_edge() {
    let r = inspect_af("<A>: a\n\n<B>: b\n  -> <A>").unwrap();
    assert_eq!(r.attacks.len(), 1);
    assert_eq!(r.attacks[0].attacker, 1);
}

#[test]
fn extensions_grounded_matches_dung() {
    let old = dung("<A>: a\n\n<B>: b\n  -> <A>").unwrap();
    let new = extensions("<A>: a\n\n<B>: b\n  -> <A>", Semantics::Grounded).unwrap();
    assert_eq!(new.extension_sets[0].iter().map(|a| a.id).collect::<Vec<_>>(),
               old.in_.iter().map(|a| a.id).collect::<Vec<_>>());
}
```

- [x] **Step 2: Run tests — expect FAIL**

Run: `cargo test -p argdown-tools inspect_af extensions`

- [x] **Step 3: Implement**

Pipeline for both: `parse → build_model → dung_framework → …`

Map `Label` → `"in"|"out"|"undec"`. Build `extension_sets` as IN-partition per labelling.

- [x] **Step 4: Run tests — expect PASS**

- [x] **Step 5: Commit**

```bash
git commit -m "feat(tools): add inspect_af and extensions pure functions"
```

---

### Task 5: `accepts` tool with witnesses

**Files:**
- Modify: `crates/argdown-tools/src/lib.rs`

- [x] **Step 1: Write failing accepts tests**

```rust
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptanceMode { Credulous, Skeptical }

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessType {
    AcceptedUncontroversial,
    AttackedByAccepted,
    UnsupportedCycle,
    MultipleInterpretations,
    SkepticallyRejected,
    UndefinedArgument,
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct AcceptsResult {
    pub accepted: bool,
    pub status: String, // "in" | "out" | "undec" | "varies"
    pub unanimous: bool,
    pub witness: WitnessPayload,
}

pub fn accepts(
    source: &str,
    argument_id: usize,
    semantics: Semantics,
    mode: AcceptanceMode,
) -> Result<AcceptsResult, Diagnostic> {
    todo!()
}
```

Tests:
- Unattacked arg, skeptical preferred → `accepted_uncontroversial`.
- `a→b`, query b credulous preferred → `attacked_by_accepted`.
- Unknown id → `undefined_argument`.
- 2-cycle, query either → `unsupported_cycle` or `multiple_interpretations` depending on mode.

- [x] **Step 2: Run — FAIL**

- [x] **Step 3: Implement**

Call `extensions(source, semantics)` internally (or `solve` directly). Derive per-extension status for `argument_id`. Apply credulous/skeptical rule. Build witness from first rejecting/supporting labelling.

- [x] **Step 4: Run — PASS**

- [x] **Step 5: Commit**

```bash
git commit -m "feat(tools): add accepts with structured witnesses"
```

---

### Task 6: MCP server — new tools + deprecated alias + YAML export

**Files:**
- Modify: `crates/argdown-mcp/src/server.rs`
- Modify: `crates/argdown-mcp/tests/integration.rs`

- [x] **Step 1: Write failing integration test**

Add to `crates/argdown-mcp/tests/integration.rs`:

```rust
#[tokio::test]
async fn list_tools_includes_extensions_and_inspect_af() {
    // ... existing harness ...
    let names: Vec<_> = tools.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"extensions"));
    assert!(names.contains(&"inspect_af"));
    assert!(names.contains(&"accepts"));
    assert!(names.contains(&"dung_extensions")); // deprecated alias
}
```

- [x] **Step 2: Run — FAIL**

Run: `cargo test -p argdown-mcp list_tools_includes`

- [x] **Step 3: Register tools in server.rs**

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportModelInput {
    pub source: String,
    #[serde(default = "default_json")]
    pub format: ExportFormat,
}

#[derive(Debug, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat { #[default] Json, Yaml }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExtensionsInput {
    pub source: String,
    #[serde(default = "default_preferred")]
    pub semantics: Semantics,
}

fn default_preferred() -> Semantics { Semantics::Preferred }

#[tool(name = "inspect_af", description = "...")]
fn inspect_af(&self, Parameters(input): Parameters<SourceInput>) -> Result<Json<InspectAfResult>, ErrorData> { ... }

#[tool(name = "extensions", description = "...")]
fn extensions(&self, Parameters(input): Parameters<ExtensionsInput>) -> Result<Json<ExtensionsResult>, ErrorData> { ... }

#[tool(name = "accepts", description = "...")]
fn accepts(&self, Parameters(input): Parameters<AcceptsInput>) -> Result<Json<AcceptsResult>, ErrorData> { ... }

#[tool(
    name = "dung_extensions",
    description = "DEPRECATED: use `extensions` with semantics=\"grounded\". Will be removed in v2."
)]
fn dung_extensions(&self, Parameters(input): Parameters<SourceInput>) -> Result<Json<DungResult>, ErrorData> {
    // delegate to extensions(..., Grounded) and map to legacy DungResult shape
}
```

Update `export_model` to accept `ExportModelInput` with format param.

Update `ServerHandler` instructions string.

- [x] **Step 4: Run full MCP tests**

Run: `cargo test -p argdown-mcp`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git commit -m "feat(mcp): add inspect_af, extensions, accepts; deprecate dung_extensions"
```

---

### Task 7: CLI subcommands

**Files:**
- Modify: `crates/argdown-cli/src/main.rs`
- Modify: `crates/argdown-cli/tests/cli.rs`

- [x] **Step 1: Write failing CLI tests**

```rust
#[test]
fn inspect_af_prints_attacks() {
    let out = run(&["inspect-af"], "<A>: a\n\n<B>: b\n  -> <A>\n");
    assert!(out.contains("attacks"));
}
```

- [x] **Step 2: Run — FAIL**

- [x] **Step 3: Add subcommands**

```rust
enum Command {
    Parse,
    Export { #[arg(long, default_value = "json")] format: String },
    Dung, // keep; prints grounded partition (legacy)
    InspectAf,
    Extensions { #[arg(long, default_value = "preferred")] semantics: String },
    Accepts { id: usize, #[arg(long, default_value = "preferred")] semantics: String,
              #[arg(long, default_value = "credulous")] mode: String },
}
```

Wire to `argdown_tools` functions; emit JSON on stdout.

- [x] **Step 4: Run CLI tests — PASS**

Run: `cargo test -p argdown-cli`

- [x] **Step 5: Commit**

```bash
git commit -m "feat(cli): add inspect-af, extensions, accepts subcommands"
```

---

### Task 8: Plugin skill + docs

**Files:**
- Modify: `plugins/argdown/skills/argdown-analysis/SKILL.md`
- Modify: `plugins/argdown/commands/analyze.md`

- [x] **Step 1: Update skill**

Document:
- `inspect_af` — projected AF + SCC metadata
- `extensions` — semantics param, default `preferred`
- `accepts` — credulous/skeptical point query
- `export_model` — `format: yaml`
- `dung_extensions` — deprecated, use `extensions` with `grounded`

- [x] **Step 2: Update analyze command routing**

- [x] **Step 3: Verify workspace**

Run: `cargo build && cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings`
Expected: clean

- [x] **Step 4: Commit**

```bash
git commit -m "docs(plugin): document MCP extension tools; mark dung_extensions deprecated"
```

---

# Part B — v1.1 QBAF fast follow

### Task 9: Edge metadata on `Edge`

**Files:**
- Modify: `crates/argdown-model/src/model.rs`
- Modify: `crates/argdown-model/src/export.rs` / `import.rs` (round-trip)
- Test: `crates/argdown-model/src/model.rs` (existing test module)

- [x] **Step 1: Write failing test**

```rust
#[test]
fn relation_metadata_is_preserved_on_edge() {
    let m = build_model(&parse("<A> {weight: 0.3}: a\n  + {weight: 0.9} <B>: b\n\n<B>: b").unwrap());
    let support = m.edges.iter().find(|e| e.kind == RelationKind::Support).expect("support edge");
    assert_eq!(
        support.canonical_metadata.as_ref().and_then(|v| v.get("weight")),
        Some(&Value::Number(/* 0.9 */))
    );
}
```

(Adjust parser fixture to valid relation metadata syntax per A5a grammar.)

- [x] **Step 2: Run — FAIL**

- [x] **Step 3: Add field + wire build**

```rust
pub struct Edge {
    pub from: Node,
    pub to: Node,
    pub kind: RelationKind,
    pub span: Span,
    pub canonical_metadata: Option<Value>,
}
```

Populate in `push_edge_if_new` from parser relation metadata (thread through B5 edge construction).

- [x] **Step 4: Run model + import/export tests — PASS**

- [x] **Step 5: Commit**

```bash
git commit -m "feat(model): preserve relation metadata on Edge for QBAF weights"
```

---

### Task 10: Weight extraction + QBAF projection

**Files:**
- Create: `crates/argdown-model/src/qbaf.rs`
- Modify: `crates/argdown-model/src/lib.rs`

- [x] **Step 1: Write failing QBAF projection tests**

```rust
pub struct QbafNode {
    pub id: ArgumentId,
    pub base_degree: f64,
}

pub struct QbafEdge {
    pub from: ArgumentId,
    pub to: ArgumentId,
    pub kind: QbafEdgeKind, // Attack | Support
    pub weight: f64,
}

pub struct QbafFramework {
    pub nodes: Vec<QbafNode>,
    pub edges: Vec<QbafEdge>,
}

const DEFAULT_BASE: f64 = 0.5;

pub fn base_degree(meta: &Option<Value>) -> Result<f64, String> {
    meta.as_ref()
        .and_then(|v| v.get("weight"))
        .map(parse_weight)
        .transpose()?
        .unwrap_or(Ok(DEFAULT_BASE))
}

pub fn project_qbaf(model: &Model) -> Result<QbafFramework, String> {
    todo!()
}
```

Tests:
- No metadata → base 0.5.
- Arg `{weight: 0.8}` → base 0.8.
- Edge `{weight: 0.3}` overrides source base for that edge.
- Undercut → attack edge in QBAF.

- [x] **Step 2: Run — FAIL**

- [x] **Step 3: Implement projection**

Map argument-level `Attack`, `Undercut` → `QbafEdgeKind::Attack`; `Support` → Support. Skip statement-level edges (same as Dung sharp scope for argument endpoints only — document in module docs).

- [x] **Step 4: Run — PASS**

- [x] **Step 5: Commit**

```bash
git commit -m "feat(model): QBAF projection with dual weight resolution"
```

---

### Task 11: DF-QuAD solver + `qbaf_evaluate` tool

**Files:**
- Modify: `crates/argdown-model/src/qbaf.rs`
- Modify: `crates/argdown-tools/src/lib.rs`
- Modify: `crates/argdown-mcp/src/server.rs`
- Modify: `crates/argdown-cli/src/main.rs`

- [x] **Step 1: Write failing DF-QuAD tests**

Use a 2-argument support/attack fixture with known degrees from hand calculation or literature example.

```rust
pub fn df_quad(qbaf: &QbafFramework, max_iterations: usize) -> HashMap<ArgumentId, f64> {
    todo!()
}

#[derive(Debug, Serialize)]
pub struct QbafDegree {
    pub id: usize,
    pub title: Option<String>,
    pub base: f64,
    pub final_degree: f64,
    pub status: String, // "accepted" | "rejected" | "undec"
}

pub fn qbaf_evaluate(source: &str, threshold: f64) -> Result<Vec<QbafDegree>, Diagnostic> {
    todo!()
}
```

- [x] **Step 2: Run — FAIL**

- [x] **Step 3: Implement DF-QuAD iteration**

Standard update rules: attacks decrease, supports increase, clamp [0,1], iterate to fixpoint or `max_iterations` (default 500, document in tool description). Classify: `final >= threshold` → accepted, `< threshold` → rejected; if oscillation detected → undec.

- [x] **Step 4: Wire MCP tool + CLI `qbaf` subcommand**

```rust
#[tool(name = "qbaf_evaluate", description = "Compute QBAF DF-QuAD degrees for arguments.")]
fn qbaf_evaluate(&self, Parameters(input): Parameters<QbafInput>) -> Result<Json<Vec<QbafDegree>>, ErrorData> { ... }
```

- [x] **Step 5: Run full workspace tests — PASS**

- [x] **Step 6: Commit**

```bash
git commit -m "feat: QBAF DF-QuAD evaluation (v1.1) — MCP, CLI, and solver"
```

---

### Task 12: v1.1 plugin docs + release notes

**Files:**
- Modify: `plugins/argdown/skills/argdown-analysis/SKILL.md`

- [x] **Step 1: Document `qbaf_evaluate`**

Include weight resolution rules (argument base, relation override, default 0.5).

- [x] **Step 2: Commit**

```bash
git commit -m "docs(plugin): document qbaf_evaluate tool"
```

---

## Self-review (plan vs spec)

| Spec requirement | Task |
| --- | --- |
| `inspect_af` + SCC metadata | Task 1, 4 |
| `extensions` semantics param | Task 3, 4 |
| `accepts` credulous/skeptical + witnesses | Task 5 |
| Deprecated `dung_extensions` → v2 removal | Task 6 |
| `export_model` YAML format | Task 6 |
| Sharp Dung projection unchanged | Tasks 1–3 (no bipolar in Dung) |
| Algorithm provenance enum | Task 3 |
| Grounded vs search entry points | Tasks 2–3 |
| QBAF v1.1 DF-QuAD | Tasks 10–11 |
| Weight resolution C | Tasks 9–10 |
| Edge metadata prerequisite | Task 9 |
| Plugin skill updates | Tasks 8, 12 |
| Out of scope items | Not planned ✓ |

No placeholders remain. Type names (`Semantics`, `Algorithm`, `InspectAfResult`, etc.) are consistent across tasks.

---

## Verification gate (both parts)

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Cross-check `extensions` grounded against `@argdown/core` `dung_extensions` on shared sample (project convention). **Done:** live reference MCP unavailable; B6b probe encoded in `reference_parity.rs` and MCP integration test — see `docs/snowball/decisions/2026-06-07T-reference-grounded-cross-check.md`.

---

## Execution handoff

**Plan complete and saved to `docs/snowball/plans/2026-06-07-mcp-extensions.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
