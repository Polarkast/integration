# Errors

Implements: [openapi.yaml](openapi.yaml), schema `ErrorCode` (the list, HTTP statuses and fixes live there; this page does not copy them) and section Errors.

Every failure on every endpoint has this body:

```json
{"error": {"code": "INVALID_SIGNATURE", "message": "..."}}
```

Use only codes from `ErrorCode`. Code and HTTP status are the contract; `message` is for humans.

## Which code to return
| Situation | Code |
| --- | --- |
| Signature header missing or wrong | `INVALID_SIGNATURE` |
| Bad JSON, missing or mistyped field, amount out of range, bet of 0, a number written `100.0` or `1e2` (spec: Money), win or rollback with unknown `player_id` or a rollback of another player's bet | `INVALID_REQUEST` |
| Token unknown, expired or not the player's, including another player's valid token (authenticate, balance, bet only; never win or rollback) | `INVALID_SESSION` |
| Bet above balance | `INSUFFICIENT_FUNDS` |
| `currency` or `precision` differs from the session's (never on rollback) | `CURRENCY_MISMATCH` |
| Known `transaction_id`, different body or call type; a rollback whose `bet_transaction_id` is a win's or a rollback's id | `TRANSACTION_MISMATCH` |
| Bet whose id was already rolled back | `TRANSACTION_ROLLED_BACK` |
| Studio launch only, from Polarkast to you: game not enabled for you or switched off, or your session currency and precision not enabled | `NOT_AVAILABLE` (you receive it; ask Polarkast to enable) |
| Studio launch only: your operator is suspended | `OPERATOR_SUSPENDED` (you receive it; contact Polarkast) |
| Your own temporary failure (database down) | `INTERNAL_ERROR` |

Check order: signature, then field validation, then business rules. Which errors are stored for replay: [idempotency.md](idempotency.md).
Never return an HTML error page, an empty body or a code outside the list. Turn unhandled exceptions into `INTERNAL_ERROR` in the same body shape.
