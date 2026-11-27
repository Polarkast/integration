# Integration prompt

Paste into the operator's coding agent. Replace the placeholders.

```text
Integrate this codebase with the Polarkast seamless-wallet API.
Start by reading <DOCS_URL>/llms.txt (the public docs are at https://github.com/Polarkast/integration), then follow the pages in its order.
Read configuration from the environment variables POLARKAST_STUDIO_URL (the studio sandbox base URL, https://sandbox.polarkast.tech), POLARKAST_OPERATOR_ID (<OPERATOR_ID>) and POLARKAST_SHARED_SECRET (the shared secret, from <SECRET_LOCATION>, never in code).
Rules:
- Keep all money as integers in minor units; never floats.
- Never put secrets in code, logs or commits; read them from configuration.
- The OpenAPI spec is authoritative when pages disagree.
- Build the five wallet endpoints, then run the conformance CLI against them and fix failures until it exits 0.
- Stop and ask a human for: credentials you do not have, anything touching production, compliance, or go-live.
- When conformance passes, make the wallet reachable over public HTTPS (locally a cloudflared quick tunnel; its URL changes every run, so a human must save the new one under Integration in the client area), launch against the sandbox and play one sandbox round as in sandbox.md. Your wallet balance must change by exactly -bet + payout.
- Then launch the game four-star-dragons and play a base and a bought round as in sandbox.md step 6 (four-star-dragons.md explains it). Your wallet balance must change by exactly -cost + payout each time. stub stays the forced-outcome test game.
- Also force one rollback on the sandbox with force_rollback as in sandbox.md step 5: your wallet must settle it and its balance must not change.
Then report what you changed.
```
