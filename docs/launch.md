# Launch

Implements: [openapi.yaml](openapi.yaml), `POST /v1/launch`. The studio base URL (`POLARKAST_STUDIO_URL`), `X-Operator-Id` (`POLARKAST_OPERATOR_ID`) and the shared secret (`POLARKAST_SHARED_SECRET`) come from the client area, where a person creates the sandbox operator ([sandbox.md](sandbox.md)). If you were not given them, implement the launch call with all three as configuration, do not guess values, and report the missing values to a human.

## When
Call it server-side when a player opens a game, with a valid session of that player.

## Request
Headers: `Content-Type: application/json`, `X-Operator-Id`, `X-Signature` (see [signing.md](signing.md)).
Body fields: `session_token`, `player_id`, `game_id`, `currency`, `language`, and optionally `brand`, `country` and `lobby_url`. Types and ranges are in the spec.

- `brand`: for an operator that runs several brands behind one wallet URL. Ask your Polarkast contact to register each brand first. Polarkast then sends it as `brand` on every wallet call of that session ([wallet.md](wallet.md)). An operator without brands omits it.
- `country`: the player's country, two uppercase letters (ISO 3166-1 alpha-2). A game blocked in that country is refused.
- `lobby_url`: an `http://` or `https://` URL; the game's home button goes there.

Coin currencies can show a symbol instead of a cash sign: Polarkast keeps it in your operator settings, so launch does not carry it. The mobile or desktop device is detected by the game client, never sent by you.

## What happens
Polarkast calls your `/wallet/authenticate` with the token. Failures come back in the standard error body ([errors.md](errors.md)); the spec says how wallet errors map.

Before it calls your wallet, Polarkast checks the settings it keeps for your operator. A suspended operator gets `OPERATOR_SUSPENDED`; a game that is not enabled for you (new operators start with none), is switched off, or is blocked in the request's `country`, and a `brand` you have not registered, get `NOT_AVAILABLE`. After `authenticate` returns the session's currency and precision, they must be enabled for you too, otherwise `NOT_AVAILABLE`. All of these are HTTP 403 and not retryable: ask your Polarkast contact to enable the game, register the brand, enable the currency and precision, or resume the operator. Stake limits per currency are set the same way; a stake outside them is refused when the player bets, before any wallet call.

## Response
`{"game_url": "..."}`. Redirect or embed the player at `game_url`. The spec states how long it is valid and that it is single-use; call launch again for a new one, never cache it.

## Fun mode
To let a player try a game with play money, for example from a "Demo" button in your lobby, call the same signed launch with `"mode": "fun"`, `game_id`, `currency`, `language` and optionally `lobby_url`, `brand` and `country`. No player is needed: leave out `session_token` and `player_id`. Polarkast calls no wallet endpoint for a fun session and records none of its rounds, so your wallet and `POST /v1/round` never see them. The currency must be enabled for you; the game uses its precision, stake limits and symbol for the play money. Fun sessions end on their own, launches and rounds have rate limits (`RATE_LIMITED`, HTTP 429, retry after the wait), and a fun session never turns into real play: all of it is in the spec, section Fun mode. For real play, launch again without `mode`.

Check: a fun launch for `four-star-dragons` returns a `game_url` without any call reaching your wallet; open it and play.

## Check
Against the sandbox, launch returns a `game_url` for your test player. Pass [conformance.md](conformance.md) first.
Production credentials and go-live are human steps ([index.md](index.md)).
