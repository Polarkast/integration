# Conformance CLI

Tests an operator wallet the way Polarkast calls it. The rules it checks come from [openapi.yaml](openapi.yaml); every failure's fix names the rule.

## Prepare
1. Run your wallet where the CLI can reach it.
2. Create a test player with a balance of at least 1000 minor units and an active session token. The CLI moves this player's balance: use a player for conformance only, and a different player for the sandbox round in [sandbox.md](sandbox.md).
3. Create a token of an expired session for the same player: a session your wallet issued whose expiry time is in the past (set the expiry back in your session store, or let a short-lived one lapse). It enables the expired-session checks for win and rollback. The reference wallet seeds one from `EXPIRED_SESSION_TOKEN`.
4. Use the sandbox shared secret a human reads from the client area ([sandbox.md](sandbox.md)). Pass it from an environment variable or secrets store; do not commit it in scripts.

## Binary
Releases are at https://github.com/Polarkast/integration/releases/tag/conformance-latest. Pick the asset for your platform:

| Platform | Asset |
| --- | --- |
| Linux x86_64 | `conformance-linux-x86_64` |
| macOS arm64 | `conformance-macos-arm64` |
| Windows x86_64 | `conformance-windows-x86_64.exe` |

```sh
curl -fL -o conformance https://github.com/Polarkast/integration/releases/download/conformance-latest/conformance-linux-x86_64   # or the asset for your platform
chmod +x conformance
```
Fallback: build it from the repository you got this documentation from: `cargo build --release -p conformance` (binary: `target/release/conformance`).

## Run
```sh
export POLARKAST_SHARED_SECRET=<from your secrets store>   # read when --secret is absent
./conformance --wallet-url <base URL of your wallet> \
  --session-token <test player token> --expired-session-token <expired token>
```
`--secret <secret>` also works but shows the secret in the process list; prefer the variable.

## Read the result
- It prints PASS, FAIL or SKIP per check: duplicates, retries, rollbacks (including unknown, late and refused bets, and a rollback naming a win or rollback), insufficient funds, bad signature, currency and precision mismatch, another player's token on bet, unknown player and session token on win and rollback, stored errors replayed, malformed requests not stored, unknown fields ignored, `100.0` and `1e2` rejected, expired session for win and rollback.
- For each failure it prints the request sent, the expected and the actual result, and the fix.
- SKIP means not verified: the check did not run (no `--expired-session-token`). A skipped check proves nothing. A go-live needs zero skips as well as zero failures.
- Exit code 0 only when no check fails.

A pass ends like this (the count grows as checks are added):
```text
PASS rollback is accepted with an expired session

39 passed, 0 failed, 0 skipped
```
and the exit code is 0.

## Loop
1. Run. 2. Fix the first failure using its printed fix and [wallet.md](wallet.md), [idempotency.md](idempotency.md) or [errors.md](errors.md). 3. Rerun. Stop at exit code 0 with 0 skipped.

Passing does not replace the human steps in [index.md](index.md).
