# Paginate `badges_of`

**Difficulty:** hard
**Labels:** help wanted, area:contracts

## Problem

v0's `badges_of(event_id, attendee)` is bounded by design: one attendee holds
at most one badge per event, so there is nothing to paginate **within an
event**. What a "my badges" screen eventually needs is all badges for an
attendee **across events**, and that listing must not become an unbounded
loop.

## Scope

- A `badges_of_page(attendee, cursor, limit)` style read with a documented
  `limit` cap, returning badges plus an opaque cursor.
- Use bounded persistent index pages, not an ever-growing single vector per attendee.
  Define write/read resource caps and extend TTL for every index page touched.
- Specify revoked/expired membership behavior and bind opaque cursors to the
  attendee and ordering. Malformed or mismatched cursors must fail predictably.
- Out of scope: changing the existing per-event `badges_of`, any indexer
  work.

## Acceptance criteria

- [ ] The cap constant is documented and enforced; a test claims up to the
      cap and walks the pages.
- [ ] `docs/events.md` and `ERRORS.md` updated only if the change adds an
      event or error variant.
- [ ] TTL of the new index entries is covered by a test, as in
      `claim_extends_the_badge_ttl`.

- [ ] Tests cover revoked and expired records, empty/final pages, malformed and
      cross-attendee cursors, and index-page TTL.
- [ ] Measured resource usage stays bounded as total attendance grows; no call
      scans or rewrites the complete attendee history.

## Where to start

`src/badges.rs::badges_of`, `src/storage.rs` (`AttendeeBadges` shows the
list-per-key pattern), and the bounded-inputs rule in `AGENTS.md`.

## How to test

```bash
cargo test
node scripts/check-errors.mjs
```
