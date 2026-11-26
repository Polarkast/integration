# Sandbox

Hosted studio sandbox: `https://sandbox.polarkast.tech`. Use it after [conformance.md](conformance.md) passes.

## 1. What you get
A human signs in to the client area at `https://client.polarkast.tech` with their email, opens Integration and creates a sandbox operator (choose its id and RNG mode; up to 3 per person). It shows the operator id and the studio base URL, and shows the shared secret once, at creation and again at each rotation; the secret cannot be read later, and rotating makes the old one stop working at once. Keep all three in configuration, under one set of names everywhere:

| Variable | Value |
| --- | --- |
| `POLARKAST_OPERATOR_ID` | the operator id (sent as `X-Operator-Id`) |
| `POLARKAST_SHARED_SECRET` | the shared secret (signs requests, verifies the wallet's) |
| `POLARKAST_STUDIO_URL` | the studio base URL, `https://sandbox.polarkast.tech` |

You register your sandbox wallet's base URL, and every later change to it, in the client area (Integration, Wallet URL): the sandbox calls only the URL registered for your operator. It must be `https://` and resolve to a public address; loopback, private, link-local, CGNAT and cloud-metadata addresses are refused, both when you save it and on every call, and redirects are not followed. A quick-tunnel URL changes every run, so save the new one each time before you launch. Never invent these values; if you lack them, stop and ask a human to create the operator and read out the secret.

A new operator has the games `stub` and `four-star-dragons` and the currency EUR with precision 2 (stakes from 1 to 1,000,000 minor units). Another currency or game is enabled on request; until then launch returns `NOT_AVAILABLE` (see [launch.md](launch.md)). Your test player's wallet must return that currency and precision from `/wallet/authenticate`.

The test game is `stub`: play it first (step 5), then the real game, `four-star-dragons` (step 6). `stub`'s outcomes are deterministic when forced (`force_outcome` works only in the sandbox and only for `stub`):

| `force_outcome` | multiplier | payout for bet 100 |
|---|---|---|
| `0` | 0.00x (loss) | 0 |
| `2` | 2.00x (win) | 200 |

Other indexes: `1` is 1.00x, `3` is 5.00x, `4` is 16.00x. Payout is `bet * multiplier`.

## 2. Make your wallet reachable over HTTPS
The sandbox calls your wallet over the internet, so it needs a public HTTPS URL. Locally, run a Cloudflare quick tunnel (no account):

```sh
cloudflared tunnel --url http://localhost:<port>
```

It prints `https://<random>.trycloudflare.com`. That URL changes on every run: keep the tunnel running, or save the new URL in the client area each time. Staging and production wallets use their own public endpoints.

## 3. Conformance first
Run the conformance CLI against your wallet ([conformance.md](conformance.md)). Do not go on until it exits 0.

## 4. Launch
Call `POST /v1/launch` on `$POLARKAST_STUDIO_URL` with `$POLARKAST_OPERATOR_ID` and `$POLARKAST_SHARED_SECRET` ([launch.md](launch.md)), using a sandbox test player whose balance is at least 1000 minor units. This must be a different player from the one the conformance CLI used, which moves its player's balance. You get `https://sandbox.polarkast.tech/g/<game_id>?t=<token>`.

## 5. Play a forced win and a forced loss
Without a game client, use the game client's own endpoints (spec: tag `game`; not part of the operator contract, you never call them in production). `launch_token` is the `t` query parameter of `game_url`; it works once and is valid for 5 minutes:

```sh
curl -s https://sandbox.polarkast.tech/game/open -H 'Content-Type: application/json' \
  -d '{"launch_token":"<t>"}'
# -> {"game_session_id":"...","player_id":"...","game_id":"stub","currency":"EUR","precision":2,"balance":...,
#     "server_seed_commitment":"...","client_seed":"...","next_nonce":0}

curl -s https://sandbox.polarkast.tech/game/round -H 'Content-Type: application/json' \
  -d '{"game_session_id":"<id>","request_id":"r1","bet":100,"force_outcome":2}'
# -> {"round_id":"...","outcome_index":2,"payout":200,"balance":...}
```

`/game/open` fields: `game_session_id` (pass it to `/game/round`), `player_id`, `game_id`, `currency`, `precision`, `balance` (minor units, read from your wallet), `lobby_url` (when launch sent one) and `currency_symbol` (when Polarkast configured one for your currency). It also accepts an optional `device` (`mobile` or `desktop`), which the game client sends, never you. On provably fair operators it also returns `server_seed_commitment` (SHA-256 commitment to the server seed, shown before the first round), `client_seed` (the client seed in use) and `next_nonce` (the nonce the next round uses); a verifier needs these three, a plain integration ignores them. Full shapes: the spec.

Repeat with a new `request_id` and `"force_outcome":0` for a loss (payout 0). A repeated `request_id` returns the stored result and places no second bet.

Check: after each round, your wallet's balance changed by exactly `-bet + payout` (win: +100, loss: -100 for bet 100), and the `balance` in the response equals your wallet's balance.

## 6. Play 4 Star Dragons
Launch `four-star-dragons` as in step 4 with `"game_id":"four-star-dragons"` and open `game_url` in a browser: that is the real game, playing against your wallet. Without a browser, `/game/open` also returns the play amounts (`bets`), the opening play amount (`bet`) and the player's pots; then play a base round and a bought round (there is no `force_outcome` for this game):

```sh
curl -s https://sandbox.polarkast.tech/game/round -H 'Content-Type: application/json' \
  -d '{"game_session_id":"<id>","request_id":"h1","bet":<bet>}'
curl -s https://sandbox.polarkast.tech/game/round -H 'Content-Type: application/json' \
  -d '{"game_session_id":"<id>","request_id":"h2","bet":<bet>,"mode":"buy"}'
# -> {"round_id":"...","mode":"buy","bet":...,"cost":...,"payout":...,"balance":...,"script":{...}}
```

Check: after each round, your wallet's balance changed by exactly `-cost + payout`. Two base rounds at one play amount carry the pots: the second round's `script.potsBefore` equals the first one's `script.potsAfter`. Details: [four-star-dragons.md](four-star-dragons.md).

## What stays human
- Enabling games, currencies and stake limits, suspending or resuming your operator (through your Polarkast contact).
- Creating the sandbox operator, reading its secret, rotating it, and registering or changing your wallet URL (a person signed in to the client area).
- Switching to production: credentials, compliance approval and go-live sign-off ([index.md](index.md)).
