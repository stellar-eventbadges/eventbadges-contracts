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

/// The longest proof a `claim` can carry. A tree over `MAX_CLAIMS_PER_EVENT`
/// leaves is `ceil(log2(10_000)) = 14` levels deep (`2^14 = 16_384`), so a
/// longer proof cannot reach any root this contract can store and is refused
/// before any hashing happens. Kept honest by
/// `proof_depth_covers_the_maximum_claims` in `src/test.rs`.
pub const MAX_PROOF_DEPTH: u32 = 14;

/// One Merkle step: hashes a node with its sibling, sorting the two first so a
/// proof is an unordered list of siblings. This is the same rule the
/// organizer's tree builder uses in `docs/claim-codes.md`.
fn hash_pair(env: &Env, a: &BytesN<32>, b: &BytesN<32>) -> BytesN<32> {
    let (low, high) = if a <= b { (a, b) } else { (b, a) };
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(&low.to_array());
    bytes[32..].copy_from_slice(&high.to_array());
    BytesN::from(env.crypto().sha256(&Bytes::from_array(env, &bytes)))
}

/// Folds `leaf` up the tree with `proof` and reports whether the result is the
/// event's committed root. An empty proof verifies only against a root that
/// *is* the leaf, which is the one-attendee case.
fn verify_merkle(env: &Env, leaf: &BytesN<32>, proof: &Vec<BytesN<32>>, root: &BytesN<32>) -> bool {
    if proof.len() > MAX_PROOF_DEPTH {
        return false;
    }

    let mut node = leaf.clone();
    for sibling in proof.iter() {
        node = hash_pair(env, &node, &sibling);
    }
    node == *root
}

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

/// Records a new event for `organizer`, with an opaque name hash, the Merkle
/// root over its attendees' claim-code hashes, a badge cap and a claim
/// deadline.
///
/// The root is stored but deliberately left out of the `EventCreated` event:
/// it is already readable from `get_event`, so repeating it would only widen
/// the event payload for indexers.
pub fn create_event(
    env: &Env,
    organizer: Address,
    name_hash: BytesN<32>,
    claim_root: BytesN<32>,
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
        claim_root,
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

/// Claims a badge for `attendee` by presenting the leaf hash of their own
/// claim code and a Merkle proof that it belongs to the event's claim root.
/// One badge per attendee per event, and one place per leaf; the window must
/// still be open and the cap not reached.
///
/// The caller hashes the code off-chain and sends the leaf, so the raw secret
/// never rides in the transaction (ADR 0002), and the leaf is spent here so
/// reading one out of a transaction cannot take a place (ADR 0003). See
/// `docs/decisions/0003-per-attendee-claim-codes.md`.
pub fn claim(
    env: &Env,
    event_id: u64,
    attendee: Address,
    leaf_hash: &BytesN<32>,
    proof: &Vec<BytesN<32>>,
) -> Result<(), Error> {
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

    if !verify_merkle(env, leaf_hash, proof, &event.claim_root) {
        return Err(Error::ClaimProofInvalid);
    }

    // The leaf is the only part of a claim that could be replayed by anyone
    // who reads the transaction, so spending it is what makes one code equal
    // one place. Doing this after the last fallible check means a failed claim
    // never burns a leaf.
    let leaf_key = DataKey::RedeemedLeaf(event_id, leaf_hash.clone());
    if env.storage().persistent().has(&leaf_key) {
        return Err(Error::ClaimCodeUsed);
    }

    event.claim_count = count_after_issue(&event)?;
    env.storage().persistent().set(&leaf_key, &true);
    extend_record_ttl(env, &leaf_key, event.closes_at);
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
