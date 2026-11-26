# Launch

Implements: [openapi.yaml](openapi.yaml), `POST /v1/launch`. The studio base URL (`POLARKAST_STUDIO_URL`), `X-Operator-Id` (`POLARKAST_OPERATOR_ID`) and the shared secret (`POLARKAST_SHARED_SECRET`) are issued by your Polarkast contact with the sandbox credentials ([sandbox.md](sandbox.md)). If you were not given them, implement the launch call with all three as configuration, do not guess values, and report the missing values to a human.

## When
Call it server-side when a player opens a game, with a valid session of that player.

## Request
Headers: `Content-Type: application/json`, `X-Operator-Id`, `X-Signature` (see [signing.md](signing.md)).
Body fields: `session_token`, `player_id`, `game_id`, `currency`, `language`. Types and ranges are in the spec.

## What happens
Polarkast calls your `/wallet/authenticate` with the token. Failures come back in the standard error body ([errors.md](errors.md)); the spec says how wallet errors map.

Before it calls your wallet, Polarkast checks the settings it keeps for your operator. A suspended operator gets `OPERATOR_SUSPENDED`; a game that is not enabled for you (new operators start with none) or is switched off gets `NOT_AVAILABLE`. After `authenticate` returns the session's currency and precision, they must be enabled for you too, otherwise `NOT_AVAILABLE`. All three are HTTP 403 and not retryable: ask your Polarkast contact to enable the game, or the currency and precision, or to resume the operator. Stake limits per currency are set the same way; a stake outside them is refused when the player bets, before any wallet call.

## Response
`{"game_url": "..."}`. Redirect or embed the player at `game_url`. The spec states how long it is valid and that it is single-use; call launch again for a new one, never cache it.

## Check
Against the sandbox, launch returns a `game_url` for your test player. Pass [conformance.md](conformance.md) first.
Production credentials and go-live are human steps ([index.md](index.md)).
