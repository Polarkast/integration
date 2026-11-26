# Integrate with Polarkast

Spec: [openapi.yaml](openapi.yaml) (authoritative). Do the steps in order. Each ends with a check.

Rules throughout: money is integers in minor units, never floats; the shared secret comes from configuration, never from code.

## 1. Implement the five wallet endpoints

On the operator's server, add `POST /wallet/authenticate`, `/wallet/balance`, `/wallet/bet`, `/wallet/win`, `/wallet/rollback`. See [wallet.md](wallet.md).
Check: each answers a signed request with the documented body.

## 2. Sign and verify

Verify `X-Signature` on every request over the raw body, before parsing. See [signing.md](signing.md).
Check: a request with a wrong signature gets `INVALID_SIGNATURE`.

## 3. Idempotency and rollback rules

Store every `bet`, `win`, `rollback` by `transaction_id` and replay the stored response on a repeat. Handle rollback of unknown and late bets. See [idempotency.md](idempotency.md) and [errors.md](errors.md).
Check: with your own request (curl or a test), send the same `bet` twice and see the balance move once and the second response equal the first; send a `rollback` for an unknown bet id (200), then a `bet` with that id (`TRANSACTION_ROLLED_BACK`).

## 4. Run the conformance CLI until it passes

See [conformance.md](conformance.md). Fix each reported failure, rerun, repeat. Done when the exit code is 0 and no check is skipped.

## 5. Call launch

Add the signed `POST /v1/launch` call where a player opens a game. See [launch.md](launch.md).
Check: it returns a `game_url`. Then play one round against the hosted sandbox: [sandbox.md](sandbox.md).

## Afterwards: look a round up

Call the signed `POST /v1/round` to get a round's transactions with their status and a link to a round page. See [rounds.md](rounds.md).
Check: looking up a sandbox round by its id lists its bet and win as `succeeded`.

## What stays human

Do not do these; stop and ask a person:

- the commercial agreement with Polarkast
- production credentials, and sandbox credentials: a person creates the sandbox operator, reads its secret and registers the wallet URL in the client area ([sandbox.md](sandbox.md))
- compliance approval
- go-live sign-off
