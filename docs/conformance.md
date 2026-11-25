# Conformance CLI

Tests an operator wallet the way Polarkast calls it. The rules it checks come from [openapi.yaml](openapi.yaml).

## Prepare
1. Run your wallet where the CLI can reach it.
2. Create a test player with a balance of at least 1000 minor units and an active session token.
3. Optional: a token of an expired session for the same player. It enables the expired-session checks for win and rollback.
4. Use the sandbox shared secret issued to you. Pass it from an environment variable or secrets store; do not commit it in scripts.

## Binary
Download the `conformance` binary for your platform (Linux x86_64, macOS arm64, Windows x86_64) from the `conformance-latest` release: https://github.com/Polarkast/integration/releases/tag/conformance-latest. Fallback: build it from the repository you got this documentation from:

```sh
cargo build --release -p conformance   # binary: target/release/conformance
```

## Run
```sh
export POLARKAST_SHARED_SECRET=<from your secrets store>   # read when --secret is absent
conformance --wallet-url <base URL of your wallet> \
  --session-token <test player token> [--expired-session-token <expired token>]
```
`--secret <secret>` also works but shows the secret in the process list; prefer the variable.

## Read the result
- It prints PASS or FAIL per check: duplicates, retries, rollbacks (including unknown and late bets), insufficient funds, bad signature, currency and precision mismatch, unknown player and session token on win and rollback, stored errors replayed, expired session for win and rollback.
- For each failure it prints the request sent, the expected and the actual result, and the fix.
- Exit code 0 only when every check passes.

## Loop
1. Run. 2. Fix the first failure using its printed fix and [wallet.md](wallet.md), [idempotency.md](idempotency.md) or [errors.md](errors.md). 3. Rerun. Stop at exit code 0.

The CLI changes the test player's balance; use a test player only. Passing does not replace the human steps in [index.md](index.md).
