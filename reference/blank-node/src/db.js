// In-memory data for the demo casino. Balances are integers in minor units.
export const players = new Map([
  ["p1", { id: "p1", name: "Ada", balance: 250000, currency: "EUR", precision: 2 }],
  ["p2", { id: "p2", name: "Linus", balance: 5000, currency: "EUR", precision: 2 }],
]);

// session token -> player id
export const sessions = new Map([
  ["tok_ada", "p1"],
  ["tok_linus", "p2"],
]);

export function playerForToken(token) {
  return players.get(sessions.get(token));
}
