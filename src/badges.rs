//! Badge issuance logic: event creation, claiming, awarding, revocation.
//!
//! `src/lib.rs` exposes these functions through `#[contractimpl]`; everything
//! that validates input, reads or writes storage, or checks a claim code
//! lives here.
//!
//! Non-transferability by construction: there is no transfer, approve or
//! operator function anywhere in this module. A badge only ever moves at the
//! moment it is minted to its holder and is deleted by `revoke`. See
//! `docs/decisions/0001-nft-approach.md`.
//!
//! Arithmetic: `max_claims` is capped and validated, and the release profile
//! builds with `overflow-checks = true` (see `Cargo.toml`), so an overflow
//! traps instead of wrapping silently.

use soroban_sdk::{Address, Bytes, BytesN, Env, Vec};

use crate::storage::{extend_instance_ttl, extend_record_ttl, DataKey};
use crate::types::{Badge, BadgeAwarded, BadgeClaimed, BadgeRevoked, Error, Event, EventCreated};

/// The most badges a single event can issue. Keeps `claim_count` arithmetic
/// and the per-attendee list bounded; documented in the README. One attendee
/// can hold at most one badge per event, enforced by the `AlreadyHeld` check
/// in `claim` and `award`.
pub const MAX_CLAIMS_PER_EVENT: u32 = 10_000;

/// Loads an event or reports its absence.
fn load_event(env: &Env, event_id: u64) -> Result<Event, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Event(event_id))
        .ok_or(Error::EventNotFound)
}

/// Shared open-event checks for `claim` and `award`.
fn check_event_open(event: &Event) -> Result<(), Error> {
    if event.claim_count >= event.max_claims {
        return Err(Error::CapReached);
    }
    Ok(())
}

/// Shared cap arithmetic: the count a successful issuance moves to.
fn count_after_issue(event: &Event) -> Result<u32, Error> {
    event
        .claim_count
        .checked_add(1)
        .filter(|next| *next <= event.max_claims)
        .ok_or(Error::CapReached)
}

/// Shared badge mint: writes the badge entry, the attendee's list entry, the
/// event's count, extends every touched record's TTL, and publishes.
fn issue_badge(env: &Env, event: &Event, attendee: &Address) {
    let event_id = event.id;
    let now = env.ledger().timestamp();
    let badge = Badge {
        event_id,
        attendee: attendee.clone(),
        organizer: event.organizer.clone(),
        issued_at: now,
    };

    let badge_key = DataKey::Badge(event_id, attendee.clone());
    env.storage().persistent().set(&badge_key, &badge);

    let mut list: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::AttendeeBadges(event_id, attendee.clone()))
        .unwrap_or(Vec::new(env));
    list.push_back(attendee.clone());
    let list_key = DataKey::AttendeeBadges(event_id, attendee.clone());
    env.storage().persistent().set(&list_key, &list);

    let event_key = DataKey::Event(event_id);
    env.storage().persistent().set(&event_key, event);

    extend_record_ttl(env, &event_key, event.closes_at);
    extend_record_ttl(env, &badge_key, event.closes_at);
    extend_record_ttl(env, &list_key, event.closes_at);
}

/// Records a new event for `organizer`, with an opaque name hash, the hash of
/// a random claim code, a badge cap and a claim deadline.
///
/// The claim code hash is stored but deliberately left out of the
/// `EventCreated` event: it is already shared out-of-band with attendees, and
/// publishing it again would only widen its exposure.
pub fn create_event(
    env: &Env,
    organizer: Address,
    name_hash: BytesN<32>,
    claim_code_hash: BytesN<32>,
    max_claims: u32,
    closes_at: u64,
) -> Result<u64, Error> {
    organizer.require_auth();

    if max_claims == 0 || max_claims > MAX_CLAIMS_PER_EVENT {
        return Err(Error::MaxClaimsTooLarge);
    }
    let now = env.ledger().timestamp();
    if closes_at <= now {
        return Err(Error::ClosesAtInPast);
    }

    let counter_key = DataKey::NextEventId;
    let event_id: u64 = env.storage().instance().get(&counter_key).unwrap_or(1);
    let next_id = event_id
        .checked_add(1)
        .unwrap_or_else(|| panic!("event id overflow"));

    let event = Event {
        id: event_id,
        organizer: organizer.clone(),
        name_hash: name_hash.clone(),
        claim_code_hash,
        max_claims,
        closes_at,
        claim_count: 0,
    };
    let event_key = DataKey::Event(event_id);
    env.storage().instance().set(&counter_key, &next_id);
    env.storage().persistent().set(&event_key, &event);

    extend_instance_ttl(env);
    extend_record_ttl(env, &event_key, closes_at);

    EventCreated {
        event_id,
        organizer,
        name_hash,
        max_claims,
        closes_at,
    }
    .publish(env);

    Ok(event_id)
}

/// Claims a badge for `attendee` using a claim code whose SHA-256 must match
/// the event's stored hash. One badge per attendee per event; the window must
/// still be open and the cap not reached.
pub fn claim(env: &Env, event_id: u64, attendee: Address, claim_code: &Bytes) -> Result<(), Error> {
    attendee.require_auth();

    let mut event = load_event(env, event_id)?;
    check_event_open(&event)?;
    if env.ledger().timestamp() > event.closes_at {
        return Err(Error::EventClosed);
    }

    let badge_key = DataKey::Badge(event_id, attendee.clone());
    if env.storage().persistent().has(&badge_key) {
        return Err(Error::AlreadyHeld);
    }

    let digest = env.crypto().sha256(claim_code);
    if BytesN::from(digest) != event.claim_code_hash {
        return Err(Error::ClaimCodeMismatch);
    }

    event.claim_count = count_after_issue(&event)?;
    issue_badge(env, &event, &attendee);

    BadgeClaimed {
        event_id,
        attendee,
        organizer: event.organizer,
    }
    .publish(env);

    Ok(())
}

/// Awards a badge directly, for attendees who cannot claim (no code, offline
/// check-in). Organizer-authorized; same window and cap rules as `claim`.
pub fn award(env: &Env, event_id: u64, attendee: Address) -> Result<(), Error> {
    let mut event = load_event(env, event_id)?;
    event.organizer.require_auth();
    check_event_open(&event)?;
    if env.ledger().timestamp() > event.closes_at {
        return Err(Error::EventClosed);
    }

    let badge_key = DataKey::Badge(event_id, attendee.clone());
    if env.storage().persistent().has(&badge_key) {
        return Err(Error::AlreadyHeld);
    }

    event.claim_count = count_after_issue(&event)?;
    issue_badge(env, &event, &attendee);

    BadgeAwarded {
        event_id,
        attendee,
        organizer: event.organizer,
    }
    .publish(env);

    Ok(())
}

/// Removes a badge. The window does not matter: an organizer must be able to
/// revoke a badge issued in error at any time.
pub fn revoke(env: &Env, event_id: u64, attendee: Address) -> Result<(), Error> {
    let event = load_event(env, event_id)?;
    event.organizer.require_auth();

    let badge_key = DataKey::Badge(event_id, attendee.clone());
    if !env.storage().persistent().has(&badge_key) {
        return Err(Error::BadgeNotFound);
    }

    env.storage().persistent().remove(&badge_key);

    let list_key = DataKey::AttendeeBadges(event_id, attendee.clone());
    if let Some(mut list) = env.storage().persistent().get::<_, Vec<Address>>(&list_key) {
        if let Some(index) = list.first_index_of(&attendee) {
            list.remove(index);
            if list.is_empty() {
                env.storage().persistent().remove(&list_key);
            } else {
                env.storage().persistent().set(&list_key, &list);
            }
        }
    }

    let mut stored = load_event(env, event_id)?;
    stored.claim_count = stored.claim_count.saturating_sub(1);
    let event_key = DataKey::Event(event_id);
    env.storage().persistent().set(&event_key, &stored);
    extend_record_ttl(env, &event_key, stored.closes_at);

    BadgeRevoked {
        event_id,
        attendee,
        organizer: event.organizer,
    }
    .publish(env);

    Ok(())
}

/// Returns the event record, extending its TTL.
pub fn get_event(env: &Env, event_id: u64) -> Result<Event, Error> {
    let event = load_event(env, event_id)?;
    extend_record_ttl(env, &DataKey::Event(event_id), event.closes_at);
    Ok(event)
}

/// Returns whether `attendee` holds a badge for the event, extending the
/// badge's TTL when it exists.
pub fn has_badge(env: &Env, event_id: u64, attendee: &Address) -> Result<bool, Error> {
    let event = load_event(env, event_id)?;

    let badge_key = DataKey::Badge(event_id, attendee.clone());
    let held = env.storage().persistent().has(&badge_key);
    if held {
        extend_record_ttl(env, &badge_key, event.closes_at);
    }
    Ok(held)
}

/// Returns the attendee's badges for the event (bounded: at most one, per
/// [`MAX_BADGES_PER_ATTENDEE_PER_EVENT`]), extending the list's TTL.
pub fn badges_of(env: &Env, event_id: u64, attendee: &Address) -> Result<Vec<Badge>, Error> {
    let event = load_event(env, event_id)?;

    let list_key = DataKey::AttendeeBadges(event_id, attendee.clone());
    let held = env.storage().persistent().has(&list_key);
    if held {
        extend_record_ttl(env, &list_key, event.closes_at);
    }

    let mut badges = Vec::new(env);
    if held {
        let badge = env
            .storage()
            .persistent()
            .get(&DataKey::Badge(event_id, attendee.clone()))
            .ok_or(Error::BadgeNotFound)?;
        badges.push_back(badge);
    }
    Ok(badges)
}
