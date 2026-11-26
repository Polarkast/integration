#!/bin/sh
# Conformance against the known-good reference (must pass) and a known-bad wallet (must fail on specific checks).
set -eu
cd "$(dirname "$0")/.."
(cd reference/node-express && npm ci --no-audit --no-fund >/dev/null)
cargo build -q -p conformance
BIN=target/debug/conformance
OUT=$(mktemp -d)
PIDS=""
trap 'kill $PIDS 2>/dev/null || true; rm -rf "$OUT"' EXIT
free_port() { node -e 'const s=require("net").createServer().listen(0,()=>{console.log(s.address().port);s.close()})'; }
wait_port() {
  n=0
  until nc -z 127.0.0.1 "$1" 2>/dev/null; do
    n=$((n + 1))
    [ $n -lt 100 ] || { echo "port $1 never opened" >&2; exit 1; }
    sleep 0.1
  done
}

P=$(free_port)
(cd reference/node-express && PORT=$P POLARKAST_SHARED_SECRET=e2e-secret SESSION_TOKEN=sess_e2e EXPIRED_SESSION_TOKEN=sess_e2e_old exec node src/index.ts) >"$OUT/ref.log" 2>&1 &
PIDS="$PIDS $!"
wait_port "$P"
"$BIN" --wallet-url "http://127.0.0.1:$P" --secret e2e-secret --session-token sess_e2e --expired-session-token sess_e2e_old
# Second run: fresh ids per run must work against the same wallet; secret from the environment.
POLARKAST_SHARED_SECRET=e2e-secret "$BIN" --wallet-url "http://127.0.0.1:$P" --session-token sess_e2e --expired-session-token sess_e2e_old >/dev/null

B=$(free_port)
PORT=$B node conformance/testdata/broken-wallet.ts >"$OUT/bad.log" 2>&1 &
PIDS="$PIDS $!"
wait_port "$B"
if "$BIN" --wallet-url "http://127.0.0.1:$B" --secret e2e-secret --session-token any >"$OUT/bad.out"; then
  echo "conformance passed against the broken wallet" >&2
  exit 1
fi
for name in "bad signature" "duplicate bet" "insufficient funds" "late bet"; do
  grep -q "^FAIL .*$name" "$OUT/bad.out" || { echo "expected FAIL for: $name" >&2; cat "$OUT/bad.out" >&2; exit 1; }
done
echo "conformance e2e ok"
