# Signing

Implements: [openapi.yaml](openapi.yaml), section Signing.

Both directions use the same rule: header `X-Signature` = lowercase hex HMAC-SHA256 of the exact raw request body bytes, keyed with the shared secret.

## Verify incoming requests (wallet endpoints)
1. Read the raw body bytes. Do not parse JSON first. Capture the raw bytes before any body parser touches the request: a JSON parser mounted earlier or globally (for example `express.json()`) consumes the stream and breaks verification. Mount a raw-body reader on the wallet routes first (Express: `express.raw`), or keep the bytes the framework offers (a `verify` hook, the unparsed request body), and parse JSON yourself after the signature passes.
2. Compute HMAC-SHA256 with the secret over those bytes, hex lowercase.
3. Compare to the header in constant time (for example `crypto.timingSafeEqual`, `hmac.compare_digest`).
4. On mismatch or a missing header, return `INVALID_SIGNATURE` (see [errors.md](errors.md)) and do nothing else.

## Sign outgoing requests (launch)
1. Serialize the body to a string once.
2. Sign that exact string; send that exact string. Do not serialize twice.
3. Add `X-Signature` and `X-Operator-Id`.

Only the body is signed, not path or headers. Responses are not signed. Load the secret from configuration; never write it in code or logs.

Check: send a body with a wrong signature to any wallet endpoint; the answer must be `INVALID_SIGNATURE`. [conformance.md](conformance.md) tests this.
