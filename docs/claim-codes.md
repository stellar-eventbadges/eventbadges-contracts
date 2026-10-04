# Claim codes: how they should be generated and used

The contract never sees a claim code. `create_event` receives only
`claim_code_hash` — the SHA-256 of the code — and `claim` receives that same
digest, which it compares to the stored hash byte for byte. The raw code never
enters a transaction: the organizer's machine generates it, the attendee's
client hashes it, and only the digest is sent. This file records how the code
itself should be produced off-chain.

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
  code at the door, private message, printed slip — and **never uploaded**. A
  claim transaction carries the digest, not the code, so the code is never
  published to the mempool or the ledger.
- **The digest is not a secret.** `get_event` returns `claim_code_hash` to
  anyone, for any event, from the moment it is created, so anyone who reads
  the chain holds the value a successful claim presents. Knowing the digest
  does not reveal the code (SHA-256 is preimage-resistant), but it is a
  bearer credential: while the window is open it can claim a slot. This is
  the sharing defect, and it is deliberately not fixed here — see
  [ROADMAP.md](../ROADMAP.md) and
  [decisions/0002](decisions/0002-claim-code-not-in-transactions.md).
- If the raw code ever leaks, the hash on-chain does not reveal it (preimage
  resistance), but anyone holding the raw code can compute the digest and
  claim. Treat a leaked code like a leaked password: generate a new event
  rather than trying to patch, since v0 has no way to rotate a stored hash.
- The code is shared by all attendees of an event (one code per event in v0);
  the contract's per-attendee cap (`AlreadyHeld`) is what stops one person
  from claiming twice. Unique per-attendee codes via Merkle proofs are
  deliberately unimplemented — see ROADMAP.md and the audit trail in
  `docs/decisions/0001-nft-approach.md`.

## Why hashes only, on-chain

The privacy rules in [AGENTS.md](../AGENTS.md) forbid personal data on-chain.
Storing `sha256(code)` instead of the code keeps the chain holding nothing but
an opaque commitment; the event's `name_hash` follows the same pattern for the
event's name. Since
[decisions/0002](decisions/0002-claim-code-not-in-transactions.md) the same
reasoning applies to the claim transaction itself: the contract stores the
digest and compares against the digest, so no transaction ever publishes a
reusable secret. Test fixtures use synthetic bytes only — nothing in
`src/test.rs` or `test_snapshots/` derives from a real person.
