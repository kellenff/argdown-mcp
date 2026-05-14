#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_FILE="$WORKTREE_ROOT/dist/server.js"

echo "=== bundle-sanity: checking dist/server.js ==="

# 1. dist/server.js exists
if [[ ! -f "$DIST_FILE" ]]; then
  echo "FAIL: dist/server.js does not exist" >&2
  exit 1
fi
echo "OK: dist/server.js exists"

# 2. First line is #!/usr/bin/env node
FIRST_LINE="$(head -1 "$DIST_FILE")"
if [[ "$FIRST_LINE" != "#!/usr/bin/env node" ]]; then
  echo "FAIL: First line of dist/server.js is not '#!/usr/bin/env node' (got: $FIRST_LINE)" >&2
  exit 1
fi
echo "OK: shebang is correct"

# 3. Bundle size < 100 MB (generous ceiling; current ~6 MB after bundling
#    everything including @argdown/node + cosmiconfig + chevrotain)
BUNDLE_SIZE=$(wc -c < "$DIST_FILE")
MAX_SIZE=$((100 * 1024 * 1024))
if [[ "$BUNDLE_SIZE" -ge "$MAX_SIZE" ]]; then
  echo "FAIL: dist/server.js is ${BUNDLE_SIZE} bytes (>= 100 MB limit)" >&2
  exit 1
fi
echo "OK: bundle size is ${BUNDLE_SIZE} bytes (< 100 MB)"

# 4. No .node files anywhere in dist/
NODE_FILES=$(find "$WORKTREE_ROOT/dist" -name "*.node" 2>/dev/null || true)
if [[ -n "$NODE_FILES" ]]; then
  echo "FAIL: Found .node files in dist/:" >&2
  echo "$NODE_FILES" >&2
  exit 1
fi
echo "OK: no .node files in dist/"

# 5. No forbidden substrings in dist/server.js
FORBIDDEN=("puppeteer" "chromium" "canvas" "jsdom")
for substr in "${FORBIDDEN[@]}"; do
  if grep -q "$substr" "$DIST_FILE"; then
    echo "FAIL: dist/server.js contains forbidden substring: '$substr'" >&2
    exit 1
  fi
done
echo "OK: no forbidden substrings found"

# 6. Server responds to JSON-RPC initialize + tools/list from a fresh tmpdir
#    Check 1 above already exits if dist/server.js is absent, so reaching here
#    guarantees the file exists.
TMPDIR_RUN="$(mktemp -d)"
echo "Testing server startup in tmpdir: $TMPDIR_RUN"
cp "$DIST_FILE" "$TMPDIR_RUN/server.js"

JSON_RPC_INPUT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"sanity-check","version":"0.0.1"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
'

INPUT_FILE="$TMPDIR_RUN/input.jsonl"
OUTPUT_FILE="$TMPDIR_RUN/out.jsonl"
printf '%s' "$JSON_RPC_INPUT" > "$INPUT_FILE"

# Determine timeout command (macOS lacks timeout(1); use perl alarm as fallback)
TIMEOUT_CMD=$(command -v gtimeout || command -v timeout || echo "")

if [ -n "$TIMEOUT_CMD" ]; then
  "$TIMEOUT_CMD" 10 node "$TMPDIR_RUN/server.js" < "$INPUT_FILE" > "$OUTPUT_FILE" 2>/dev/null
  NODE_EXIT=$?
else
  perl -e 'alarm 10; exec @ARGV' node "$TMPDIR_RUN/server.js" < "$INPUT_FILE" > "$OUTPUT_FILE" 2>/dev/null
  NODE_EXIT=$?
fi

if [[ $NODE_EXIT -ne 0 ]]; then
  echo "FAIL: server exited with code $NODE_EXIT (or timed out after 10s)" >&2
  echo "bundle-sanity: server did not respond within 10s" >&2
  rm -rf "$TMPDIR_RUN"
  exit 1
fi

# Parse and assert the JSON-RPC responses using node
node -e "
const fs = require('fs');
const lines = fs.readFileSync('$OUTPUT_FILE', 'utf8').trim().split('\n').filter(l => l.trim());

if (lines.length < 2) {
  console.error('FAIL: expected at least 2 JSON-RPC response lines, got ' + lines.length);
  process.exit(1);
}

let id2Response = null;
for (const line of lines) {
  let obj;
  try { obj = JSON.parse(line); } catch (e) {
    console.error('FAIL: could not parse response line as JSON: ' + line);
    process.exit(1);
  }
  if (obj.id === 2) { id2Response = obj; }
}

if (!id2Response) {
  console.error('FAIL: no JSON-RPC response with id=2 (tools/list) found in output');
  process.exit(1);
}

if (id2Response.error !== undefined) {
  console.error('FAIL: tools/list response (id=2) has an error field: ' + JSON.stringify(id2Response));
  process.exit(1);
}

if (!id2Response.result || !Array.isArray(id2Response.result.tools)) {
  console.error('FAIL: tools/list response (id=2) result.tools is not an array: ' + JSON.stringify(id2Response));
  process.exit(1);
}

console.log('OK: server responds to JSON-RPC initialize + tools/list (tools count: ' + id2Response.result.tools.length + ')');
"
PARSE_EXIT=$?
rm -rf "$TMPDIR_RUN"
if [[ $PARSE_EXIT -ne 0 ]]; then
  exit 1
fi

echo "=== bundle-sanity: all checks passed ==="
