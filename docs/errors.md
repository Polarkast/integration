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
| Bad JSON, missing or mistyped field, amount out of range, bet of 0, win or rollback with unknown `player_id` or a rollback of another player's bet | `INVALID_REQUEST` |
| Token unknown, expired or not the player's (authenticate, balance, bet only; never win or rollback) | `INVALID_SESSION` |
| Bet above balance | `INSUFFICIENT_FUNDS` |
| `currency` or `precision` differs from the session's (never on rollback) | `CURRENCY_MISMATCH` |
| Known `transaction_id`, different body or call type | `TRANSACTION_MISMATCH` |
| Bet whose id was already rolled back | `TRANSACTION_ROLLED_BACK` |
| Your own temporary failure (database down) | `INTERNAL_ERROR` |

Check order: signature, then field validation, then business rules.
Never return an HTML error page, an empty body or a code outside the list. Turn unhandled exceptions into `INTERNAL_ERROR` in the same body shape.
