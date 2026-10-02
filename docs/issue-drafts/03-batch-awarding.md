# Add batch awarding

**Difficulty:** medium
**Labels:** help wanted, area:contracts

## Problem

Offline check-in at a real event means awarding dozens of badges one
transaction at a time — slow and expensive for the organizer.

## Scope

- `award_batch(event_id, attendees: Vec<Address>)`, organizer-authorized,
  with a **documented, bounded** batch cap (the contract rule against
  unbounded input lists).
- Per-attendee semantics identical to `award`: window, cap, `AlreadyHeld`.
  Decide and record whether a single failure aborts the batch or skips that
  attendee — and emit per-attendee events either way.
- Out of scope: batching in `claim` (attendees claim individually), any
  relayer/gasless flow.

## Acceptance criteria

- [ ] The cap constant is documented in the README and enforced with a new
      error variant plus an `error_paths` test.
- [ ] A test covers a mixed batch (some already-held) and asserts the chosen
      failure semantics.
- [ ] `ERRORS.md` gains the new variant row in the same change.

## Where to start

`src/badges.rs::award`, `src/types.rs` (error ranges), the bounded-list
convention in `AGENTS.md`.

## How to test

```bash
cargo test
node scripts/check-errors.mjs
stellar contract build
```
