# Game catalog

Implements: [openapi.yaml](openapi.yaml), `GET /v1/games` and the schemas `GamesResponse`, `CatalogGame` and `CatalogStake`. The studio base URL (`POLARKAST_STUDIO_URL`), `X-Operator-Id` (`POLARKAST_OPERATOR_ID`) and the shared secret (`POLARKAST_SHARED_SECRET`) come from the client area ([sandbox.md](sandbox.md)). If you were not given them, take all three as configuration, do not guess values, and report the missing values to a human.

## What it returns
The games Polarkast offers to import into your lobby, for your operator, ordered by `game_id`: name, category, RTP per round mode, platforms, free-round support, blocked countries, thumbnails and dates, and the stakes of each currency enabled for you. It moves no money and is safe to repeat. Field types, units and ranges are in the spec; rates are integers in basis points (9400 is 94.00%), never floats.

## Call it
It is a GET, signed like every studio call ([signing.md](signing.md)): `X-Signature` is the HMAC-SHA256 of the empty body, so sign the empty string.

## How often
Sync it when you set up and then about once a day. The list changes when a game is added, retired or switched on or off for you, not between rounds. A game is never listed before Polarkast has its RTP.

## Reading it
- `enabled`: you can launch the game now. A game that is listed but not enabled for you, or switched off, has `enabled: false`; ask your Polarkast contact to enable it ([launch.md](launch.md)). A suspended operator is still answered, with every game `enabled: false`.
- `stakes` and `bets`: one entry per currency enabled for you, with your minimum and maximum stake in minor units. For hold-and-win, pay-anywhere, canals and offers games `bets` lists the play amounts a session in that currency offers, the same as `/game/open`; a free round's `bet` must be one of them.
- `modes`: `base` first, then the game's other modes (`buy`, `ante`; for a canals game `ante`, `bonus`, `torrent`, `canal`, `flood`; for an offers game `rare`, `ante`, `bonus`, `grand`), each with its `cost_x100` (hundredths of the play amount) and its own `rtp`.
- RTP variants: a variant of a game is a game of its own, with its own `game_id` and `rtp`. Enable the one you want.
- `end_date`: the game is being retired. It stays listed and playable through that day (UTC); after it, it is no longer listed and launches and new rounds are `NOT_AVAILABLE` (rounds already open still settle). Stop offering it to players before then.
- `blocked_countries`: launches with one of these as `country` get `NOT_AVAILABLE`.

## Check
With placeholders for your values, the signature of the empty body goes in the header and the response lists your games:

```sh
SIG=$(printf '' | openssl dgst -sha256 -hmac "$POLARKAST_SHARED_SECRET" -hex | awk '{print $NF}')
curl -s "$POLARKAST_STUDIO_URL/v1/games" -H "X-Operator-Id: $POLARKAST_OPERATOR_ID" -H "X-Signature: $SIG"
```

Done when the status is 200 and `games` has `stub` with `rtp` 9600. The same call with a wrong signature returns 401 `INVALID_SIGNATURE` ([errors.md](errors.md)).
