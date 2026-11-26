---
name: integrate
description: Integrate an operator's casino codebase with the Polarkast seamless-wallet API (five wallet endpoints, signing, idempotency, launch) until the conformance CLI passes. Use when asked to integrate or connect to Polarkast.
---

# Integrate with Polarkast

Read `llms.txt` first, then the pages it lists. The spec `openapi.yaml` is authoritative.

Rules: money is integers in minor units; secrets come from configuration, never code, under the names `POLARKAST_SHARED_SECRET`, `POLARKAST_OPERATOR_ID` and `POLARKAST_STUDIO_URL` (a person creates the sandbox operator in the client area and reads out the secret once); stop and ask a human for production credentials, compliance approval, commercial terms and go-live.

## Procedure
1. Find where the codebase keeps players, balances, currency and sessions.
2. Add `POST /wallet/authenticate`, `/wallet/balance`, `/wallet/bet`, `/wallet/win`, `/wallet/rollback` per `wallet.md`.
3. Verify `X-Signature` on the raw body before parsing, per `signing.md`.
4. Store every money call by `transaction_id` atomically; replay, mismatch and rollback rules per `idempotency.md`. Use only the error codes of `errors.md`.
5. Add the signed `POST /v1/launch` call per `launch.md`, configured from `POLARKAST_STUDIO_URL`, `POLARKAST_OPERATOR_ID` and `POLARKAST_SHARED_SECRET`.
6. Start the wallet, create a conformance test player with at least 1000 minor units, a session token and a token of an expired session (expiry in the past), then run:
   `conformance --wallet-url <url> --session-token <token> --expired-session-token <token>` (secret from `POLARKAST_SHARED_SECRET`)
   Get the binary asset for your platform (`conformance-linux-x86_64`, `conformance-macos-arm64`, `conformance-windows-x86_64.exe`) from https://github.com/Polarkast/integration/releases/tag/conformance-latest (fallback: `cargo build --release -p conformance`).
   Fix each printed failure and rerun (see `conformance.md`). SKIP means not verified; go-live needs zero skips.
7. Make the wallet reachable over public HTTPS (locally `cloudflared tunnel --url http://localhost:<port>`; the URL changes every run, so a human must save the new one under Integration in the client area), launch against the sandbox with a different test player than conformance used, and play one round per `sandbox.md`. The wallet balance must change by exactly -bet + payout.

## Check
The `conformance` command exits 0 with every check PASS and none SKIP, and one sandbox round (`sandbox.md`) settled against your wallet. If it cannot run for lack of credentials or the binary, stop and ask a human.
