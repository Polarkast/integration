# Wallet endpoints

Implements: [openapi.yaml](openapi.yaml), paths `/wallet/*`, and the sections Transport, Money, Identifiers. Polarkast calls these on the operator's base URL. Every request is signed ([signing.md](signing.md)). Every failure uses the error body in [errors.md](errors.md). Amounts are integers in minor units with `currency` and `precision`.

Every wallet request may carry an optional `brand` (spec: Brand): the brand the player launched under, the same on every call of a session, absent when the launch had none. Use it to pick the right brand's player or ledger; ignore it if you have one brand. It is not part of the idempotency body comparison ([idempotency.md](idempotency.md)); a rollback carries its bet's brand.

Order inside every handler: verify signature, validate fields, then act. Integers are integer literals: `100.0` and `1e2` are `INVALID_REQUEST` (spec: Money).

## POST /wallet/authenticate
- When: a player launches a game.
- Do: look up `session_token`. Return `player_id`, the session's `currency`, `precision` and current `balance`. These fix the currency and precision for the session.
- Money: none. Read-only.
- Errors: `INVALID_SESSION` for an unknown or expired token.

## POST /wallet/balance
- When: the game shows or refreshes the balance.
- Do: return `balance`, `currency`, `precision` for the session's player.
- Money: none.
- Errors: `INVALID_SESSION` if the token is invalid or not this `player_id`.

## POST /wallet/bet
- When: a round starts.
- Do: debit `amount` and return the balance after. Must be idempotent ([idempotency.md](idempotency.md)).
- Money: debit once per `transaction_id`.
- Errors: `INSUFFICIENT_FUNDS` (debit nothing), `INVALID_SESSION` (the token must be valid now and be `player_id`'s own: another player's token is refused), `CURRENCY_MISMATCH`, `TRANSACTION_ROLLED_BACK`, `TRANSACTION_MISMATCH`.

## POST /wallet/win
- When: a round ends. `amount` 0 closes a losing round.
- Do: credit `amount`, return the balance after. A zero amount still gets a stored transaction.
- Money: credit once per `transaction_id`.
- Never check the session token's validity or expiry (spec: Win and rollback checks). Still check player, currency and precision.
- Optional `reward_id`: the win closes a free round, whose `round_id` has no bet ([freebets.md](freebets.md)). Credit `amount` like any win, also when the reward has expired since; it is part of the body that idempotency compares.
- Errors: `INVALID_REQUEST` (unknown `player_id`), `CURRENCY_MISMATCH`, `TRANSACTION_MISMATCH`.

## POST /wallet/rollback
- When: Polarkast is unsure whether a bet happened (spec: Timeouts and retries).
- Do: reverse the bet named by `bet_transaction_id`. Answer HTTP 200 for the cases in the spec: bet exists, already rolled back, bet unknown or refused. Record the bet id as rolled back in every case.
- Money: credit the stake back at most once per bet.
- Never check the session token's validity or expiry (spec: Win and rollback checks).
- Errors: `INVALID_REQUEST` (unknown `player_id`, or the bet is another player's), `TRANSACTION_MISMATCH` (also when `bet_transaction_id` is a win's or rollback's id). Never `CURRENCY_MISMATCH` or `INVALID_SESSION`.

Response time targets and retry behaviour are in the spec, section Timeouts and retries.
