# Rivermeet

Implements: [openapi.yaml](openapi.yaml), sections Games and Fun mode. `game_id`: `rivermeet`.

## The game
A 7 by 7 slot where five or more matching symbols touching side to side win, clear and tumble. Every cleared cell becomes a canal, and canals that join the first column (the Euphrates) or the last (the Tigris) are watered. When one canal touches both, the rivers meet and High water starts in the same round: free spins whose wins are multiplied by how much of the land is watered. Watering all 49 cells is the whole land and pays the top prize at once. The player can play a round with the Royal dig (the ante), which makes the rivers meet more often, or buy High water outright: Open the sluices (`bonus`), Torrent (`torrent`, fewer but steeper spins), A canal across (`canal`, one row already dug) or Flood season (`flood`, three rows already dug). RTP is 96.33% for base rounds, 96.37% with the ante, 96.42% for `bonus`, 96.27% for `torrent`, 96.34% for `canal` and 96.38% for `flood`; one round pays at most 25,000 times the play amount. The client shows the full rules and the game sheet.

## What your wallet sees
One bet and one win per round, like any game ([wallet.md](wallet.md)). The bet's `amount` is the round's cost: the play amount for a base round, then 1.35, 131.94, 131.94, 190.05 and 293 times it for the ante, `bonus`, `torrent`, `canal` and `flood` (spec: `CanalsGame.costs`). Each is offered only at play amounts where its cost is a whole number of minor units, so its return holds at every play amount: the ante and `canal` at multiples of 20, `bonus` and `torrent` at multiples of 50, `flood` at any (spec: section Games); another play amount in that mode is `INVALID_REQUEST`. Your stake limits for the currency apply to the cost, so the game offers each mode only at play amounts whose cost is within your maximum stake. The win's `amount` is the payout; 0 closes a losing round. Nothing carries from one round to the next.

## Play amounts
The game offers your minimum stake for the currency, then steps of 1, 2 and 5 up to your maximum stake (spec: `GameOpenResponse.bets`). Ask your Polarkast contact to set stake limits per currency.

## Launch and rounds
Launch it like any game ([launch.md](launch.md)); `game_url` serves the game, which talks only to Polarkast. For play money in your lobby, launch with `"mode": "fun"` (launch.md, Fun mode). `POST /v1/round` describes each round in `canals`: mode, play amount, cost and win split between the base spin and High water ([rounds.md](rounds.md)).

## Check
In the sandbox, launch it for your test player, open `game_url` in a browser and play a base round, a round with the ante and a bought round: each time your wallet's balance changes by exactly `-cost + payout`. Look each one up with `POST /v1/round`: its `canals.cost` is the amount your wallet was debited.
