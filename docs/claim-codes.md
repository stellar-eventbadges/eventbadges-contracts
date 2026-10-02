# Claim codes: how they should be generated and used

The contract never sees a claim code. `create_event` receives only
`claim_code_hash` — the SHA-256 of the code — and `claim` receives the raw
code and checks it against that hash. This file records how the code itself
should be produced off-chain.

## The rule

**A claim code is a random secret, generated on the organizer's machine, never
derived from personal data.**

- Generate it with a cryptographically secure random generator — for example
  `openssl rand -hex 32`, or a password manager's generator set to maximum
  length. Do not build it from a name, an email, a ticket number, a date, or
  anything else a attendee could guess or that would leak who attended if the
  code were later exposed.
- Hash it once, locally: `sha256(code)` produces the `BytesN<32>` the contract
  stores as `claim_code_hash`. The raw code is then shared out-of-band — QR
  code at the door, private message, printed slip — and never uploaded.
- If the raw code ever leaks, the hash on-chain does not reveal it (preimage
  resistance), but anyone holding the raw code can claim. Treat a leaked code
  like a leaked password: generate a new event rather than trying to patch,
  since v0 has no way to rotate a stored hash.
- The code is shared by all attendees of an event (one code per event in v0);
  the contract's per-attendee cap (`AlreadyHeld`) is what stops one person
  from claiming twice. Unique per-attendee codes via Merkle proofs are
  deliberately unimplemented — see ROADMAP.md and the audit trail in
  `docs/decisions/0001-nft-approach.md`.

## Why hashes only, on-chain

The privacy rules in [AGENTS.md](../AGENTS.md) forbid personal data on-chain.
Storing `sha256(code)` instead of the code keeps the chain holding nothing but
an opaque commitment; the event's `name_hash` follows the same pattern for the
event's name. Test fixtures use synthetic bytes only — nothing in
`src/test.rs` or `test_snapshots/` derives from a real person.
