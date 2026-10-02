# Add unique per-attendee claim codes via Merkle proofs

**Difficulty:** hard
**Labels:** help wanted, area:contracts

## Problem

v0 uses one shared claim code per event. Anyone who receives it before the
window opens can claim on behalf of a stranger (the stranger can still be
revoked, but the friction is real), and the code must be re-generated per
event. Real check-in flows need a code that only one attendee can redeem.

## Scope

- The organizer commits to a Merkle root of per-attendee code hashes at
  `create_event` time (a new field or a `create_event_v2`; the ADR records the
  choice).
- `claim` takes a Merkle proof alongside the code and verifies membership
  before minting.
- Keep the existing `claim_code_hash` path working or replace it explicitly —
  do not leave two claim systems half-alive.
- Out of scope: anything about the app's QR flow.

## Acceptance criteria

- [ ] A design note exists in `docs/decisions/` covering root storage and
      proof format.
- [ ] `claim` rejects a proof that does not verify, with a distinct error
      variant and an `error_paths` test.
- [ ] A used code cannot be reused (the per-attendee cap already blocks it;
      add the test).
- [ ] `cargo test`, `node scripts/check-errors.mjs` and
      `stellar contract build` all pass; `ERRORS.md` and `docs/events.md`
      updated in the same change.

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
