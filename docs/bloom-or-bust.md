# Bloom or Bust

Implements: [openapi.yaml](openapi.yaml), sections Games and Fun mode. `game_id`: `bloom-or-bust`.

## The game
A trade with no grid: a bid climbs from 1.00x and breaks at a point drawn with the round. Up to 8 offers are pinned on the board before the climb, each at 1.1x or more; an offer the bid reaches is signed, and the trade wins the highest signed offer. Some boards carry a golden seal; when the bid reaches it, the Mania follows in the same round: the seal's bid is added, then three bubbles play, each climbing from where the last left off and adding its best signed offer. A bid that reaches the top of the board wins the top prize, Semper Augustus, and ends the round. The player can trade a Rare bulb (`rare`: fewer, higher offers and a longer climb), trade a Busy market (`ante`: a golden seal on every board), or start the Mania outright: Start the Mania (`bonus`, a seal at 15x) or the Grand Mania (`grand`, a seal at 50x). RTP is 96.34% for base rounds, 96.38% for `rare`, 96.31% with the ante, 96.37% for `bonus` and 96.34% for `grand`; one round pays at most 25,000 times the play amount. The client shows the full rules and the game sheet.

## What your wallet sees
One bet and one win per round, like any game ([wallet.md](wallet.md)). The bet's `amount` is the round's cost: the play amount for a base or `rare` round, then 1.41, 117.84 and 386.1 times it for the ante, `bonus` and `grand` (spec: `OffersGame.costs`). The ante, `bonus` and `grand` are offered only at play amounts where their cost is a whole number of minor units, so their return holds at every play amount: the ante at multiples of 100, `bonus` at multiples of 25 and `grand` at multiples of 10 (spec: section Games); another play amount in that mode is `INVALID_REQUEST`. Your stake limits for the currency apply to the cost, so the game offers each mode only at play amounts whose cost is within your maximum stake. The win's `amount` is the payout; 0 closes a losing round. Nothing carries from one round to the next.

## Play amounts
The game offers your minimum stake for the currency, then steps of 1, 2 and 5 up to your maximum stake (spec: `GameOpenResponse.bets`). Ask your Polarkast contact to set stake limits per currency.

## Launch and rounds
Launch it like any game ([launch.md](launch.md)); `game_url` serves the game, which talks only to Polarkast. For play money in your lobby, launch with `"mode": "fun"` (launch.md, Fun mode). `POST /v1/round` describes each round in `offers`: mode, play amount, cost and win split between the trade and the Mania ([rounds.md](rounds.md)).

## Check
In the sandbox, launch it for your test player, open `game_url` in a browser and play a base round, a round with the ante and a started Mania: each time your wallet's balance changes by exactly `-cost + payout`. Look each one up with `POST /v1/round`: its `offers.cost` is the amount your wallet was debited.
