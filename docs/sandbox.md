# Sandbox

Hosted studio sandbox: `https://sandbox.polarkast.tech`. Use it after [conformance.md](conformance.md) passes.

## 1. What you get
Request sandbox credentials from your Polarkast contact. Polarkast issues, per operator: an operator id, a shared secret and the studio base URL. Keep them in configuration, under one set of names everywhere:

| Variable | Value |
| --- | --- |
| `POLARKAST_OPERATOR_ID` | the operator id (sent as `X-Operator-Id`) |
| `POLARKAST_SHARED_SECRET` | the shared secret (signs requests, verifies the wallet's) |
| `POLARKAST_STUDIO_URL` | the studio base URL, `https://sandbox.polarkast.tech` |

You register your sandbox wallet's base URL, and every later change to it, through your Polarkast contact: the sandbox calls only the URL registered for your operator. A quick-tunnel URL changes every run, so send the new one each time before you launch. Never invent these values; if you lack them, stop and ask a human.

Polarkast enables the game `stub` and the currency EUR with precision 2 (stakes from 1 to 1,000,000 minor units) for your operator. Another currency or game is enabled on request; until then launch returns `NOT_AVAILABLE` (see [launch.md](launch.md)). Your test player's wallet must return that currency and precision from `/wallet/authenticate`.

The sandbox game is `stub`. Its outcomes are deterministic when forced (`force_outcome` works only in the sandbox):

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

It prints `https://<random>.trycloudflare.com`. That URL changes on every run: keep the tunnel running, or send Polarkast the new URL each time. Staging and production wallets use their own public endpoints.

## 3. Conformance first
Run the conformance CLI against your wallet ([conformance.md](conformance.md)). Do not go on until it exits 0.

## 4. Launch
Call `POST /v1/launch` on `$POLARKAST_STUDIO_URL` with `$POLARKAST_OPERATOR_ID` and `$POLARKAST_SHARED_SECRET` ([launch.md](launch.md)), using a sandbox test player whose balance is at least 1000 minor units. This must be a different player from the one the conformance CLI used, which moves its player's balance. You get `https://sandbox.polarkast.tech/g/<game_id>?t=<token>`.

## 5. Play a forced win and a forced loss
Without a game client, use the sandbox test endpoints (spec: tag `sandbox`; sandbox only, not part of the operator contract). `launch_token` is the `t` query parameter of `game_url`; it works once and is valid for 5 minutes:

```sh
curl -s https://sandbox.polarkast.tech/game/open -H 'Content-Type: application/json' \
  -d '{"launch_token":"<t>"}'
# -> {"game_session_id":"...","player_id":"...","game_id":"stub","currency":"EUR","precision":2,"balance":...,
#     "server_seed_commitment":"...","client_seed":"...","next_nonce":0}

curl -s https://sandbox.polarkast.tech/game/round -H 'Content-Type: application/json' \
  -d '{"game_session_id":"<id>","request_id":"r1","bet":100,"force_outcome":2}'
# -> {"round_id":"...","outcome_index":2,"payout":200,"balance":...}
```

`/game/open` fields: `game_session_id` (pass it to `/game/round`), `player_id`, `game_id`, `currency`, `precision`, `balance` (minor units, read from your wallet). On provably fair operators it also returns `server_seed_commitment` (SHA-256 commitment to the server seed, shown before the first round), `client_seed` (the client seed in use) and `next_nonce` (the nonce the next round uses); a verifier needs these three, a plain integration ignores them. Full shapes: the spec.

Repeat with a new `request_id` and `"force_outcome":0` for a loss (payout 0). A repeated `request_id` returns the stored result and places no second bet.

Check: after each round, your wallet's balance changed by exactly `-bet + payout` (win: +100, loss: -100 for bet 100), and the `balance` in the response equals your wallet's balance.

## What stays human
- Enabling games, currencies and stake limits, suspending or resuming your operator (through your Polarkast contact).
- Sandbox credentials and registering or changing your wallet URL (through your Polarkast contact).
- Switching to production: credentials, compliance approval and go-live sign-off ([index.md](index.md)).
