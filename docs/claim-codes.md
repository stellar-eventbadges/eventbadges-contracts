# Claim codes: how they should be generated and used

The contract never sees a claim code. `create_event` receives a Merkle root
over the SHA-256 of every attendee's code, and `claim` receives one leaf plus
the proof that connects it to that root. Codes stay on the organizer's machine
and on the attendees' devices; only hashes travel. This file records how the
codes and the tree should be produced off-chain.

See [decisions/0003](decisions/0003-per-attendee-claim-codes.md) for why the
tree exists, and what it does and does not fix.

## The rule

**One random code per attendee, generated on the organizer's machine, never
derived from personal data, never reused across events.**

- Generate one code per attendee with a cryptographically secure random
  generator — for example `openssl rand -hex 32` per person, or a password
  manager's generator set to maximum length. Do not build a code from a name,
  an email, a ticket number, a date, or anything else an attendee could guess
  or that would leak who attended if the code were later exposed.
- **Hash each code once, locally**: `leaf = sha256(code)`, where `code` is the
  32 raw bytes. This is exactly what `hashClaimCode` in the app already does,
  so the app's existing helper produces leaves unchanged.
- **Build the tree** over the leaves, one leaf per attendee:

  ```text
  node(a, b)  = sha256( min(a, b) || max(a, b) )   # sorted pair
  leaf_i      = sha256( code_i )                   # 32-byte code
  ```

  Sort the two children before hashing, so a proof is an unordered list of
  siblings. When a level has an odd number of nodes, the last node is hashed
  with **itself** — that is what keeps every proof exactly
  `ceil(log2(attendees))` hashes long, and it is what the contract verifies.
  The root is the last node left.
- **Hand each attendee their own code** — QR code at the door, private
  message, printed slip — and keep nothing. The organizer needs to keep only
  the codes until they are distributed, and the tree (or the attendee list) if
  they want to issue a proof again.
- **A one-attendee event needs no tree**: its root is that attendee's leaf and
  the proof is empty.
- **The root is public and harmless.** `get_event` returns it to anyone, but
  the leaves are not derivable from it, so the contract no longer publishes the
  means to claim. This is the change from the single-code design, where the
  stored digest *was* the value a claim presented.
- **A code takes exactly one place.** `claim` spends the leaf it verifies, so
  the same code cannot take a second place even if the leaf is read out of a
  claim transaction (`ClaimCodeUsed`). The per-attendee cap stops the same
  address claiming twice; the spent-leaf record stops the same *code*.
- **If a code leaks**, whoever has it can take the one place it was for, before
  the attendee does. The contract cannot tell who a leaf was meant for: revoke
  the badge that used it and `award` one to the attendee who was shut out. If
  you need more than that, address-bound leaves are the design to revisit — see
  ADR 0003's rejected-options section, which needs attendee addresses before
  the event is created.
- **Generate fresh codes per event.** The contract keys spent leaves by event,
  so the same code in two trees would work in both; that is not a feature
  anyone should rely on.

## Why hashes only, on-chain

The privacy rules in [AGENTS.md](../AGENTS.md) forbid personal data on-chain.
Storing hashes instead of codes keeps the chain holding nothing but opaque
commitments; the event's `name_hash` follows the same pattern for the event's
name. Since
[decisions/0002](decisions/0002-claim-code-not-in-transactions.md) the claim
transaction carries a digest rather than the code, and since
[decisions/0003](decisions/0003-per-attendee-claim-codes.md) that digest is a
leaf of a tree whose root is the only thing stored — so neither the code nor
the means to claim it is published. Test fixtures use synthetic bytes only —
nothing in `src/test.rs` or `test_snapshots/` derives from a real person.
