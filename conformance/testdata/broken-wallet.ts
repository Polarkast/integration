// Deliberately naive wallet for testing the conformance suite: always 200, no signature check,
// no idempotency, no funds check, no session check. Never copy this.
import { createServer } from "node:http";

let balance = 100000;
const bets = new Map<string, number>();
createServer((req, res) => {
  let raw = "";
  req.on("data", (d) => (raw += d));
  req.on("end", () => {
    const b = JSON.parse(raw || "{}");
    const op = req.url?.split("/").pop();
    if (op === "bet") { balance -= b.amount; bets.set(b.transaction_id, b.amount); }
    if (op === "win") balance += b.amount;
    if (op === "rollback") balance += bets.get(b.bet_transaction_id) ?? 0;
    res.setHeader("Content-Type", "application/json");
    res.end(JSON.stringify({ player_id: "player-42", balance, currency: "EUR", precision: 2 }));
  });
}).listen(Number(process.env.PORT ?? 8082));
