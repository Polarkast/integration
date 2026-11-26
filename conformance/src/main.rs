//! Plays the studio's role against an operator wallet and checks it against docs/openapi.yaml.
//! Every check cites the spec rule it tests in its `fix` text.

use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

const USAGE: &str = "usage: conformance --wallet-url <url> --session-token <token> [--expired-session-token <token>]\n  the shared secret is read from POLARKAST_SHARED_SECRET (or --secret, which shows it in the process list)";

struct Fail {
    req: String,
    expected: String,
    actual: String,
    fix: String,
}
type R<T = ()> = Result<T, Fail>;

struct Resp {
    req: String,
    status: u16,
    text: String,
    body: Value,
}

enum Sig {
    Good,
    Bad,
    Missing,
}

impl Resp {
    fn fail(&self, expected: impl Into<String>, fix: &str) -> Fail {
        Fail {
            req: self.req.clone(),
            expected: expected.into(),
            actual: format!("{} {}", self.status, self.text),
            fix: fix.into(),
        }
    }
    fn code(&self) -> &str {
        self.body["error"]["code"].as_str().unwrap_or("")
    }
    /// Expect an error body with this HTTP status and code.
    fn err(&self, status: u16, code: &str, fix: &str) -> R {
        if self.status == status && self.code() == code {
            return Ok(());
        }
        Err(self.fail(format!("HTTP {status} with error code {code}"), fix))
    }
    /// Expect HTTP 200 with an integer balance; returns it.
    fn ok(&self, cur: &str, prec: i64, fix: &str) -> R<i64> {
        let b = &self.body;
        match (self.status, b["balance"].as_i64()) {
            (200, Some(bal)) if b["currency"] == cur && b["precision"] == prec => Ok(bal),
            _ => Err(self.fail(
                format!("HTTP 200 with integer balance, currency {cur}, precision {prec}"),
                fix,
            )),
        }
    }
}

struct Ctx {
    http: reqwest::blocking::Client,
    url: String,
    secret: String,
    token: String,
    run: String,
    n: Cell<u32>,
    player: String,
    cur: String,
    prec: i64,
    passed: u32,
    failed: u32,
    skipped: u32,
}

impl Ctx {
    fn id(&self) -> String {
        self.n.set(self.n.get() + 1);
        format!("{}-{}", self.run, self.n.get())
    }

    fn send(&self, path: &str, body: &Value, sig: Sig) -> Resp {
        self.send_raw(path, body.to_string(), sig)
    }
    /// Sends exactly these bytes, signed over them.
    fn send_raw(&self, path: &str, raw: String, sig: Sig) -> Resp {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).unwrap();
        mac.update(raw.as_bytes());
        let good: String = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let mut rq = self
            .http
            .post(format!("{}{}", self.url, path))
            .header("Content-Type", "application/json")
            .body(raw.clone());
        match sig {
            Sig::Good => rq = rq.header("X-Signature", good),
            Sig::Bad => rq = rq.header("X-Signature", "0".repeat(64)),
            Sig::Missing => {}
        }
        let req = format!("POST {path} {raw}");
        match rq.send().and_then(|r| Ok((r.status().as_u16(), r.text()?))) {
            Ok((status, text)) => {
                let body = serde_json::from_str(&text).unwrap_or(Value::Null);
                Resp {
                    req,
                    status,
                    text,
                    body,
                }
            }
            Err(e) => Resp {
                req,
                status: 0,
                text: format!("no response: {e}"),
                body: Value::Null,
            },
        }
    }
    fn post(&self, path: &str, body: &Value) -> Resp {
        self.send(path, body, Sig::Good)
    }

    fn bet(&self, id: &str, amount: i64) -> Value {
        json!({"transaction_id": id, "round_id": self.id(), "player_id": self.player, "game_id": "conformance",
               "session_token": self.token, "amount": amount, "currency": self.cur, "precision": self.prec})
    }
    fn win(&self, id: &str, amount: i64) -> Value {
        self.bet(id, amount)
    }
    fn rollback(&self, id: &str, bet_id: &str) -> Value {
        json!({"transaction_id": id, "bet_transaction_id": bet_id, "round_id": self.id(), "player_id": self.player,
               "game_id": "conformance", "session_token": self.token})
    }

    fn balance_req(&self) -> Resp {
        let b = json!({"session_token": self.token, "player_id": self.player, "game_id": "conformance"});
        self.post("/wallet/balance", &b)
    }
    fn bal(&self) -> R<i64> {
        self.balance_req().ok(
            &self.cur,
            self.prec,
            "Balance: /wallet/balance returns the balance in the session's currency and precision.",
        )
    }
    /// Balance read back through /wallet/balance must equal `want`: proves money did (not) move.
    fn bal_is(&self, want: i64, fix: &str) -> R {
        let r = self.balance_req();
        let got = r.ok(&self.cur, self.prec, fix)?;
        if got == want {
            Ok(())
        } else {
            Err(r.fail(format!("balance {want} (as read back)"), fix))
        }
    }
    /// A money call that must succeed with `want` as the returned balance.
    fn money_ok(&self, path: &str, body: &Value, want: i64, fix: &str) -> R {
        let r = self.post(path, body);
        if r.ok(&self.cur, self.prec, fix)? == want {
            Ok(())
        } else {
            Err(r.fail(format!("HTTP 200 with balance {want}"), fix))
        }
    }
    /// Place a bet that must succeed; returns its transaction id.
    fn placed_bet(&self, amount: i64) -> R<String> {
        let id = self.id();
        let b0 = self.bal()?;
        self.money_ok(
            "/wallet/bet",
            &self.bet(&id, amount),
            b0 - amount,
            "Bet: debits `amount` and returns the new balance.",
        )?;
        Ok(id)
    }

    fn run(&mut self, name: &str, f: impl FnOnce(&Ctx) -> R) {
        match f(self) {
            Ok(()) => {
                self.passed += 1;
                println!("PASS {name}");
            }
            Err(e) => {
                self.failed += 1;
                println!(
                    "FAIL {name}\n  request:  {}\n  expected: {}\n  actual:   {}\n  fix:      {}",
                    e.req, e.expected, e.actual, e.fix
                );
            }
        }
    }
    fn skip(&mut self, name: &str) {
        self.skipped += 1;
        println!("SKIP {name} (NOT VERIFIED: no --expired-session-token)");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return;
    }
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let (Some(url), Some(secret), Some(token)) = (
        flag("--wallet-url"),
        flag("--secret").or_else(|| std::env::var("POLARKAST_SHARED_SECRET").ok()),
        flag("--session-token"),
    ) else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };
    let expired = flag("--expired-session-token");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut c = Ctx {
        http: reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap(),
        url: url.trim_end_matches('/').into(),
        secret,
        token,
        run: format!("conf-{nanos:x}"),
        n: Cell::new(0),
        player: String::new(),
        cur: String::new(),
        prec: 0,
        passed: 0,
        failed: 0,
        skipped: 0,
    };

    // Authenticate fixes player, currency and precision for every later check; without it nothing else can run.
    let auth = c.post("/wallet/authenticate", &json!({"session_token": c.token}));
    let b = &auth.body;
    let fix = "Authenticate: returns player_id, currency, precision and balance (integer minor units, at least 1000 for this suite) for a valid session token.";
    match (
        b["player_id"].as_str(),
        b["currency"].as_str(),
        b["precision"].as_i64(),
        b["balance"].as_i64(),
    ) {
        (Some(p), Some(cur), Some(prec), Some(bal)) if auth.status == 200 && bal >= 1000 => {
            c.player = p.into();
            c.cur = cur.into();
            c.prec = prec;
            c.passed += 1;
            println!("PASS authenticate returns player, currency, precision, balance");
        }
        _ => {
            let e = auth.fail(
                "HTTP 200 with player_id, currency, precision, balance >= 1000",
                fix,
            );
            println!(
                "FAIL authenticate returns player, currency, precision, balance\n  request:  {}\n  expected: {}\n  actual:   {}\n  fix:      {}",
                e.req, e.expected, e.actual, e.fix
            );
            println!("\nstopped: cannot continue without a valid session; 0 passed, 1 failed");
            std::process::exit(1);
        }
    }

    c.run("authenticate rejects an unknown session token", |c| {
        c.post(
            "/wallet/authenticate",
            &json!({"session_token": format!("{}-unknown", c.run)}),
        )
        .err(
            403,
            "INVALID_SESSION",
            "Errors: an unknown or expired session token is INVALID_SESSION, HTTP 403.",
        )
    });
    c.run("balance", |c| c.bal().map(|_| ()));
    c.run("bet debits the stake", |c| c.placed_bet(100).map(|_| ()));
    c.run("duplicate bet returns the original response and moves no money", |c| {
        let fix = "Idempotency: the same transaction_id with the same body returns the original response (same status and body) and moves no money.";
        let b0 = c.bal()?;
        let body = c.bet(&c.id(), 100);
        let first = c.post("/wallet/bet", &body);
        first.ok(&c.cur, c.prec, fix)?;
        let again = c.post("/wallet/bet", &body);
        if again.status != first.status || again.body != first.body {
            return Err(again.fail(format!("the original response: {} {}", first.status, first.text), fix));
        }
        c.bal_is(b0 - 100, fix)
    });
    c.run("same transaction id with a different body is TRANSACTION_MISMATCH", |c| {
        let fix = "Idempotency: the same transaction_id with any field different returns TRANSACTION_MISMATCH (HTTP 409) and moves no money.";
        let id = c.id();
        let b0 = c.bal()?;
        c.money_ok("/wallet/bet", &c.bet(&id, 100), b0 - 100, fix)?;
        c.post("/wallet/bet", &c.bet(&id, 101)).err(409, "TRANSACTION_MISMATCH", fix)?;
        c.bal_is(b0 - 100, fix)
    });
    c.run("transaction id reused by another call type is TRANSACTION_MISMATCH", |c| {
        let fix = "Idempotency: one transaction_id namespace across bet, win and rollback; reuse by a different call type returns TRANSACTION_MISMATCH (HTTP 409).";
        let id = c.placed_bet(100)?;
        let b1 = c.bal()?;
        c.post("/wallet/win", &c.win(&id, 100)).err(409, "TRANSACTION_MISMATCH", fix)?;
        c.bal_is(b1, fix)
    });
    c.run("win credits the payout", |c| {
        let b0 = c.bal()?;
        c.money_ok(
            "/wallet/win",
            &c.win(&c.id(), 250),
            b0 + 250,
            "Win: credits `amount` and returns the new balance.",
        )
    });
    c.run("duplicate win returns the original response and moves no money", |c| {
        let fix = "Idempotency: the same transaction_id with the same body returns the original response and moves no money.";
        let b0 = c.bal()?;
        let body = c.win(&c.id(), 250);
        let first = c.post("/wallet/win", &body);
        first.ok(&c.cur, c.prec, fix)?;
        let again = c.post("/wallet/win", &body);
        if again.status != first.status || again.body != first.body {
            return Err(again.fail(format!("the original response: {} {}", first.status, first.text), fix));
        }
        c.bal_is(b0 + 250, fix)
    });
    c.run("win with amount 0 closes a round and moves no money", |c| {
        let b0 = c.bal()?;
        c.money_ok(
            "/wallet/win",
            &c.win(&c.id(), 0),
            b0,
            "Win: an `amount` of 0 closes a losing round, is HTTP 200 and moves no money.",
        )
    });
    c.run("rollback of a known bet credits the stake back", |c| {
        let b0 = c.bal()?;
        let bet = c.placed_bet(100)?;
        c.money_ok("/wallet/rollback", &c.rollback(&c.id(), &bet), b0, "Rollback: a bet that exists and is not yet rolled back has its stake credited back in full; the response balance is the balance after the call.")?;
        c.bal_is(b0, "Rollback: the stake is credited back in full.")
    });
    c.run("duplicate rollback returns the original response and moves no money", |c| {
        let fix = "Idempotency: the same transaction_id with the same body returns the original response and moves no money.";
        let bet = c.placed_bet(100)?;
        let b0 = c.bal()? + 100;
        let body = c.rollback(&c.id(), &bet);
        let first = c.post("/wallet/rollback", &body);
        first.ok(&c.cur, c.prec, fix)?;
        let again = c.post("/wallet/rollback", &body);
        if again.status != first.status || again.body != first.body {
            return Err(again.fail(format!("the original response: {} {}", first.status, first.text), fix));
        }
        c.bal_is(b0, fix)
    });
    c.run("rollback of an already rolled-back bet moves no money", |c| {
        let fix = "Rollback: a bet already rolled back by another rollback is answered HTTP 200 and no money moves.";
        let b0 = c.bal()?;
        let bet = c.placed_bet(100)?;
        c.money_ok("/wallet/rollback", &c.rollback(&c.id(), &bet), b0, fix)?;
        c.money_ok("/wallet/rollback", &c.rollback(&c.id(), &bet), b0, fix)?;
        c.bal_is(b0, fix)
    });
    c.run("rollback of an unknown bet succeeds and moves no money", |c| {
        let fix = "Rollback: a bet the operator has never seen is answered HTTP 200 and no money moves.";
        let b0 = c.bal()?;
        c.money_ok("/wallet/rollback", &c.rollback(&c.id(), &c.id()), b0, fix)?;
        c.bal_is(b0, fix)
    });
    c.run("late bet after its rollback is TRANSACTION_ROLLED_BACK", |c| {
        let fix = "Idempotency: a bet arriving after a rollback referencing its transaction_id is rejected with TRANSACTION_ROLLED_BACK (HTTP 409) and moves no money.";
        let b0 = c.bal()?;
        let bet = c.id();
        c.money_ok("/wallet/rollback", &c.rollback(&c.id(), &bet), b0, "Rollback: an unknown bet is answered HTTP 200 and no money moves.")?;
        c.post("/wallet/bet", &c.bet(&bet, 100)).err(409, "TRANSACTION_ROLLED_BACK", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("replayed bet after it was rolled back is TRANSACTION_ROLLED_BACK", |c| {
        let fix = "Idempotency: a bet that was rolled back stays rolled back; replaying it returns TRANSACTION_ROLLED_BACK (HTTP 409) and moves no money.";
        let b0 = c.bal()?;
        let id = c.id();
        let body = c.bet(&id, 100);
        c.money_ok("/wallet/bet", &body, b0 - 100, "Bet: debits `amount`.")?;
        c.money_ok("/wallet/rollback", &c.rollback(&c.id(), &id), b0, "Rollback: credits the stake back.")?;
        c.post("/wallet/bet", &body).err(409, "TRANSACTION_ROLLED_BACK", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("insufficient funds is rejected and debits nothing", |c| {
        let fix = "Bet: when the balance is below `amount` the error is INSUFFICIENT_FUNDS (HTTP 402) and nothing is debited.";
        let b0 = c.bal()?;
        c.post("/wallet/bet", &c.bet(&c.id(), b0 + 1)).err(402, "INSUFFICIENT_FUNDS", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("bad signature is rejected", |c| {
        let fix = "Signing: a request whose X-Signature does not match the HMAC-SHA256 of the raw body is rejected with INVALID_SIGNATURE (HTTP 401) before anything else happens.";
        let b0 = c.bal()?;
        c.send("/wallet/bet", &c.bet(&c.id(), 100), Sig::Bad).err(401, "INVALID_SIGNATURE", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("missing signature is rejected", |c| {
        let fix = "Signing: every request carries X-Signature; a request without it is rejected with INVALID_SIGNATURE (HTTP 401).";
        let b0 = c.bal()?;
        c.send("/wallet/bet", &c.bet(&c.id(), 100), Sig::Missing).err(401, "INVALID_SIGNATURE", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("bet with a different currency is CURRENCY_MISMATCH", |c| {
        let fix = "Money: every money call must repeat the session's currency exactly, otherwise CURRENCY_MISMATCH (HTTP 422); nothing moves.";
        let b0 = c.bal()?;
        let mut body = c.bet(&c.id(), 100);
        body["currency"] = json!(if c.cur == "ZZZ" { "YYY" } else { "ZZZ" });
        c.post("/wallet/bet", &body).err(422, "CURRENCY_MISMATCH", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("win with a different currency is CURRENCY_MISMATCH", |c| {
        let fix = "Money: every money call must repeat the session's currency exactly, otherwise CURRENCY_MISMATCH (HTTP 422); nothing moves.";
        let b0 = c.bal()?;
        let mut body = c.win(&c.id(), 100);
        body["currency"] = json!(if c.cur == "ZZZ" { "YYY" } else { "ZZZ" });
        c.post("/wallet/win", &body).err(422, "CURRENCY_MISMATCH", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("bet with a different precision is CURRENCY_MISMATCH", |c| {
        let fix = "Money: every money call must repeat the session's precision exactly, otherwise CURRENCY_MISMATCH (HTTP 422); nothing moves.";
        let b0 = c.bal()?;
        let mut body = c.bet(&c.id(), 100);
        body["precision"] = json!(if c.prec == 3 { 4 } else { 3 });
        c.post("/wallet/bet", &body).err(422, "CURRENCY_MISMATCH", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("bet with amount 0 is INVALID_REQUEST", |c| {
        let fix = "Bet: `amount` must be greater than 0; out-of-range values are INVALID_REQUEST (HTTP 400).";
        let b0 = c.bal()?;
        c.post("/wallet/bet", &c.bet(&c.id(), 0)).err(400, "INVALID_REQUEST", fix)?;
        c.bal_is(b0, fix)
    });

    c.run("bet with an unknown session token is INVALID_SESSION and the error is replayed", |c| {
        let fix = "Errors: a bet needs a valid session token (INVALID_SESSION, HTTP 403); Idempotency: every definitive error is stored and replayed with the same status and body.";
        let b0 = c.bal()?;
        let mut body = c.bet(&c.id(), 100);
        body["session_token"] = json!(format!("{}-unknown", c.run));
        let first = c.post("/wallet/bet", &body);
        first.err(403, "INVALID_SESSION", fix)?;
        let again = c.post("/wallet/bet", &body);
        if again.status != first.status || again.body != first.body {
            return Err(again.fail(format!("the original response: {} {}", first.status, first.text), fix));
        }
        c.bal_is(b0, fix)
    });
    c.run("insufficient funds error is replayed after the balance changed", |c| {
        let fix = "Idempotency: every definitive error (here INSUFFICIENT_FUNDS) is stored and replayed unchanged, even if the balance changed; only INTERNAL_ERROR is not stored.";
        let b0 = c.bal()?;
        let body = c.bet(&c.id(), b0 + 1);
        c.post("/wallet/bet", &body).err(402, "INSUFFICIENT_FUNDS", fix)?;
        c.money_ok("/wallet/win", &c.win(&c.id(), 10), b0 + 10, "Win: credits `amount`.")?;
        c.post("/wallet/bet", &body).err(402, "INSUFFICIENT_FUNDS", fix)?;
        c.bal_is(b0 + 10, fix)
    });
    c.run("win with an unknown player_id is INVALID_REQUEST", |c| {
        let fix = "Win and rollback checks: an unknown player_id on win is INVALID_REQUEST (HTTP 400); nothing moves.";
        let b0 = c.bal()?;
        let mut body = c.win(&c.id(), 100);
        body["player_id"] = json!(format!("{}-nobody", c.run));
        c.post("/wallet/win", &body).err(400, "INVALID_REQUEST", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("rollback with an unknown player_id is INVALID_REQUEST", |c| {
        let fix = "Win and rollback checks: an unknown player_id on rollback is INVALID_REQUEST (HTTP 400); nothing moves.";
        let b0 = c.bal()?;
        let bet = c.placed_bet(100)?;
        let mut body = c.rollback(&c.id(), &bet);
        body["player_id"] = json!(format!("{}-nobody", c.run));
        c.post("/wallet/rollback", &body).err(400, "INVALID_REQUEST", fix)?;
        c.bal_is(b0 - 100, fix)
    });
    c.run("win and rollback accept an unknown session token", |c| {
        let fix = "Win and rollback checks: the session token is never checked for validity, so an unknown token is accepted (HTTP 200) on win and rollback; INVALID_SESSION is never returned for them.";
        let b0 = c.bal()?;
        let unknown = json!(format!("{}-unknown", c.run));
        let mut w = c.win(&c.id(), 1);
        w["session_token"] = unknown.clone();
        c.money_ok("/wallet/win", &w, b0 + 1, fix)?;
        let bet = c.placed_bet(100)?;
        let mut rb = c.rollback(&c.id(), &bet);
        rb["session_token"] = unknown;
        c.money_ok("/wallet/rollback", &rb, b0 + 1, fix)
    });
    c.run("win and rollback with a malformed session token are INVALID_REQUEST", |c| {
        let fix = "Win and rollback checks: the session token must still be a well-formed Id (1 to 128 characters from A-Za-z0-9_-.:); otherwise INVALID_REQUEST (HTTP 400).";
        let b0 = c.bal()?;
        let mut w = c.win(&c.id(), 1);
        w["session_token"] = json!("not valid!");
        c.post("/wallet/win", &w).err(400, "INVALID_REQUEST", fix)?;
        let mut rb = c.rollback(&c.id(), &c.id());
        rb["session_token"] = json!("not valid!");
        c.post("/wallet/rollback", &rb).err(400, "INVALID_REQUEST", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("rollback of a refused bet succeeds and the bet id counts as rolled back", |c| {
        let fix = "Idempotency: a rollback of a bet that was refused (never debited, e.g. INSUFFICIENT_FUNDS) is answered HTTP 200 and no money moves; the bet id is recorded as rolled back, so replaying that bet returns TRANSACTION_ROLLED_BACK (HTTP 409), not its stored error.";
        let b0 = c.bal()?;
        let id = c.id();
        let body = c.bet(&id, b0 + 1);
        c.post("/wallet/bet", &body).err(402, "INSUFFICIENT_FUNDS", "Bet: when the balance is below `amount` the error is INSUFFICIENT_FUNDS (HTTP 402).")?;
        c.money_ok("/wallet/rollback", &c.rollback(&c.id(), &id), b0, fix)?;
        c.post("/wallet/bet", &body).err(409, "TRANSACTION_ROLLED_BACK", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("rollback naming a win is TRANSACTION_MISMATCH", |c| {
        let fix = "Idempotency: a rollback whose bet_transaction_id is the id of a win is TRANSACTION_MISMATCH (HTTP 409): the id is already used by another call type. Nothing moves and the id is not recorded as rolled back.";
        let w = c.id();
        let b0 = c.bal()?;
        c.money_ok("/wallet/win", &c.win(&w, 10), b0 + 10, "Win: credits `amount`.")?;
        c.post("/wallet/rollback", &c.rollback(&c.id(), &w)).err(409, "TRANSACTION_MISMATCH", fix)?;
        c.bal_is(b0 + 10, fix)
    });
    c.run("rollback naming a rollback is TRANSACTION_MISMATCH", |c| {
        let fix = "Idempotency: a rollback whose bet_transaction_id is the id of a rollback is TRANSACTION_MISMATCH (HTTP 409): the id is already used by another call type. Nothing moves.";
        let b0 = c.bal()?;
        let bet = c.placed_bet(100)?;
        let r = c.id();
        c.money_ok("/wallet/rollback", &c.rollback(&r, &bet), b0, "Rollback: credits the stake back.")?;
        c.post("/wallet/rollback", &c.rollback(&c.id(), &r)).err(409, "TRANSACTION_MISMATCH", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("bet with another player's session token is INVALID_SESSION", |c| {
        let fix = "Errors: a bet needs a session token that is valid and belongs to player_id; another player's token is INVALID_SESSION (HTTP 403) and nothing is debited. (The suite sends the test player's valid token with a different player_id.)";
        let b0 = c.bal()?;
        let mut body = c.bet(&c.id(), 100);
        body["player_id"] = json!(format!("{}-other", c.run));
        c.post("/wallet/bet", &body).err(403, "INVALID_SESSION", fix)?;
        c.bal_is(b0, fix)
    });
    c.run("unknown fields do not make a repeated body different", |c| {
        let fix = "Idempotency: \"the same body\" compares the fields the spec defines; fields it does not define are ignored, so a repeat that differs only in an unknown field returns the original response and moves no money.";
        let b0 = c.bal()?;
        let mut body = c.bet(&c.id(), 100);
        body["x_unknown"] = json!(1);
        let first = c.post("/wallet/bet", &body);
        first.ok(&c.cur, c.prec, fix)?;
        body["x_unknown"] = json!("other");
        let again = c.post("/wallet/bet", &body);
        if again.status != first.status || again.body != first.body {
            return Err(again.fail(format!("the original response: {} {}", first.status, first.text), fix));
        }
        c.bal_is(b0 - 100, fix)
    });
    c.run("a malformed request is not stored and leaves its transaction id free", |c| {
        let fix = "Idempotency: only a well-formed request is stored; a malformed one (here a bet of 0, INVALID_REQUEST) does not use up its transaction_id, so the same id with a valid body is executed normally.";
        let id = c.id();
        let b0 = c.bal()?;
        c.post("/wallet/bet", &c.bet(&id, 0)).err(400, "INVALID_REQUEST", fix)?;
        c.money_ok("/wallet/bet", &c.bet(&id, 100), b0 - 100, fix)
    });
    c.run("amounts written as 100.0 or 1e2 are INVALID_REQUEST", |c| {
        let fix = "Money: a JSON number is an integer only when written as an integer literal; `100.0` and `1e2` are INVALID_REQUEST (HTTP 400) for amount even though their value is 100. Inspect the number as written, not the parsed value. Nothing moves.";
        let b0 = c.bal()?;
        for lit in ["100.0", "1e2"] {
            let id = c.id();
            let lit = format!("\"amount\":{lit}");
            let raw = c.bet(&id, 100).to_string().replace("\"amount\":100", &lit);
            c.send_raw("/wallet/bet", raw, Sig::Good).err(400, "INVALID_REQUEST", fix)?;
            let raw = c.win(&id, 100).to_string().replace("\"amount\":100", &lit);
            c.send_raw("/wallet/win", raw, Sig::Good).err(400, "INVALID_REQUEST", fix)?;
        }
        c.bal_is(b0, fix)
    });
    c.run("rollback is never CURRENCY_MISMATCH", |c| {
        let fix = "Rollback: it has no currency or precision fields, so CURRENCY_MISMATCH never applies; unknown fields are ignored. The response carries the bet owner's balance, currency and precision.";
        let b0 = c.bal()?;
        let bet = c.placed_bet(100)?;
        let mut body = c.rollback(&c.id(), &bet);
        body["currency"] = json!(if c.cur == "ZZZ" { "YYY" } else { "ZZZ" });
        body["precision"] = json!(if c.prec == 3 { 4 } else { 3 });
        c.money_ok("/wallet/rollback", &body, b0, fix)
    });

    match expired {
        Some(old) => {
            c.run("win is accepted with an expired session", |c| {
                let b0 = c.bal()?;
                let mut body = c.win(&c.id(), 1);
                body["session_token"] = json!(old);
                c.money_ok("/wallet/win", &body, b0 + 1, "Win: the operator must accept a win even after the session token expired (the token is not checked).")
            });
            c.run("rollback is accepted with an expired session", |c| {
                let b0 = c.bal()?;
                let bet = c.placed_bet(100)?;
                let mut body = c.rollback(&c.id(), &bet);
                body["session_token"] = json!(old);
                c.money_ok("/wallet/rollback", &body, b0, "Rollback: the operator must accept a rollback even after the session token expired (the token is not checked).")
            });
        }
        None => {
            c.skip("win is accepted with an expired session");
            c.skip("rollback is accepted with an expired session");
        }
    }

    println!(
        "\n{} passed, {} failed, {} skipped",
        c.passed, c.failed, c.skipped
    );
    if c.skipped > 0 {
        println!(
            "skipped checks are not verified; go-live needs zero skips (pass --expired-session-token)"
        );
    }
    std::process::exit(if c.failed == 0 { 0 } else { 1 });
}
