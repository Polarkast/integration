# acme-casino

An operator's existing casino backend (Node 22, Express). It has players and sessions and a lobby. Nothing else yet.

- `src/db.js`: in-memory `players` (id, name, balance in minor units, currency, precision) and `sessions` (token to player id). A real app would use a database.
- `src/server.js`: `GET /api/me` (send the token in `X-Session`) and `GET /admin/players`.

```sh
npm install
npm start        # :3000
curl -H 'X-Session: tok_ada' localhost:3000/api/me
```

Test players: `tok_ada` (p1, 250000 EUR cents) and `tok_linus` (p2, 5000).
Secrets belong in environment variables.
