import { createHmac, timingSafeEqual } from "node:crypto";
import express from "express";

const MAX = Number.MAX_SAFE_INTEGER; // 2^53-1, spec "Money"

// ponytail: in-memory store, lost on restart and single process. Use a database with a unique
// transaction_id constraint and row locks (or serializable transactions) for anything real.
export type Store = {
  players: Map<string, { balance: number; currency: string; precision: number }>;
  sessions: Map<string, { player_id: string; expired: boolean }>;
  txs: Map<string, { fp: string; resp: Resp }>; // one namespace for bet, win, rollback
  bets: Map<string, { player_id: string; amount: number }>; // bets that debited money
  rolledBack: Set<string>; // bet ids rolled back, including bets never seen
};
type Resp = { status: number; body: any };
type Body = Record<string, any>;

export function seedStore(env: Record<string, string | undefined> = process.env): Store {
  const player_id = env.PLAYER_ID ?? "player-42";
  return {
    players: new Map([[player_id, {
      balance: Number(env.PLAYER_BALANCE ?? 100000),
      currency: env.PLAYER_CURRENCY ?? "EUR",
      precision: Number(env.PLAYER_PRECISION ?? 2),
    }]]),
    sessions: new Map([
      [env.SESSION_TOKEN ?? "sess_test", { player_id, expired: false }],
      // Optional: an already-expired session for the same player (used by the conformance suite).
      ...(env.EXPIRED_SESSION_TOKEN ? [[env.EXPIRED_SESSION_TOKEN, { player_id, expired: true }] as const] : []),
    ]),
    txs: new Map(), bets: new Map(), rolledBack: new Set(),
  };
}

const err = (status: number, code: string, message: string): Resp => ({ status, body: { error: { code, message } } });
const ok = (p: { balance: number; currency: string; precision: number }, extra = {}): Resp =>
  ({ status: 200, body: { ...extra, balance: p.balance, currency: p.currency, precision: p.precision } });
const invalidSession = () => err(403, "INVALID_SESSION", "Session token is unknown or expired, or not this player's; issue a new session token and launch again.");
const mismatch = () => err(409, "TRANSACTION_MISMATCH", "transaction_id was already used with a different body or call type; use a new transaction_id for a new operation, or resend the identical body.");
const rolledBackErr = () => err(409, "TRANSACTION_ROLLED_BACK", "A rollback already references this bet transaction_id; the bet was not applied and must not be retried.");
const wrongCurrency = (p: { currency: string; precision: number }) =>
  err(422, "CURRENCY_MISMATCH", `Session currency is ${p.currency} with precision ${p.precision}; send exactly those values.`);
const unknownPlayer = () => err(400, "INVALID_REQUEST", "player_id is unknown; send the player_id from /wallet/authenticate.");

// Field types from the spec's component schemas.
const checks = {
  id: (v: unknown) => typeof v === "string" && /^[A-Za-z0-9_.:-]{1,128}$/.test(v),
  currency: (v: unknown) => typeof v === "string" && /^[A-Z0-9_]{1,16}$/.test(v),
  precision: (v: unknown) => Number.isInteger(v) && (v as number) >= 0 && (v as number) <= 18,
  // ponytail: JSON.parse reads 100.0 and 1e2 as the integer 100, so those are accepted; inspect the
  // raw text if byte-level strictness is ever needed. Values above 2^53-1 lose exactness and are rejected.
  amount: (v: unknown) => Number.isSafeInteger(v) && (v as number) >= 0,
  positive: (v: unknown) => Number.isSafeInteger(v) && (v as number) >= 1,
};
type Kind = keyof typeof checks;

const ids = (...names: string[]) => Object.fromEntries(names.map((n) => [n, "id" as Kind]));
const money = { amount: "amount", currency: "currency", precision: "precision" } as Record<string, Kind>;
const base = ids("transaction_id", "round_id", "player_id", "game_id", "session_token");
const schemas: Record<string, Record<string, Kind>> = {
  authenticate: ids("session_token"),
  balance: ids("session_token", "player_id", "game_id"),
  bet: { ...base, ...money, amount: "positive" },
  win: { ...base, ...money },
  rollback: { ...base, bet_transaction_id: "id" },
};

function validate(schema: Record<string, Kind>, body: unknown): Resp | null {
  if (typeof body !== "object" || body === null || Array.isArray(body))
    return err(400, "INVALID_REQUEST", "Body must be a JSON object; send valid JSON as in the spec.");
  for (const [f, kind] of Object.entries(schema))
    if (!checks[kind]((body as Body)[f]))
      return err(400, "INVALID_REQUEST", `${f} is missing or not a valid ${kind}; fix it as the spec defines. Amounts are integers of minor units up to ${MAX}.`);
  return null;
}

// Spec "Signing": lowercase hex HMAC-SHA256 over the raw bytes, constant-time compare.
export function signatureOk(secret: string, raw: Buffer, header: unknown): boolean {
  if (typeof header !== "string" || !/^[0-9a-f]{64}$/.test(header)) return false;
  const want = createHmac("sha256", secret).update(raw).digest();
  return timingSafeEqual(want, Buffer.from(header, "hex"));
}

export function createApp(secret: string, store: Store = seedStore()) {
  const { players, sessions, txs, bets, rolledBack } = store;

  const session = (token: string, player_id?: string) => {
    const s = sessions.get(token);
    return s && !s.expired && (player_id === undefined || s.player_id === player_id) ? players.get(s.player_id) : undefined;
  };

  // Spec "Idempotency": same id + same fields replays the stored response; anything else is a mismatch.
  // Everything here is synchronous, so check-then-write cannot interleave with another request.
  function once(kind: string, b: Body, exec: () => Resp): Resp {
    const id = b.transaction_id;
    const fp = kind + JSON.stringify(Object.keys(schemas[kind]).map((f) => b[f]));
    const prior = txs.get(id);
    if (prior && prior.fp !== fp) return mismatch();
    if (kind === "bet" && rolledBack.has(id)) return rolledBackErr(); // late or replayed bet
    if (prior) return prior.resp;
    const resp = exec();
    if (resp.body.error?.code !== "INTERNAL_ERROR") txs.set(id, { fp, resp }); // INTERNAL_ERROR never stored
    return resp;
  }

  const handlers: Record<string, (b: Body) => Resp> = {
    authenticate: (b) => {
      const p = session(b.session_token);
      return p ? ok(p, { player_id: sessions.get(b.session_token)!.player_id }) : invalidSession();
    },
    balance: (b) => {
      const p = session(b.session_token, b.player_id);
      return p ? ok(p) : invalidSession();
    },
    bet: (b) => once("bet", b, () => {
      const p = session(b.session_token, b.player_id); // session must be valid now
      if (!p) return invalidSession();
      if (b.currency !== p.currency || b.precision !== p.precision) return wrongCurrency(p);
      if (p.balance < b.amount) return err(402, "INSUFFICIENT_FUNDS", "Balance is below the stake; deposit funds or lower the bet. Nothing was debited.");
      p.balance -= b.amount;
      bets.set(b.transaction_id, { player_id: b.player_id, amount: b.amount });
      return ok(p);
    }),
    win: (b) => once("win", b, () => {
      // Session token deliberately not checked: win must be accepted after expiry.
      const p = players.get(b.player_id);
      if (!p) return unknownPlayer();
      if (b.currency !== p.currency || b.precision !== p.precision) return wrongCurrency(p);
      if (p.balance + b.amount > MAX) return err(400, "INVALID_REQUEST", `Credit would push the balance above ${MAX}.`);
      p.balance += b.amount;
      return ok(p);
    }),
    rollback: (b) => once("rollback", b, () => {
      // Session token deliberately not checked: rollback must be accepted after expiry.
      const p = players.get(b.player_id);
      if (!p) return unknownPlayer();
      const bet = bets.get(b.bet_transaction_id);
      if (bet && bet.player_id !== b.player_id) return err(400, "INVALID_REQUEST", "player_id is not the bet's player; send the bet's player_id.");
      if (bet && !rolledBack.has(b.bet_transaction_id)) p.balance += bet.amount; // first rollback only
      rolledBack.add(b.bet_transaction_id); // also for an unknown bet, so a late bet is refused
      return ok(p);
    }),
  };

  const app = express();
  app.disable("x-powered-by");
  app.use(express.raw({ type: () => true, limit: "100kb" })); // raw bytes first; JSON is parsed by hand
  app.post("/wallet/:op", (req, res) => {
    const op = String(req.params.op);
    const raw: Buffer = Buffer.isBuffer(req.body) ? req.body : Buffer.alloc(0);
    let r: Resp;
    if (!Object.hasOwn(handlers, op)) r = err(400, "INVALID_REQUEST", "Unknown path; use /wallet/authenticate, balance, bet, win or rollback.");
    else if (!signatureOk(secret, raw, req.get("X-Signature")))
      r = err(401, "INVALID_SIGNATURE", "X-Signature does not match HMAC-SHA256 of the raw body; sign the exact bytes you send with the shared secret and encode as lowercase hex.");
    else {
      let body: unknown;
      try { body = JSON.parse(raw.toString("utf8")); } catch { body = undefined; }
      r = validate(schemas[op], body) ?? handlers[op](body as Body);
    }
    res.status(r.status).json(r.body);
  });
  app.use((_req, res) => res.status(400).json(err(400, "INVALID_REQUEST", "Use POST on a /wallet/* path.").body));
  app.use(((e: any, _req, res, _next) => {
    const client = e?.status >= 400 && e.status < 500;
    res.status(client ? 400 : 500).json(client
      ? err(400, "INVALID_REQUEST", "Request could not be read; send a JSON body under 100 kB.").body
      : err(500, "INTERNAL_ERROR", "Temporary failure; retry the same request with the same transaction_id.").body);
  }) as express.ErrorRequestHandler);
  return app;
}
