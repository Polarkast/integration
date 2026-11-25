# Launch

Implements: [openapi.yaml](openapi.yaml), `POST /v1/launch`. The studio base URL and `X-Operator-Id` are issued with the sandbox credentials. If you were not given them, implement the launch call with both as configuration, do not guess values, and report the missing values to a human.

## When
Call it server-side when a player opens a game, with a valid session of that player.

## Request
Headers: `Content-Type: application/json`, `X-Operator-Id`, `X-Signature` (see [signing.md](signing.md)).
Body fields: `session_token`, `player_id`, `game_id`, `currency`, `language`. Types and ranges are in the spec.

## What happens
Polarkast calls your `/wallet/authenticate` with the token. Failures come back in the standard error body ([errors.md](errors.md)); the spec says how wallet errors map.

## Response
`{"game_url": "..."}`. Redirect or embed the player at `game_url`. The spec states how long it is valid and that it is single-use; call launch again for a new one, never cache it.

## Check
Against the sandbox, launch returns a `game_url` for your test player. Pass [conformance.md](conformance.md) first.
Production credentials and go-live are human steps ([index.md](index.md)).
