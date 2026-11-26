# 4 Star Dragons

Implements: [openapi.yaml](openapi.yaml), sections Games and Fun mode. `game_id`: `four-star-dragons`.

## The game
A 5-reel, 3-row, 20-line slot with a hold-and-win feature, the Star Hoard. Coins carry prizes, and six or more on the board start the Star Hoard: coins stay and empty cells respin. Four dragons, the spec's pots, each fill from their own coins; a full dragon joins the next Star Hoard with its power (a named prize, a multiplier, a collect, or a board opened to 5 by 5). The player can also buy a round that starts the Star Hoard at once. RTP is 93.99% for base and bought rounds; one round pays at most 5,000 times the play amount. The client shows the full rules and the game sheet.

## What your wallet sees
One bet and one win per round, like any game ([wallet.md](wallet.md)). The bet's `amount` is the round's cost: the play amount for a base round, 50 times it for a bought round. Your stake limits for the currency apply to that amount, so the game offers a bought round only at play amounts whose cost is within your maximum stake. The win's `amount` is the payout; 0 closes a losing round.

The dragons are the player's and carry from round to round. Polarkast keeps them per player, currency and play amount; they never expire and never touch your wallet. A bought round leaves them alone, and a bet that fails or is rolled back leaves them as they were. The first time a player uses a play amount, its dragons start part-way, where a long game typically stands.

## Play amounts
The game offers your minimum stake for the currency, then steps of 1, 2 and 5 up to your maximum stake (spec: `GameOpenResponse.bets`). Ask your Polarkast contact to set stake limits per currency.

## Launch and rounds
Launch it like any game ([launch.md](launch.md)); `game_url` serves the game, which talks only to Polarkast. For play money in your lobby, launch with `"mode": "fun"` (launch.md, Fun mode). `POST /v1/round` describes each round in `hold_and_win`: mode, play amount, cost, win split and the dragons before and after ([rounds.md](rounds.md)).

## Check
In the sandbox, launch it for your test player, open `game_url` in a browser and play a base round and a bought round: each time your wallet's balance changes by exactly `-cost + payout`. Close the game and launch again at the same play amount: the dragons are where you left them. Without a browser: [sandbox.md](sandbox.md), step 6.
