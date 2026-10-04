# 0003. Per-attendee claim codes via Merkle proofs

Date: 2026-10-04
Status: accepted — implemented 2026-10-04

## Context

v0 has one claim code per event. `create_event` stores a single digest and
`claim` compares the presented digest against it, so the code is a shared
bearer token: one code opens every place the event has. Two defects follow, and
[ADR 0002](0002-claim-code-not-in-transactions.md) names both while fixing only
the first:

| Defect | What it means |
|---|---|
| Publicity | fixed by ADR 0002: a claim transaction carries the digest, not the code |
| **Sharing** | the code is shared by every attendee, so a leaked code takes a place, and — worse — the stored digest is readable through `get_event`, so the means to claim is published by the contract itself |

`ROADMAP.md` lists this defect as "unique per-attendee claim codes (research
Merkle proofs first)". This is that work.

## What was checked

- **The commitment is public either way.** `get_event` and `load_event` have no
  `require_auth`, so whatever `create_event` stores can be read by anyone. A
  Merkle **root** is safe to publish because the leaves are not derivable from
  it; a single digest is not, because it *is* the value a claim presents.
- **The client already has the leaf function.** `hashClaimCode` in
  `eventbadges-app/src/lib/claimCode.ts` produces `SHA-256` of the 32-byte code
  in the browser, cross-checked against Node's own SHA-256, so leaves need no
  new client code.
- **The per-attendee cap already exists.** `AlreadyHeld` stops one address
  claiming twice; it does not stop a *second address* reusing a leaf it saw in
  someone else's transaction, which is why this decision adds a nullifier.
- **Nothing is deployed.** No state to migrate, no integration to keep
  compatible, so the argument name, the stored field and the error enum can all
  change shape.

## Decision

**One leaf per attendee, committed as a Merkle root.**

- `create_event` takes `claim_root: BytesN<32>` (renamed from
  `claim_code_hash`: same type, same position, different meaning).
- **Leaf**: `SHA-256(code)` for each attendee's own 32-byte code. Nothing else
  is mixed in, so the app's existing helper produces leaves directly, and a
  one-attendee event's root *is* that leaf — an empty proof verifies.
- **Internal node**: `SHA-256(min(a, b) || max(a, b))`, a sorted pair, so a
  proof is an unordered list of siblings.
- **Odd levels**: the last node is hashed with itself. This keeps every proof
  exactly `ceil(log2(leaves))` long, which means the verifier never needs to
  know how many leaves the tree has.
- **Claim**: `claim(event_id, attendee, leaf_hash, proof)` verifies the proof
  against the stored root, then spends the leaf, then issues the badge. The
  argument is a digest, not a code, so ADR 0002's property is preserved.
- **Depth bound**: `MAX_PROOF_DEPTH = 14`, derived from
  `MAX_CLAIMS_PER_EVENT = 10_000` (`2^14 = 16_384`). A longer proof is refused
  before any hashing.
- **Nullifier**: `DataKey::RedeemedLeaf(event_id, leaf)` is written when a leaf
  is claimed. Without it, a leaf is a reusable bearer token again — anyone who
  reads a claim transaction could present the same leaf with their own address
  and `AlreadyHeld` would not object, because their key is different. With it,
  **one code, one place**.
- **Check order**: auth, event exists, cap, deadline, already-held, proof,
  leaf already spent. A spent leaf defended with a valid proof gets
  `ClaimCodeUsed`; a bad proof gets `ClaimProofInvalid`.
- **Errors**: `ClaimCodeMismatch` is **removed**. No code path can return it
  now, and this repository requires exactly one error-path test per variant
  that triggers the real failure, so an unreachable variant cannot be kept.
  `ClaimProofInvalid = 14` and `ClaimCodeUsed = 15` take its place in the
  lifecycle range (10–29). `ERRORS.md` and the app's vendored table move with
  it.

### Options considered and rejected

- **Address-bound leaves** — `leaf = SHA-256(address || code)` — would fix
  more: only the named address could redeem, so a leaked code would be
  worthless *and* the front-running surface would disappear, because the leaf
  would be useless to anyone else even if they saw it in a transaction.
  Rejected **for now**, not on merit: it requires every attendee's address
  before the event is created, and the documented flow hands codes out at the
  door (the worked example, and the app's QR drafts 01 and 02). Revisit if
  pre-registration becomes part of the product; the leaf formula is the only
  thing that would change, and the tree, proof and nullifier rules all stand.
- **A nullifier-free design, relying on `AlreadyHeld`.** Rejected: it is not a
  reuse guard across addresses, and a code that two people can use is not a
  per-attendee code.
- **Per-attendee codes without a tree** (N codes stored individually).
  Rejected: it puts `max_claims` entries in storage at creation time and needs
  a lookup per claim, for the same security; the tree stores 32 bytes per
  event.
- **Re-committing the root later** (a mutable root so codes can be added after
  creation). Rejected for v0: it adds an organizer-authorized state transition
  whose failure mode is invalidating codes already handed out. If address
  binding is wanted alongside the door flow, this is the design to revisit —
  with the app collecting addresses at check-in.

## What this does not fix

- **A leaked code can still be used by whoever holds it first.** The nullifier
  bounds the damage to the one place that leaf was for; it cannot tell who the
  leaf was meant for. The organizer's remedy is `revoke` (which frees the cap
  slot) plus `award` for the attendee who was locked out.
- **An attendee can still be locked out by a stranger**, and the badge they
  lose is theirs until the organizer intervenes.
- **The organizer is trusted to build the tree honestly.** Someone who builds
  a root with a leaf for themselves can claim a place, exactly as they could
  `award` one to themselves. The tree is a fairness mechanism, not a check on
  the organizer.

## Consequences

- **Easy:** the chain stops publishing the means to claim. `claim_root` is
  public and harmless; the leaves behind it are not derivable from it.
- **Easy:** no new event, no new error for the common paths, and the app's
  existing hash helper produces leaves unchanged.
- **Hard / accepted:** one persistent entry per spent leaf, bounded by
  `max_claims` per event, which must be extended alongside the event's other
  records.
- **Hard / accepted:** the organizer's workflow gains a step — generate one
  code per attendee, build the tree, distribute the codes — recorded in
  `docs/claim-codes.md`. A single-leaf tree still behaves like v0's one shared
  code for a one-attendee event.
- **Not optional:** the privacy text that says the public digest "is all anyone
  needs to claim a place" becomes false the moment this lands, and must be
  corrected in the same push (`eventbadges-app/docs/attendee-notice.md`,
  `src/lib/privacyNotice.ts`, and the docs book's privacy and threat-model
  pages).
- **What would change our mind:** a product decision to collect attendee
  addresses at registration, which makes address-bound leaves possible and
  retires the last of this defect. Or evidence that organizers cannot manage
  the tree-building step — in which case the honest fallback is a root of one
  leaf plus `award`, not a return to a single shared code.
