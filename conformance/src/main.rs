//! Plays the studio's role against an operator wallet and checks it against docs/openapi.yaml.
//! The checks live in lib.rs; the client area runs the same ones.

use conformance::{Check, Status, run};

const USAGE: &str = "usage: conformance --wallet-url <url> --session-token <token> [--expired-session-token <token>]\n  the shared secret is read from POLARKAST_SHARED_SECRET (or --secret, which shows it in the process list)";

fn show(c: &Check) {
    match c.status {
        Status::Pass => println!("PASS {}", c.name),
        Status::Fail => println!(
            "FAIL {}\n  request:  {}\n  expected: {}\n  actual:   {}\n  fix:      {}",
            c.name, c.request, c.expected, c.actual, c.fix
        ),
        Status::Skip => println!("SKIP {} ({})", c.name, c.fix),
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
    let http = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let r = run(http, &url, &secret, &token, expired.as_deref(), &mut |c| {
        show(c);
        true
    });
    let n = |s| r.checks.iter().filter(|c| c.status == s).count();
    let (passed, failed, skipped) = (n(Status::Pass), n(Status::Fail), n(Status::Skip));
    if r.stopped {
        println!("\nstopped: cannot continue without a valid session; 0 passed, 1 failed");
        std::process::exit(1);
    }
    println!("\n{passed} passed, {failed} failed, {skipped} skipped");
    if skipped > 0 {
        println!(
            "skipped checks are not verified; go-live needs zero skips (pass --expired-session-token)"
        );
    }
    std::process::exit(if failed == 0 { 0 } else { 1 });
}
