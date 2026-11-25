# Polarkast operator kit

Everything an operator (or their coding agent) needs to integrate a casino with the Polarkast seamless-wallet API.

- `docs/`: the integration docs and the API spec. Start at [`docs/llms.txt`](docs/llms.txt).
- `conformance/`: the conformance CLI that tests your wallet the way Polarkast calls it.
- `reference/node-express`: a working reference wallet. `reference/blank-node`: a blank operator backend to integrate into.

## Get the conformance binary

Download it from the [`conformance-latest` release](https://github.com/Polarkast/integration/releases/tag/conformance-latest) (Linux x86_64, macOS arm64, Windows x86_64). Or build it:

```sh
cargo build --release -p conformance   # target/release/conformance
```

Usage: [`docs/conformance.md`](docs/conformance.md).

This repository is a read-only mirror; do not send changes here.
