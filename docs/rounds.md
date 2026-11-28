# Round lookup

Implements: [openapi.yaml](openapi.yaml), `POST /v1/round`. Same base URL, `X-Operator-Id` and shared secret as launch ([launch.md](launch.md)); sign the body as in [signing.md](signing.md).

## When
Call it server-side when you need to see a round: a support case, a back office that embeds a round page, or reconciliation of a transaction you are unsure about.

## Request
Body: exactly one of `round_id` or `transaction_id` (a bet, win or rollback id). Both, or neither, is `INVALID_REQUEST`. You can read only your own rounds; any other id is the same `INVALID_REQUEST` as an unknown one.

## Response
- `transactions`: every bet, win and rollback of the round in the order Polarkast logged it, each with its `status`. The spec defines each status (`pending`, `succeeded`, `failed`, `needs_reconciliation`, `resolved`) and what to do about it. A `needs_reconciliation` entry is for you and your Polarkast contact to settle; do not guess at it.
- `round_url`: a link to an HTML page of the round (game, time, every transaction, the outcome, and for provably fair play the proof). Embed or open it; the spec states how long it is valid. It is a bearer link: do not log or share it, and call again for a fresh one when it expires.

- `hold_and_win`: only for a hold-and-win game such as [4 Star Dragons](four-star-dragons.md): the round's `mode` (`base` or `buy`), the play amount `bet`, the amount debited `cost` (the bet transaction's amount), the win split `win_x100` (lines, collect, feature; hundredths of the play amount) and the player's pots before and after (`null` for a bought round). The page adds the round's key moments and how to replay it from its recorded draws.
- `pay_anywhere`: only for a pay-anywhere game such as [Nova Forge](nova-forge.md): the round's `mode` (`base`, `ante` or `buy`), the play amount `bet`, the amount debited `cost` and the win split `win_x100` (base spin and free spins; hundredths of the play amount). The page adds the round's key moments and how to replay it from its recorded draws.
- `canals`: only for a canals game such as [Rivermeet](rivermeet.md): the round's `mode` (`base`, `ante`, `bonus`, `torrent`, `canal` or `flood`), the play amount `bet`, the amount debited `cost` and the win split `win_x100` (base spin and High water; hundredths of the play amount). The page adds the round's key moments and how to replay it from its recorded draws.
- `offers`: only for an offers game such as [Bloom or Bust](bloom-or-bust.md): the round's `mode` (`base`, `rare`, `ante`, `bonus` or `grand`), the play amount `bet`, the amount debited `cost` and the win split `win_x100` (trade and Mania; hundredths of the play amount). The page adds the round's key moments and how to replay it from its recorded draws.

Neither the response nor the page contains session tokens, signatures or your raw wallet requests and responses. Fun rounds are never recorded, so they are never found.

## Check
Play a round in the sandbox ([sandbox.md](sandbox.md)), then look it up by its `round_id` and again by its bet's `transaction_id`: both list the same transactions with status `succeeded`. Open `round_url` in a browser and see the outcome. A body with a wrong signature gets `INVALID_SIGNATURE`.
