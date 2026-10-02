# Contract events

Only events the code actually emits are documented here. Anything not listed is
**not implemented yet**.

Events are defined with `#[contractevent]` in `src/types.rs` and published with
`EventName { .. }.publish(&env)`. The SDK puts the event's name into the topics
as a `Symbol`, followed by every field marked `#[topic]`; the remaining fields
land in the data map. All four events below are asserted with exact topic and
data layouts in `src/test.rs`.

## `EventCreated`

Emitted once per event, by `create_event`.

| | |
|---|---|
| Topics | `Symbol("event_created")`, then `event_id` (`u64`) |
| Data | map with `organizer` (`Address`), `name_hash` (`BytesN<32>`), `max_claims` (`u32`), `closes_at` (`u64`) |
| Emitted by | `create_event` in `src/badges.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

The claim code hash is deliberately **not** in the event data: it is stored
with the event, but publishing it again would only widen its exposure. See
[claim-codes.md](claim-codes.md).

## `BadgeClaimed`

Emitted on every successful claim, by `claim`.

| | |
|---|---|
| Topics | `Symbol("badge_claimed")`, then `event_id` (`u64`) and `attendee` (`Address`) |
| Data | map with `organizer` (`Address`) |
| Emitted by | `claim` in `src/badges.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

An indexer can therefore watch for one attendee's badge across all events by
the topic pair alone.

## `BadgeAwarded`

Emitted on every successful direct issuance, by `award`.

| | |
|---|---|
| Topics | `Symbol("badge_awarded")`, then `event_id` (`u64`) and `attendee` (`Address`) |
| Data | map with `organizer` (`Address`) |
| Emitted by | `award` in `src/badges.rs` |
| Asserted in | `award_publishes_the_documented_event` in `src/test.rs` |

## `BadgeRevoked`

Emitted once per removal, by `revoke`.

| | |
|---|---|
| Topics | `Symbol("badge_revoked")`, then `event_id` (`u64`) and `attendee` (`Address`) |
| Data | map with `organizer` (`Address`) |
| Emitted by | `revoke` in `src/badges.rs` |
| Asserted in | `lifecycle_publishes_documented_events` in `src/test.rs` |

## Not implemented yet

- No other events exist yet. New events are documented here in the same commit
  that adds them to `src/types.rs`.
