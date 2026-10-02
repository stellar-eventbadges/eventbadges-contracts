# Add event series and streak badges

**Difficulty:** medium
**Labels:** help wanted, area:contracts

## Problem

Communities run recurring events and would like to recognize attendance
across them ("attended 3 of this series"), but v0 events are isolated.

## Scope

- A `series` grouping that events can join at creation, with series-level
  reads (badges held per attendee across the series).
- Streak logic stays **off-chain** in this feature: the contract only records
  the facts (which events an address attended), and the app derives streaks.
  On-chain streak rules would couple the contract to one community's policy.
- Privacy unchanged: series membership is addresses and hashes only.
- Out of scope: retroactive linking of existing events, any new issuance
  path.

## Acceptance criteria

- [ ] `docs/decisions/` records the series model and the off-chain streak
      decision.
- [ ] Series reads are bounded (documented caps) and tested.
- [ ] Existing event isolation is preserved: an event without a series
      behaves exactly as today.

## Where to start

`src/types.rs` (`Event`), `src/storage.rs` (a new key family), and the TTL
rule — series entries need their own deadline reasoning.

## How to test

```bash
cargo test
node scripts/check-errors.mjs
```
