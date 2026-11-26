# Nova Forge

Implements: [openapi.yaml](openapi.yaml), sections Games and Fun mode. `game_id`: `nova-forge`.

## The game
A 6 by 5 slot that pays anywhere: eight or more of one symbol pay wherever they land, and the winning symbols tumble out while new ones drop in, until nothing pays. Multiplier symbols multiply a winning spin. Four scatters start 15 free spins, in which every winning spin's multipliers add to one multiplier that only grows; three scatters add 5 more. The player can play a round with the ante, which costs more and lands scatters more often, or buy a round that starts the free spins at once. RTP is 93.99% for base rounds, 93.97% with the ante and 94.00% for bought rounds; one round pays at most 5,000 times the play amount. The client shows the full rules and the game sheet.

## What your wallet sees
One bet and one win per round, like any game ([wallet.md](wallet.md)). The bet's `amount` is the round's cost: the play amount for a base round, 1.25 times it with the ante, 100 times it for a bought round. The ante is offered only at play amounts where 1.25 times is a whole number of minor units (multiples of 4), so its return holds at every play amount (spec: section Games). Your stake limits for the currency apply to the cost, so the game offers the ante and a bought round only at play amounts whose cost is within your maximum stake. The win's `amount` is the payout; 0 closes a losing round. Nothing carries from one round to the next.

## Play amounts
The game offers your minimum stake for the currency, then steps of 1, 2 and 5 up to your maximum stake (spec: `GameOpenResponse.bets`). Ask your Polarkast contact to set stake limits per currency.

## Launch and rounds
Launch it like any game ([launch.md](launch.md)); `game_url` serves the game, which talks only to Polarkast. For play money in your lobby, launch with `"mode": "fun"` (launch.md, Fun mode). `POST /v1/round` describes each round in `pay_anywhere`: mode, play amount, cost and win split ([rounds.md](rounds.md)).

## Check
In the sandbox, launch it for your test player, open `game_url` in a browser and play a base round, a round with the ante and a bought round: each time your wallet's balance changes by exactly `-cost + payout`. Look each one up with `POST /v1/round`: its `pay_anywhere.cost` is the amount your wallet was debited.
