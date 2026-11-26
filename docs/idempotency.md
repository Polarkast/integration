# Idempotency and rollback

Implements: [openapi.yaml](openapi.yaml), sections Idempotency and Timeouts and retries.

## Rules for bet, win, rollback
`transaction_id` is one namespace across all three calls. Keep a table keyed by `transaction_id` with the body and the response.

1. Unknown id: execute, then store body and response. Do not store `INTERNAL_ERROR`.
2. Known id, every field identical: return the stored response (same status, same body, same `balance`). Move no money. Every definitive error (`INVALID_SESSION`, `INSUFFICIENT_FUNDS`, ...) is stored and replayed too; only `INTERNAL_ERROR` is not. Exception: a rolled-back bet, see below.
3. Known id, any field different or a different call type: return `TRANSACTION_MISMATCH`. Move no money.

Two details (spec: Idempotency):
- "Identical" compares the fields the spec defines for that call. Unknown fields are ignored: a repeat that differs only in one is a replay. Key order and whitespace do not matter.
- Store only the response to a well-formed request (valid signature, fields pass validation). A malformed request (`INVALID_SIGNATURE`, or `INVALID_REQUEST` for bad JSON, a missing, mistyped or out-of-range field, `100.0` or `1e2`) is never stored and does not use up its id. A definitive error decided by business rules is stored, including `INVALID_REQUEST` for an unknown `player_id`.

Make check-and-write atomic: a unique constraint on `transaction_id` and a database transaction around the balance change. Two simultaneous requests with one id must not both move money.

## Rollback
`rollback` names the bet in `bet_transaction_id`.
- Bet exists, not rolled back: credit the stake, mark it rolled back.
- Bet already rolled back: move nothing, return 200.
- Rollback has no `currency` or `precision`: `CURRENCY_MISMATCH` never applies. The response balance is the bet owner's. Unknown `player_id`, or a bet of another player: `INVALID_REQUEST` (spec: Win and rollback checks).
- Bet unknown: move nothing, return 200, and record the id as rolled back.
- Bet was refused (for example `INSUFFICIENT_FUNDS`, so never debited): same as unknown. Return 200, move nothing, record the id as rolled back. Replaying that bet then returns `TRANSACTION_ROLLED_BACK`, not its stored error.
- `bet_transaction_id` is the id of a `win` or of a `rollback`: return `TRANSACTION_MISMATCH` (409). That id belongs to another call type, which is what the one-namespace rule covers; `INVALID_REQUEST` would be wrong because the request is well-formed. Move nothing and do not record the id as rolled back.
- A bet that arrives later with a rolled-back id: return `TRANSACTION_ROLLED_BACK`, debit nothing. Replaying a rolled-back bet also returns it: `TRANSACTION_ROLLED_BACK` wins over the stored response. Order: `TRANSACTION_MISMATCH`, then `TRANSACTION_ROLLED_BACK`, then replay (spec: Idempotency).

## What Polarkast will do to you
Timeouts, retry conditions and backoff are in the spec (Timeouts and retries). In short:
- `bet` is never retried; after a timeout expect a `rollback` for it, possibly for a bet you never saw or one that arrives later.
- `win` and `rollback` are retried with the identical body until you answer definitively, so duplicates and late arrivals are normal. They arrive even after the session token expired: accept them.
- Return `INTERNAL_ERROR` only for failures where a retry is safe.

Check: run [conformance.md](conformance.md); it exercises duplicates, retries, unknown and late bets.
