---
name: integrate
description: Integrate an operator's casino codebase with the Polarkast seamless-wallet API (five wallet endpoints, signing, idempotency, launch) until the conformance CLI passes. Use when asked to integrate or connect to Polarkast.
---

# Integrate with Polarkast

Read `llms.txt` first, then the pages it lists. The spec `openapi.yaml` is authoritative.

Rules: money is integers in minor units; secrets come from configuration, never code; stop and ask a human for production credentials, compliance approval, commercial terms and go-live.

## Procedure
1. Find where the codebase keeps players, balances, currency and sessions.
2. Add `POST /wallet/authenticate`, `/wallet/balance`, `/wallet/bet`, `/wallet/win`, `/wallet/rollback` per `wallet.md`.
3. Verify `X-Signature` on the raw body before parsing, per `signing.md`.
4. Store every money call by `transaction_id` atomically; replay, mismatch and rollback rules per `idempotency.md`. Use only the error codes of `errors.md`.
5. Add the signed `POST /v1/launch` call per `launch.md`.
6. Start the wallet, create a test player with at least 1000 minor units and a session token, then run:
   `POLARKAST_SHARED_SECRET=<secret> conformance --wallet-url <url> --session-token <token> [--expired-session-token <token>]`
   Get the binary from the `conformance-latest` release of https://github.com/Polarkast/integration (fallback: `cargo build --release -p conformance`).
   Fix each printed failure and rerun (see `conformance.md`).
7. Make the wallet reachable over public HTTPS (locally `cloudflared tunnel --url http://localhost:<port>`; the URL changes every run, so a human must give Polarkast the new one), launch against the sandbox and play one round per `sandbox.md`. The wallet balance must change by exactly -bet + payout.

## Check
The `conformance` command exits 0 with every check PASS, and one sandbox round (`sandbox.md`) settled against your wallet. If it cannot run for lack of credentials or the binary, stop and ask a human.
