# Sandbox

Hosted studio sandbox: `https://sandbox.polarkast.tech`. Use it after [conformance.md](conformance.md) passes.

## 1. What you get
Polarkast issues, per operator: an operator id, a shared secret and the studio base URL. You give Polarkast your sandbox wallet's base URL. Never invent these values; if you lack them, stop and ask a human.

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
Call `POST /v1/launch` on `https://sandbox.polarkast.tech` with your operator id and secret ([launch.md](launch.md)), using a test player whose balance is at least 1000 minor units. You get `https://sandbox.polarkast.tech/g/<game_id>?t=<token>`.

## 5. Play a forced win and a forced loss
Without a game client, use the sandbox test endpoints with the `t` value from `game_url` (single-use, valid 5 minutes):

```sh
curl -s https://sandbox.polarkast.tech/game/open -H 'Content-Type: application/json' \
  -d '{"launch_token":"<t>"}'
# -> {"game_session_id":"...","player_id":"...","game_id":"stub","currency":"EUR","precision":2,"balance":...}

curl -s https://sandbox.polarkast.tech/game/round -H 'Content-Type: application/json' \
  -d '{"game_session_id":"<id>","request_id":"r1","bet":100,"force_outcome":2}'
# -> {"round_id":"...","outcome_index":2,"payout":200,"balance":...}
```

Repeat with a new `request_id` and `"force_outcome":0` for a loss (payout 0). A repeated `request_id` returns the stored result and places no second bet.

Check: after each round, your wallet's balance changed by exactly `-bet + payout` (win: +100, loss: -100 for bet 100), and the `balance` in the response equals your wallet's balance.

## What stays human
- Sandbox credentials and giving Polarkast your wallet URL.
- Switching to production: credentials, compliance approval and go-live sign-off ([index.md](index.md)).
