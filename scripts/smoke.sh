#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_FILE="$WORKTREE_ROOT/dist/server.js"

echo "=== smoke: composite end-to-end test ==="

# ── build if needed ─────────────────────────────────────────────────────────────
if [[ ! -f "$DIST_FILE" ]]; then
  echo "smoke: dist/server.js not found — running yarn build..."
  (cd "$WORKTREE_ROOT" && yarn build)
fi

# ── Phase A: MCP Inspector ───────────────────────────────────────────────────────
echo ""
echo "--- Phase A: MCP Inspector (tools/list) ---"

TMPDIR_INSPECTOR=""
cleanup_inspector() {
  [[ -n "$TMPDIR_INSPECTOR" ]] && rm -rf "$TMPDIR_INSPECTOR"
}
trap cleanup_inspector EXIT

TMPDIR_INSPECTOR=$(mktemp -d)

echo "Phase A: installing @modelcontextprotocol/inspector into $TMPDIR_INSPECTOR..."
(
  cd "$TMPDIR_INSPECTOR"
  npm init -y > /dev/null 2>&1
  npm install @modelcontextprotocol/inspector > /dev/null 2>&1
)

INSPECTOR_INDEX="$TMPDIR_INSPECTOR/node_modules/@modelcontextprotocol/inspector-cli/build/index.js"
if [[ ! -f "$INSPECTOR_INDEX" ]]; then
  echo "FAIL: Phase A — inspector-cli index.js not found after npm install" >&2
  exit 1
fi

# Run from within inspector-cli/build so relative package.json import resolves correctly
echo "Phase A: spawning inspector (tools/list)..."
PHASE_A_OUTPUT=$(
  cd "$(dirname "$INSPECTOR_INDEX")" && \
  perl -e 'alarm 60; exec @ARGV' \
    node index.js \
    node \
    --method tools/list \
    -- \
    "$DIST_FILE" \
    2>&1
)
PHASE_A_EXIT=$?

if [[ $PHASE_A_EXIT -ne 0 ]]; then
  echo "FAIL: Phase A — inspector exited with code $PHASE_A_EXIT" >&2
  echo "Inspector output:" >&2
  echo "$PHASE_A_OUTPUT" >&2
  exit 1
fi

if ! echo "$PHASE_A_OUTPUT" | grep -q '"parse"'; then
  echo "FAIL: Phase A — tools/list response does not contain tool name 'parse'" >&2
  echo "Inspector output: $PHASE_A_OUTPUT" >&2
  exit 1
fi

if ! echo "$PHASE_A_OUTPUT" | grep -q '"export_json"'; then
  echo "FAIL: Phase A — tools/list response does not contain tool name 'export_json'" >&2
  echo "Inspector output: $PHASE_A_OUTPUT" >&2
  exit 1
fi

echo "Phase A: tools/list OK (both 'parse' and 'export_json' present)"

# Also invoke tools/call parse
echo "Phase A: spawning inspector (tools/call parse)..."
PHASE_A_CALL_OUTPUT=$(
  cd "$(dirname "$INSPECTOR_INDEX")" && \
  perl -e 'alarm 60; exec @ARGV' \
    node index.js \
    node \
    --method tools/call \
    --tool-name parse \
    --tool-arg kind=inline \
    --tool-arg "source=[a]: hello" \
    -- \
    "$DIST_FILE" \
    2>&1
)
PHASE_A_CALL_EXIT=$?

if [[ $PHASE_A_CALL_EXIT -ne 0 ]]; then
  echo "FAIL: Phase A — tools/call parse exited with code $PHASE_A_CALL_EXIT" >&2
  echo "Inspector output:" >&2
  echo "$PHASE_A_CALL_OUTPUT" >&2
  exit 1
fi

if ! echo "$PHASE_A_CALL_OUTPUT" | grep -q "Parsed"; then
  echo "FAIL: Phase A — tools/call parse response does not contain 'Parsed'" >&2
  echo "Inspector output: $PHASE_A_CALL_OUTPUT" >&2
  exit 1
fi

echo "Phase A: tools/call parse OK (response contains 'Parsed')"
echo "OK: Phase A passed"

# ── Phase B: Tarball install + run ───────────────────────────────────────────────
echo ""
echo "--- Phase B: Tarball install + run ---"

TMPDIR_RUN_PHASE_B=""
cleanup_phase_b() {
  [[ -n "$TMPDIR_RUN_PHASE_B" ]] && rm -rf "$TMPDIR_RUN_PHASE_B"
  cleanup_inspector
}
trap cleanup_phase_b EXIT

TMPDIR_RUN_PHASE_B=$(mktemp -d)

echo "Phase B: packing tarball..."
(cd "$WORKTREE_ROOT" && yarn pack -o "$TMPDIR_RUN_PHASE_B/argdown-mcp.tgz" 2>&1) | grep -v '^➤' || true

if [[ ! -f "$TMPDIR_RUN_PHASE_B/argdown-mcp.tgz" ]]; then
  echo "FAIL: Phase B — yarn pack did not produce argdown-mcp.tgz" >&2
  exit 1
fi

echo "Phase B: installing tarball via npm (simulating real consumer)..."
(
  cd "$TMPDIR_RUN_PHASE_B"
  npm install ./*.tgz > /dev/null 2>&1
)

ARGDOWN_BIN="$TMPDIR_RUN_PHASE_B/node_modules/.bin/argdown-mcp"
if [[ ! -f "$ARGDOWN_BIN" ]]; then
  echo "FAIL: Phase B — argdown-mcp binary not found in node_modules/.bin after npm install" >&2
  exit 1
fi

echo "Phase B: invoking tarball server with initialize + tools/list..."
PHASE_B_INPUT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"1.0"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

PHASE_B_OUTPUT=$(
  echo "$PHASE_B_INPUT" | \
  perl -e 'alarm 10; exec @ARGV' "$ARGDOWN_BIN" 2>/dev/null
)
PHASE_B_EXIT=$?

if [[ $PHASE_B_EXIT -ne 0 ]]; then
  echo "FAIL: Phase B — tarball server exited with code $PHASE_B_EXIT (possible timeout)" >&2
  exit 1
fi

if ! echo "$PHASE_B_OUTPUT" | grep -q '"parse"'; then
  echo "FAIL: Phase B — tools/list response does not contain tool name 'parse'" >&2
  echo "Server output: $PHASE_B_OUTPUT" >&2
  exit 1
fi

if ! echo "$PHASE_B_OUTPUT" | grep -q '"export_json"'; then
  echo "FAIL: Phase B — tools/list response does not contain tool name 'export_json'" >&2
  echo "Server output: $PHASE_B_OUTPUT" >&2
  exit 1
fi

echo "OK: Phase B passed"

# ── Phase C: SIGPIPE / stdin-close shutdown ──────────────────────────────────────
echo ""
echo "--- Phase C: SIGPIPE / stdin-close shutdown ---"

echo "Phase C: spawning server, sending initialize, then closing stdin..."

# Use gdate for nanosecond precision on macOS if available; fall back to date +%s
if command -v gdate >/dev/null 2>&1; then
  START_NS=$(gdate +%s%N)
else
  START_NS=$(date +%s%N 2>/dev/null || perl -MTime::HiRes=time -e 'printf "%.0f\n", time()*1000000000')
fi

# Send initialize then EOF; measure how long the server takes to exit.
PHASE_C_OUTPUT=$(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"1.0"}}}' | \
  perl -e 'alarm 5; exec @ARGV' node "$DIST_FILE" 2>/dev/null
)
PHASE_C_EXIT=$?

if command -v gdate >/dev/null 2>&1; then
  END_NS=$(gdate +%s%N)
else
  END_NS=$(date +%s%N 2>/dev/null || perl -MTime::HiRes=time -e 'printf "%.0f\n", time()*1000000000')
fi

ELAPSED_MS=$(( (END_NS - START_NS) / 1000000 ))

# perl alarm exits with 142 (SIGALRM); if we hit the alarm the server didn't exit cleanly.
if [[ $PHASE_C_EXIT -eq 142 ]]; then
  echo "FAIL: Phase C — server did not exit within 5s after stdin closed (SIGALRM fired)" >&2
  exit 1
fi

# Also reject non-zero exits (server should shut down cleanly with 0).
if [[ $PHASE_C_EXIT -ne 0 ]]; then
  echo "FAIL: Phase C — server exited with non-zero status $PHASE_C_EXIT after stdin close" >&2
  exit 1
fi

# Sanity: should contain the initialize response (proves the server actually started)
if ! echo "$PHASE_C_OUTPUT" | grep -q '"protocolVersion"'; then
  echo "FAIL: Phase C — no initialize response received from server" >&2
  echo "Server output: $PHASE_C_OUTPUT" >&2
  exit 1
fi

# Enforce < 2000ms shutdown time
if [[ $ELAPSED_MS -ge 2000 ]]; then
  echo "FAIL: Phase C — server took ${ELAPSED_MS}ms to exit after stdin close (threshold: 2000ms)" >&2
  exit 1
fi

echo "Phase C: server exited cleanly with status $PHASE_C_EXIT in ${ELAPSED_MS}ms"
echo "OK: Phase C passed"

# ── All phases green ─────────────────────────────────────────────────────────────
echo ""
echo "OK: all smoke phases passed"
