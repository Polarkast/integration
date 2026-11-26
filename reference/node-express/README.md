# Reference operator wallet (Node/Express)

A minimal seamless-wallet integration for `docs/openapi.yaml`. Copy `src/wallet.ts`. Node 22.18+ runs the TypeScript directly (type stripping, no build step).

```sh
npm ci
POLARKAST_SHARED_SECRET=changeme npm start     # listens on :8081
npm test                             # tsc --noEmit, then node --test
```

## Environment

| Variable | Default | Meaning |
| --- | --- | --- |
| `POLARKAST_SHARED_SECRET` | required | HMAC key shared with Polarkast (the old name `SHARED_SECRET` is still read) |
| `PORT` | `8081` | listen port |
| `PLAYER_ID` | `player-42` | seeded test player |
| `SESSION_TOKEN` | `sess_test` | the player's session token |
| `EXPIRED_SESSION_TOKEN` | unset | if set, seeds an already-expired session for the test player |
| `PLAYER_BALANCE` | `100000` | minor units |
| `PLAYER_CURRENCY` / `PLAYER_PRECISION` | `EUR` / `2` | fixed for the session |

## Where each spec rule lives (`src/wallet.ts`)

| Rule | Function |
| --- | --- |
| Signature over raw bytes, constant-time, checked first | `express.raw` in `createApp`, `signatureOk` |
| Field types, ranges, `INVALID_REQUEST` | `checks`, `schemas`, `validate` |
| One error shape and the closed code list | `err` and the helpers beside it |
| Same id + same body replays, different body or call type is `TRANSACTION_MISMATCH` | `once` |
| Late or replayed bet after rollback is `TRANSACTION_ROLLED_BACK` | `once`, `rollback` handler |
| Rollback of a bet, a rolled-back bet, an unknown bet | `rollback` handler |
| Session valid for authenticate, balance, bet only | `session` |
| Win/rollback: unknown player or another player's bet is `INVALID_REQUEST`; no `CURRENCY_MISMATCH` on rollback | `unknownPlayer`, `rollback` handler |
| Win and rollback accepted after the session expired | `win`, `rollback` handlers (token unchecked) |
| Currency and precision must match the session | `bet`, `win` handlers, `wrongCurrency` |
| Integer amounts, at most 2^53-1 | `checks.amount`, `checks.positive` |
| `100.0` and `1e2` are not integers (`INVALID_REQUEST`) | `parseJson` |
| Only well-formed requests are stored; malformed ones leave the id free | `once` runs after `validate` |
| Body identity compares the spec's fields; unknown fields ignored | `once` (`schemas[kind]`) |
| Rollback of a refused bet: 200, id recorded as rolled back | `rollback` handler |
| Rollback naming a win or rollback id: `TRANSACTION_MISMATCH` | `rollback` handler (`txs` kind) |
| Bet with another player's token: `INVALID_SESSION` | `session` |

`src/launch.ts` `launch()` is the operator's signed `POST /v1/launch` call.
