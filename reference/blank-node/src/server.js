import express from "express";
import { players, playerForToken } from "./db.js";

const app = express();
app.use(express.json());

// Lobby: the player's own profile, looked up by session token.
app.get("/api/me", (req, res) => {
  const player = playerForToken(req.get("X-Session"));
  if (!player) return res.status(401).json({ error: "not logged in" });
  res.json(player);
});

// Back office: list players.
app.get("/admin/players", (_req, res) => res.json([...players.values()]));

const port = Number(process.env.PORT ?? 3000);
app.listen(port, () => console.log(`acme-casino on :${port}`));
