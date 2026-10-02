# Add badge metadata and images

**Difficulty:** medium
**Labels:** help wanted, area:contracts

## Problem

Badges carry no artwork or metadata, so the app can only render a generic
card. The playbook's v0 boundary leaves metadata out deliberately; this draft
defines how to add it without putting personal data or large blobs on-chain.

## Scope

- Event-level (not badge-level) metadata: a `metadata_hash` and a content
  address pointing at off-chain storage the organizer controls, following the
  OpenZeppelin Stellar metadata approach as the pattern (cite it, do not copy
  code; evaluate its license and audit status first).
- No attendee-derived metadata in v0 of this feature: names, photos of
  people, or anything personal stays off-chain.
- Out of scope: ipfs pinning services, any centralized default URL the
  contract hardcodes.

## Acceptance criteria

- [ ] `docs/decisions/` records the storage location choice and the privacy
      review.
- [ ] The contract stores only hashes/addresses; `ERRORS.md` and
      `docs/events.md` updated if anything new is emitted.
- [ ] A test asserts the metadata fields round-trip and that none of them can
      be set by a non-organizer.

## Where to start

`src/types.rs` (`Event` struct), `src/badges.rs::create_event`, and the
privacy rules in `AGENTS.md`.

## How to test

```bash
cargo test
node scripts/check-errors.mjs
```
