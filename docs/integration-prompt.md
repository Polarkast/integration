# Integration prompt

Paste into the operator's coding agent. Replace the placeholders.

```text
Integrate this codebase with the Polarkast seamless-wallet API.
Start by reading <DOCS_URL>/llms.txt (the public docs are at https://github.com/Polarkast/integration), then follow the pages in its order.
Studio sandbox base URL: <SANDBOX_BASE_URL> (https://sandbox.polarkast.tech).
Operator id: <OPERATOR_ID>. The shared secret is in <SECRET_LOCATION> (an environment variable or secret store, never in code).
Rules:
- Keep all money as integers in minor units; never floats.
- Never put secrets in code, logs or commits; read them from configuration.
- The OpenAPI spec is authoritative when pages disagree.
- Build the five wallet endpoints, then run the conformance CLI against them and fix failures until it exits 0.
- Stop and ask a human for: credentials you do not have, anything touching production, compliance, or go-live.
- When conformance passes, make the wallet reachable over public HTTPS (locally a cloudflared quick tunnel; its URL changes every run, so a human must give Polarkast the new one), launch against the sandbox and play one sandbox round as in sandbox.md. Your wallet balance must change by exactly -bet + payout.
Then report what you changed.
```
