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

# 3. Bundle size < 5 MB
BUNDLE_SIZE=$(wc -c < "$DIST_FILE")
MAX_SIZE=$((5 * 1024 * 1024))
if [[ "$BUNDLE_SIZE" -ge "$MAX_SIZE" ]]; then
  echo "FAIL: dist/server.js is ${BUNDLE_SIZE} bytes (>= 5 MB limit)" >&2
  exit 1
fi
echo "OK: bundle size is ${BUNDLE_SIZE} bytes (< 5 MB)"

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
TMPDIR_RUN="$(mktemp -d)"
echo "Testing server startup in tmpdir: $TMPDIR_RUN"
cp "$DIST_FILE" "$TMPDIR_RUN/server.js"

JSON_RPC_INPUT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"sanity-check","version":"0.0.1"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
'

RESPONSE=$(echo "$JSON_RPC_INPUT" | node "$TMPDIR_RUN/server.js" 2>/dev/null || true)

if [[ -z "$RESPONSE" ]]; then
  echo "FAIL: server produced no response to initialize + tools/list" >&2
  rm -rf "$TMPDIR_RUN"
  exit 1
fi

# Check that response contains jsonrpc field (basic JSON-RPC response)
if ! echo "$RESPONSE" | grep -q '"jsonrpc"'; then
  echo "FAIL: server response does not look like JSON-RPC (no 'jsonrpc' field)" >&2
  echo "Response was: $RESPONSE" >&2
  rm -rf "$TMPDIR_RUN"
  exit 1
fi

rm -rf "$TMPDIR_RUN"
echo "OK: server responds to JSON-RPC initialize + tools/list"

echo "=== bundle-sanity: all checks passed ==="
