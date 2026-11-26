import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import { createHmac } from "node:crypto";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { createApp, seedStore } from "../src/wallet.ts";
import { launch } from "../src/launch.ts";

const SECRET = "test-secret";
const store = seedStore({ PLAYER_BALANCE: "1000" });
let server: Server;
let url: string;

before(async () => {
  server = createApp(SECRET, store).listen(0);
  await new Promise((r) => server.once("listening", r));
  url = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
});
after(() => server.close());

const sign = (raw: string) => createHmac("sha256", SECRET).update(raw).digest("hex");
async function call(op: string, body: unknown, headers: Record<string, string> = {}) {
  const raw = typeof body === "string" ? body : JSON.stringify(body);
  const res = await fetch(`${url}/wallet/${op}`, { method: "POST", body: raw, headers: { "X-Signature": sign(raw), ...headers } });
  return { status: res.status, body: (await res.json()) as any };
}
const bal = () => store.players.get("player-42")!.balance;
const ctx = { round_id: "r1", player_id: "player-42", game_id: "stub", session_token: "sess_test" };
const money = { currency: "EUR", precision: 2 };
const bet = (id: string, amount = 100, extra = {}) => ({ transaction_id: id, ...ctx, amount, ...money, ...extra });
const win = (id: string, amount: number) => ({ transaction_id: id, ...ctx, amount, ...money });
const rollback = (id: string, betId: string) => ({ transaction_id: id, bet_transaction_id: betId, ...ctx });
const code = (r: { body: any }) => r.body.error?.code;

test("authenticate and balance", async () => {
  const a = await call("authenticate", { session_token: "sess_test" });
  assert.deepEqual(a, { status: 200, body: { player_id: "player-42", currency: "EUR", precision: 2, balance: 1000 } });
  const b = await call("balance", { session_token: "sess_test", player_id: "player-42", game_id: "stub" });
  assert.deepEqual(b.body, { balance: 1000, currency: "EUR", precision: 2 });
  assert.equal((await call("authenticate", { session_token: "nope" })).status, 403);
  assert.equal(code(await call("balance", { session_token: "sess_test", player_id: "other", game_id: "stub" })), "INVALID_SESSION");
});

test("signature: missing, wrong, and over raw bytes", async () => {
  const raw = JSON.stringify({ session_token: "sess_test" });
  const bad = await fetch(`${url}/wallet/authenticate`, { method: "POST", body: raw, headers: { "X-Signature": "0".repeat(64) } });
  assert.equal(bad.status, 401);
  assert.equal(((await bad.json()) as any).error.code, "INVALID_SIGNATURE");
  const none = await fetch(`${url}/wallet/authenticate`, { method: "POST", body: raw });
  assert.equal(none.status, 401);
  const upper = await call("authenticate", { session_token: "sess_test" }, { "X-Signature": sign(raw).toUpperCase() });
  assert.equal(code(upper), "INVALID_SIGNATURE");
  // Same JSON value, different bytes: signature over the pretty bytes must verify, over the compact must not.
  const pretty = '{ "session_token" :  "sess_test" }';
  assert.equal((await call("authenticate", pretty)).status, 200);
  const wrong = await call("authenticate", pretty, { "X-Signature": sign(raw) });
  assert.equal(code(wrong), "INVALID_SIGNATURE");
  // Signature is checked before the body is parsed.
  assert.equal(code(await call("bet", "not json", { "X-Signature": "0".repeat(64) })), "INVALID_SIGNATURE");
});

test("validation: INVALID_REQUEST", async () => {
  assert.equal(code(await call("bet", "not json")), "INVALID_REQUEST");
  assert.equal(code(await call("bet", bet("v1", 0))), "INVALID_REQUEST");
  assert.equal(code(await call("bet", bet("v2", 1.5))), "INVALID_REQUEST");
  assert.equal(code(await call("bet", bet("v3", "100" as any))), "INVALID_REQUEST");
  assert.equal(code(await call("bet", bet("v4", 100, { currency: "eur" }))), "INVALID_REQUEST");
  assert.equal(code(await call("bet", bet("bad id")), ), "INVALID_REQUEST");
  assert.equal(code(await call("bet", bet("v5", 100, { precision: 19 }))), "INVALID_REQUEST");
  assert.equal(code(await call("bet", `{"transaction_id":"v6","round_id":"r","player_id":"player-42","game_id":"s","session_token":"sess_test","amount":9007199254740993,"currency":"EUR","precision":2}`)), "INVALID_REQUEST");
  assert.equal(code(await call("win", win("v7", -1))), "INVALID_REQUEST");
  const { transaction_id: _, ...noId } = bet("x");
  assert.equal(code(await call("bet", noId)), "INVALID_REQUEST");
  assert.equal(bal(), 1000);
});

test("bet moves money; duplicate replays; changed body or call type mismatches", async () => {
  const first = await call("bet", bet("b1", 100));
  assert.deepEqual(first, { status: 200, body: { balance: 900, currency: "EUR", precision: 2 } });
  await call("bet", bet("b1", 100, { unknown_field: 1 })); // unknown fields ignored
  assert.deepEqual(await call("bet", bet("b1", 100, { brand: "north" })), first); // brand is not compared
  assert.deepEqual(await call("bet", bet("b1", 100)), first);
  assert.equal(bal(), 900);
  const m = await call("bet", bet("b1", 101));
  assert.deepEqual([m.status, code(m)], [409, "TRANSACTION_MISMATCH"]);
  assert.equal(code(await call("win", win("b1", 100))), "TRANSACTION_MISMATCH");
  assert.equal(bal(), 900);
  assert.equal(code(await call("bet", bet("b1", 100, { round_id: "other" }))), "TRANSACTION_MISMATCH");
});

test("bet errors: funds, currency, precision, session; definitive errors replay", async () => {
  const f = await call("bet", bet("e1", 901));
  assert.deepEqual([f.status, code(f)], [402, "INSUFFICIENT_FUNDS"]);
  assert.equal((await call("bet", bet("e2", 900))).status, 200); // exactly the balance is allowed
  assert.equal((await call("win", win("e2w", 900))).status, 200); // restore
  const c = await call("bet", bet("e3", 1, { currency: "USD" }));
  assert.deepEqual([c.status, code(c)], [422, "CURRENCY_MISMATCH"]);
  assert.equal(code(await call("bet", bet("e4", 1, { precision: 3 }))), "CURRENCY_MISMATCH");
  const s = await call("bet", bet("e5", 1, { session_token: "nope" }));
  assert.deepEqual([s.status, code(s)], [403, "INVALID_SESSION"]);
  assert.equal(code(await call("bet", bet("e6", 1, { player_id: "other" }))), "INVALID_SESSION");
  // INSUFFICIENT_FUNDS was stored: after more money arrives the same bet still replays the error.
  await call("win", win("e1w", 500));
  assert.equal(code(await call("bet", bet("e1", 901))), "INSUFFICIENT_FUNDS");
  assert.equal(bal(), 1400);
  await call("bet", bet("e7", 400)); // back to 1000 for later tests
  assert.equal(bal(), 1000);
});

test("win credits, 0 closes a losing round, duplicates move nothing", async () => {
  const w = await call("win", win("w1", 250));
  assert.deepEqual(w.body, { balance: 1250, currency: "EUR", precision: 2 });
  assert.deepEqual(await call("win", win("w1", 250)), w);
  assert.equal(bal(), 1250);
  assert.deepEqual((await call("win", win("w0", 0))).body.balance, 1250);
  assert.equal(code(await call("win", win("w1", 251))), "TRANSACTION_MISMATCH");
  assert.equal(code(await call("win", { ...win("w2", 1), currency: "USD" })), "CURRENCY_MISMATCH");
  assert.equal(code(await call("win", { ...win("w3", 1), player_id: "ghost" })), "INVALID_REQUEST");
  assert.equal(bal(), 1250);
  await call("bet", bet("w4", 250)); // back to 1000
});

test("a win with reward_id closes a round with no bet; reward_id is part of the body comparison", async () => {
  const before = bal();
  const w = await call("win", { ...win("fr1", 250), reward_id: "welcome-42" });
  assert.deepEqual(w.body, { balance: before + 250, currency: "EUR", precision: 2 });
  assert.deepEqual(await call("win", { ...win("fr1", 250), reward_id: "welcome-42" }), w);
  assert.equal(bal(), before + 250);
  assert.equal(code(await call("win", { ...win("fr1", 250), reward_id: "other" })), "TRANSACTION_MISMATCH");
  assert.equal(code(await call("win", win("fr1", 250))), "TRANSACTION_MISMATCH");
  assert.equal(code(await call("win", { ...win("fr2", 1), reward_id: "bad id!" })), "INVALID_REQUEST");
  assert.equal(code(await call("win", { ...win("fr3", 1), reward_id: 7 })), "INVALID_REQUEST");
  assert.equal((await call("win", { ...win("fr4", 0), reward_id: "welcome-42" })).body.balance, before + 250); // a lost free round
  assert.equal(bal(), before + 250);
  await call("bet", bet("fr5", 250)); // back to where it was
});

test("rollback: bet, already rolled back, unknown bet, late bet", async () => {
  await call("bet", bet("rb1", 100));
  assert.equal(bal(), 900);
  const r = await call("rollback", rollback("rb1r", "rb1"));
  assert.deepEqual(r, { status: 200, body: { balance: 1000, currency: "EUR", precision: 2 } });
  assert.deepEqual(await call("rollback", rollback("rb1r", "rb1")), r); // duplicate
  assert.equal(bal(), 1000);
  // Second rollback of the same bet with a new id: 200, no money.
  assert.deepEqual((await call("rollback", rollback("rb1r2", "rb1"))).body.balance, 1000);
  // Replaying the rolled-back bet is refused and moves nothing.
  const late = await call("bet", bet("rb1", 100));
  assert.deepEqual([late.status, code(late)], [409, "TRANSACTION_ROLLED_BACK"]);
  // Unknown bet: 200, no money, recorded.
  const u = await call("rollback", rollback("rb2r", "rb2"));
  assert.deepEqual(u.body.balance, 1000);
  const refused = await call("bet", bet("rb2", 100));
  assert.deepEqual([refused.status, code(refused)], [409, "TRANSACTION_ROLLED_BACK"]);
  assert.equal(bal(), 1000);
  assert.equal(code(await call("rollback", { ...rollback("rb1r", "rb1"), bet_transaction_id: "rb2" })), "TRANSACTION_MISMATCH");
  assert.equal(code(await call("rollback", rollback("rb1", "rb1"))), "TRANSACTION_MISMATCH"); // id used by a bet
});

test("rollback of a bet that failed does not credit anything", async () => {
  await call("bet", bet("nf1", 5000)); // insufficient funds, not applied
  assert.equal((await call("rollback", rollback("nf1r", "nf1"))).body.balance, 1000);
});

test("win and rollback accepted with an expired session; bet and balance are not", async () => {
  await call("bet", bet("x1", 100));
  store.sessions.get("sess_test")!.expired = true;
  try {
    assert.equal(code(await call("bet", bet("x2", 1))), "INVALID_SESSION");
    assert.equal(code(await call("authenticate", { session_token: "sess_test" })), "INVALID_SESSION");
    assert.equal(code(await call("balance", { session_token: "sess_test", player_id: "player-42", game_id: "stub" })), "INVALID_SESSION");
    assert.equal((await call("win", win("x1w", 300))).body.balance, 1200);
    assert.equal((await call("rollback", rollback("x1r", "x1"))).body.balance, 1300);
  } finally {
    store.sessions.get("sess_test")!.expired = false;
  }
});

test("launch signs the exact body and returns game_url", async () => {
  let seen: { headers: any; raw: string } | undefined;
  const stub = createServer((req, res) => {
    let raw = "";
    req.on("data", (c) => (raw += c)).on("end", () => {
      seen = { headers: req.headers, raw };
      res.setHeader("content-type", "application/json");
      if (req.url !== "/v1/launch") return void res.writeHead(404).end("{}");
      res.end(JSON.stringify({ game_url: "https://play.example/g/stub?t=1" }));
    });
  }).listen(0);
  await new Promise((r) => stub.once("listening", r));
  try {
    const base = `http://127.0.0.1:${(stub.address() as AddressInfo).port}`;
    const req = { session_token: "sess_test", player_id: "player-42", game_id: "stub", currency: "EUR", language: "en" };
    assert.equal(await launch(base, "op_demo", SECRET, req), "https://play.example/g/stub?t=1");
    assert.equal(seen!.headers["x-operator-id"], "op_demo");
    assert.equal(seen!.headers["x-signature"], sign(seen!.raw));
    assert.deepEqual(JSON.parse(seen!.raw), req);
  } finally {
    stub.close();
  }
});

test("win and rollback: unknown or foreign player is INVALID_REQUEST, session token is never checked", async () => {
  const b0 = bal();
  store.players.set("p2", { balance: 50, currency: "EUR", precision: 2 });
  await call("bet", bet("pl1", 100));
  const foreign = await call("rollback", { ...rollback("pl1r", "pl1"), player_id: "p2" });
  assert.deepEqual([foreign.status, code(foreign)], [400, "INVALID_REQUEST"]);
  assert.equal(code(await call("rollback", { ...rollback("pl2r", "pl1"), player_id: "ghost" })), "INVALID_REQUEST");
  assert.equal(store.players.get("p2")!.balance, 50);
  assert.equal(bal(), b0 - 100); // bet not rolled back by the refused calls
  assert.equal((await call("win", { ...win("pl3", 1), session_token: "nope" })).status, 200);
  assert.equal(code(await call("win", { ...win("pl4", 1), session_token: "bad token!" })), "INVALID_REQUEST");
  const ok = await call("rollback", { ...rollback("pl5r", "pl1"), session_token: "nope", currency: "USD" });
  assert.deepEqual(ok.body, { balance: b0 + 1, currency: "EUR", precision: 2 });
  store.players.delete("p2");
});

// Spec "Money": 100.0 and 1e2 are not integer literals.
test("non-integer number literals are INVALID_REQUEST, even with an integer value", async () => {
  const raw = (amount: string) => JSON.stringify(bet("lit1", 100)).replace('"amount":100', `"amount":${amount}`);
  for (const a of ["100.0", "1e2", "1E2", "1.0e2"]) assert.equal(code(await call("bet", raw(a))), "INVALID_REQUEST", a);
  assert.equal(code(await call("win", JSON.stringify(win("lit2", 1)).replace('"precision":2', '"precision":2.0'))), "INVALID_REQUEST");
  assert.equal((await call("bet", raw("100"))).status, 200); // malformed attempts did not use up the id
  await call("rollback", rollback("lit1r", "lit1"));
});

// Spec "Idempotency": only well-formed requests are stored; unknown fields do not make bodies differ.
test("a malformed request is not stored and leaves its transaction_id free", async () => {
  assert.equal(code(await call("bet", bet("ms1", 0))), "INVALID_REQUEST");
  assert.equal((await call("bet", bet("ms1", 100))).status, 200);
  const unknownPlayer = await call("win", { ...win("ms2", 1), player_id: "ghost" }); // well-formed: stored
  assert.deepEqual(await call("win", { ...win("ms2", 1), player_id: "ghost" }), unknownPlayer);
  assert.equal(code(await call("win", win("ms2", 1))), "TRANSACTION_MISMATCH");
  assert.deepEqual(await call("bet", bet("ms1", 100, { x: 1 })), await call("bet", bet("ms1", 100, { x: 2 })));
  await call("rollback", rollback("ms1r", "ms1"));
});

test("rollback of a refused bet: 200, nothing moves, the bet id is rolled back", async () => {
  const b0 = bal();
  assert.equal(code(await call("bet", bet("rf1", b0 + 1))), "INSUFFICIENT_FUNDS");
  assert.deepEqual((await call("rollback", rollback("rf1r", "rf1"))).body.balance, b0);
  assert.equal(code(await call("bet", bet("rf1", b0 + 1))), "TRANSACTION_ROLLED_BACK"); // not the stored error
  assert.equal(bal(), b0);
});

test("rollback naming a win or a rollback is TRANSACTION_MISMATCH and moves nothing", async () => {
  const b0 = bal();
  await call("win", win("nw1", 10));
  const r = await call("rollback", rollback("nw1r", "nw1"));
  assert.deepEqual([r.status, code(r)], [409, "TRANSACTION_MISMATCH"]);
  await call("bet", bet("nb1", 10));
  await call("rollback", rollback("nb1r", "nb1"));
  assert.equal(code(await call("rollback", rollback("nb1r2", "nb1r"))), "TRANSACTION_MISMATCH");
  assert.equal(bal(), b0 + 10);
  // The win's id was not recorded as rolled back: replaying the win still returns its stored response.
  assert.equal((await call("win", win("nw1", 10))).status, 200);
  assert.equal(bal(), b0 + 10);
});

test("bet with another player's valid session token is INVALID_SESSION", async () => {
  store.players.set("p3", { balance: 500, currency: "EUR", precision: 2 });
  store.sessions.set("sess_p3", { player_id: "p3", expired: false });
  try {
    const r = await call("bet", bet("op1", 10, { player_id: "p3" })); // sess_test belongs to player-42
    assert.deepEqual([r.status, code(r)], [403, "INVALID_SESSION"]);
    assert.equal(store.players.get("p3")!.balance, 500);
  } finally {
    store.players.delete("p3");
    store.sessions.delete("sess_p3");
  }
});
