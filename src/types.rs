//! Contract error codes, stored types, and events.
//!
//! Codes are grouped into ranges by category so that new variants can be added
//! without renumbering the ones that already exist:
//!
//! ```text
//! 1-9    Lookup
//! 10-29  Lifecycle & timing
//! 30-49  Validation
//! ```
//!
//! Every variant here has exactly one row in `ERRORS.md` at the repository
//! root and exactly one test in `src/error_paths.rs`. `scripts/check-errors.mjs`
//! fails CI if this enum and `ERRORS.md` drift apart.

use soroban_sdk::{contracterror, contractevent, contracttype, Address, BytesN};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // 1-9: Lookup
    /// No event record exists for the given id.
    EventNotFound = 1,
    /// No badge record exists for the given event and attendee.
    BadgeNotFound = 2,
    // 10-29: Lifecycle & timing
    /// The event's claim window has closed (`closes_at` passed).
    EventClosed = 10,
    /// The presented claim code hash does not match the event's stored hash.
    ClaimCodeMismatch = 11,
    /// The event already holds `max_claims` badges.
    CapReached = 12,
    /// The attendee already holds a badge for this event.
    AlreadyHeld = 13,
    // 30-49: Validation
    /// `create_event` was called with a cap larger than the contract allows.
    MaxClaimsTooLarge = 30,
    /// `create_event` was called with a close time that is not in the future.
    ClosesAtInPast = 31,
}

/// An event recorded on-chain, with its claim window and cap.
///
/// `name_hash` and `claim_code_hash` are opaque 32-byte hashes: the organizer
/// derives them off-chain and they must never encode personal data. The claim
/// code hash is the SHA-256 of a random secret the organizer distributes
/// out-of-band — see `docs/claim-codes.md`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    /// Event id, starting at 1.
    pub id: u64,
    /// The organizer who created the event and controls its badges.
    pub organizer: Address,
    /// Opaque hash of the event name (no personal data).
    pub name_hash: BytesN<32>,
    /// SHA-256 of the secret claim code; checked on every claim.
    pub claim_code_hash: BytesN<32>,
    /// Maximum badges this event can issue.
    pub max_claims: u32,
    /// Claim deadline, in Unix seconds (the same clock as
    /// `env.ledger().timestamp()`). Claims and awards fail after it.
    pub closes_at: u64,
    /// Badges issued so far (claims plus awards).
    pub claim_count: u32,
}

/// A non-transferable attendance badge.
///
/// The badge is stored under `(event_id, attendee)` and has no transfer,
/// approve or operator path anywhere in the contract — see
/// `docs/decisions/0001-nft-approach.md`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Badge {
    /// The event the badge belongs to.
    pub event_id: u64,
    /// The holder. Storage key, but repeated so `badges_of` returns
    /// self-contained records.
    pub attendee: Address,
    /// The organizer that issued (claimed or awarded) the badge.
    pub organizer: Address,
    /// Issue time, in Unix seconds.
    pub issued_at: u64,
}

/// Emitted by `create_event` once an event is recorded.
///
/// The claim code hash is deliberately not in the event data: it is stored
/// with the event, but publishing it again would only widen its exposure.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventCreated {
    /// The new event's id.
    #[topic]
    pub event_id: u64,
    /// The organizer that created the event.
    pub organizer: Address,
    /// Opaque hash of the event name.
    pub name_hash: BytesN<32>,
    /// Maximum badges the event can issue.
    pub max_claims: u32,
    /// Claim deadline, in Unix seconds.
    pub closes_at: u64,
}

/// Emitted by `claim` when an attendee claims their own badge.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeClaimed {
    /// The event the badge belongs to.
    #[topic]
    pub event_id: u64,
    /// The attendee who claimed the badge.
    #[topic]
    pub attendee: Address,
    /// The organizer that created the event.
    pub organizer: Address,
}

/// Emitted by `award` when the organizer issues a badge directly.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeAwarded {
    /// The event the badge belongs to.
    #[topic]
    pub event_id: u64,
    /// The attendee who received the badge.
    #[topic]
    pub attendee: Address,
    /// The organizer that awarded the badge.
    pub organizer: Address,
}

/// Emitted by `revoke` after the organizer removes a badge.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeRevoked {
    /// The event the badge belonged to.
    #[topic]
    pub event_id: u64,
    /// The attendee whose badge was removed.
    #[topic]
    pub attendee: Address,
    /// The organizer that revoked the badge.
    pub organizer: Address,
}
