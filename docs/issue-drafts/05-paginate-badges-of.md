# Paginate `badges_of`

**Difficulty:** easy
**Labels:** good first issue, area:contracts

## Problem

v0's `badges_of(event_id, attendee)` is bounded by design: one attendee holds
at most one badge per event, so there is nothing to paginate **within an
event**. What a "my badges" screen eventually needs is all badges for an
attendee **across events**, and that listing must not become an unbounded
loop.

## Scope

- A `badges_of_page(attendee, cursor, limit)` style read with a documented
  `limit` cap, returning badges plus an opaque cursor.
- The attendee's cross-event index is a new persistent entry per attendee;
  it must follow the TTL rules like every other record.
- Out of scope: changing the existing per-event `badges_of`, any indexer
  work.

## Acceptance criteria

- [ ] The cap constant is documented and enforced; a test claims up to the
      cap and walks the pages.
- [ ] `docs/events.md` and `ERRORS.md` updated only if the change adds an
      event or error variant.
- [ ] TTL of the new index entries is covered by a test, as in
      `claim_extends_the_badge_ttl`.

## Where to start

`src/badges.rs::badges_of`, `src/storage.rs` (`AttendeeBadges` shows the
list-per-key pattern), and the bounded-inputs rule in `AGENTS.md`.

## How to test

```bash
cargo test
node scripts/check-errors.mjs
```
