# Add unique per-attendee claim codes via Merkle proofs

**Status:** implemented 2026-10-04 — kept for the reasoning, not as open work
**Difficulty:** hard
**Labels:** help wanted, area:contracts

> **What landed.** [ADR 0003](../decisions/0003-per-attendee-claim-codes.md),
> and with it: `claim_root` on `Event` (replacing the single
> `claim_code_hash`), `claim(event_id, attendee, leaf_hash, proof)`, a
> sorted-pair tree with duplicated odd levels, `MAX_PROOF_DEPTH = 14`, a
> spent-leaf record, and the errors `ClaimProofInvalid` (14) and
> `ClaimCodeUsed` (15). `ClaimCodeMismatch` (11) is retired: no path returns
> it, and this repository will not keep an error variant no test can reach.
>
> **One correction to the criteria below.** "A used code cannot be reused (the
> per-attendee cap already blocks it)" was wrong. The cap blocks the same
> *address* claiming twice; a second address presenting a leaf it read out of
> a claim transaction is a different key and would have passed. The spent-leaf
> record is what makes one code one place, and it has its own test.
>
> **Not fixed, and now its own item:** address-bound leaves. The leaf is
> `SHA-256(code)`, so a leaked code can still be used by whoever holds it
> first. ADR 0003 records the address-bound alternative as rejected for now
> (it needs attendee addresses before the event is created) and `ROADMAP.md`
> carries it as the next step.

## Problem

v0 uses one shared claim code per event. Anyone who receives it before the
window opens can claim on behalf of a stranger (the stranger can still be
revoked, but the friction is real), and the code must be re-generated per
event. Real check-in flows need a code that only one attendee can redeem.

## Scope

- The organizer commits to a Merkle root of per-attendee code hashes at
  `create_event` time (a new field or a `create_event_v2`; the ADR records the
  choice).
- `claim` takes a Merkle proof alongside the presented code hash and verifies
  membership before minting. (The hash-only argument is
  [decisions/0002](../decisions/0002-claim-code-not-in-transactions.md); a
  per-attendee design must keep that property, so the proof proves membership
  of a digest, not of the raw code.)
- Keep the existing `claim_code_hash` path working or replace it explicitly —
  do not leave two claim systems half-alive.
- Out of scope: anything about the app's QR flow.

## Acceptance criteria

- [x] A design note exists in `docs/decisions/` covering root storage and
      proof format — ADR 0003.
- [x] `claim` rejects a proof that does not verify, with a distinct error
      variant and an `error_paths` test — `ClaimProofInvalid`,
      `error_path_claim_proof_invalid`, plus proof-from-another-tree, missing
      proof and over-long proof tests in `src/test.rs`.
- [x] A used code cannot be reused — via the spent-leaf record, not the cap as
      this draft originally assumed; `error_path_claim_code_used` and
      `a_leaf_can_take_only_one_place`.
- [x] `cargo test`, `node --test`, `node scripts/check-errors.mjs`,
      `node scripts/check-events.mjs` and `stellar contract build` all pass;
      `ERRORS.md` updated in the same change. `docs/events.md` needed no
      change: no event layout moved, and the checker proves it.

## Where to start

`src/badges.rs::claim`, `src/types.rs` (new error variants), the storage
sketch in `docs/decisions/0001-nft-approach.md`, and any of Soroban's
Merkle-example contracts for the proof verifier pattern.

## How to test

```bash
cargo test
node scripts/check-errors.mjs
stellar contract build
```
