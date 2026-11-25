# Integration prompt

Paste into the operator's coding agent. Replace the two placeholders.

```text
Integrate this codebase with the Polarkast seamless-wallet API.
Start by reading <DOCS_URL>/llms.txt, then follow the pages in its order.
Sandbox credentials: <CREDENTIALS_REFERENCE>.
Rules:
- Keep all money as integers in minor units; never floats.
- Never put secrets in code, logs or commits; read them from configuration.
- The OpenAPI spec is authoritative when pages disagree.
- Build the five wallet endpoints, then run the conformance CLI against them and fix failures until it exits 0.
- Stop and ask a human for: credentials you do not have, anything touching production, compliance, or go-live.
When conformance passes, report what you changed.
```
