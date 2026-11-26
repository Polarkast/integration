# Free rounds

Implements: [openapi.yaml](openapi.yaml), section Free rounds, `POST /v1/rewards`, `POST /v1/rewards/cancel`, and `reward_id` on `/wallet/win`. Do [conformance.md](conformance.md) first: your wallet must accept a win with `reward_id`.

## Which games
Only games whose `freebet_support` is true in `GET /v1/games` ([catalog.md](catalog.md)) can be named in a reward; any other game is `INVALID_REQUEST`. The bet must also be a valid stake for each game, as that endpoint's `stakes` and `bets` show.

## Give a player free rounds
Call the signed `POST /v1/rewards` ([signing.md](signing.md), headers as in [launch.md](launch.md)) with your own `reward_id`, the player, currency and precision, the `game_ids`, the number of `rounds`, the `bet` of each round and `expires_at`:

```json
{ "reward_id": "welcome-42", "player_id": "player-42", "currency": "EUR", "precision": 2,
  "game_ids": ["stub"], "rounds": 10, "bet": 100, "expires_at": "2026-12-31T23:59:59Z" }
```

It makes no wallet call and moves no money. It is idempotent on `reward_id`: repeat it after a timeout and you get the reward as it stands (`status`, `rounds_used`, `win`). The same `reward_id` with another body is `TRANSACTION_MISMATCH`. Refusals (suspended operator, a game not enabled or without free rounds, a currency or precision not enabled, a `bet` that is not a valid stake for every game, a bad `expires_at`) are in the spec; a refused create does not use up its `reward_id`.

## What the player sees
A session of that player, currency and precision lists the reward in `/game/open`'s `rewards`, and the game client plays each free round by sending its `reward_id` to `/game/round`. There is no bet and no `/wallet/bet` call.

## What your wallet sees
One `/wallet/win` per free round, a round that won nothing included, with `amount` 0 then. It carries `reward_id` and a `round_id` that no bet has. Credit it like any win and fund it from the reward ([wallet.md](wallet.md)); `reward_id` is part of the body [idempotency.md](idempotency.md) compares. It arrives even after the reward expired or was used up: a round played before expiry always pays out.

## Cancel and expiry
`POST /v1/rewards/cancel` with `{ "reward_id": "welcome-42" }` cancels a reward only before its first free round has started; after that it is `INVALID_REQUEST` and the reward stays. Rounds not started by `expires_at` lapse. Look a free round up with `POST /v1/round` ([rounds.md](rounds.md)): it has `reward_id`, no bet transaction and a `cost` of 0.

## Check
In the sandbox, with your registered wallet ([sandbox.md](sandbox.md)): create a reward for your test player on `stub` with `rounds` 2 and `bet` 100 (use placeholders `$POLARKAST_STUDIO_URL`, `$POLARKAST_OPERATOR_ID`, `$POLARKAST_SHARED_SECRET`), launch `stub`, and play two free rounds against `/game/round` with the reward's `reward_id` and `"bet":100`. Your wallet saw no bet and two wins with `reward_id`, and its balance rose by the two payouts alone. A third free round is refused with `INVALID_REQUEST`, and `POST /v1/rewards` with the same body returns `status` `used`, `rounds_used` 2.
