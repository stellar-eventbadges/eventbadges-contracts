#![no_std]

// Keep this file thin: `#[contract]` and `#[contractimpl]` only. Logic,
// types, and storage rules live in the modules below.
mod badges;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, Vec};

use crate::types::{Badge, Error, Event};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Records a new event for `organizer`, with an opaque `name_hash`, the
    /// SHA-256 of a random claim code, a badge cap and a claim deadline
    /// (`closes_at`, Unix seconds). Returns the new event id.
    ///
    /// Errors:
    /// - [`Error::MaxClaimsTooLarge`] for `max_claims` outside 1..=`MAX_CLAIMS_PER_EVENT`.
    /// - [`Error::ClosesAtInPast`] when `closes_at` is not in the future.
    pub fn create_event(
        env: Env,
        organizer: Address,
        name_hash: BytesN<32>,
        claim_code_hash: BytesN<32>,
        max_claims: u32,
        closes_at: u64,
    ) -> Result<u64, Error> {
        badges::create_event(
            &env,
            organizer,
            name_hash,
            claim_code_hash,
            max_claims,
            closes_at,
        )
    }

    /// Claims a badge for `attendee` using a claim code whose SHA-256 must
    /// match the event's stored hash.
    ///
    /// Errors: [`Error::EventNotFound`], [`Error::CapReached`],
    /// [`Error::EventClosed`], [`Error::AlreadyHeld`],
    /// [`Error::ClaimCodeMismatch`].
    pub fn claim(env: Env, event_id: u64, attendee: Address, claim_code: Bytes) -> Result<(), Error> {
        badges::claim(&env, event_id, attendee, &claim_code)
    }

    /// Awards a badge directly, for attendees who cannot claim. Organizer-authorized.
    ///
    /// Errors: [`Error::EventNotFound`], [`Error::CapReached`],
    /// [`Error::EventClosed`], [`Error::AlreadyHeld`].
    pub fn award(env: Env, event_id: u64, attendee: Address) -> Result<(), Error> {
        badges::award(&env, event_id, attendee)
    }

    /// Removes a badge. Organizer-authorized; allowed at any time.
    ///
    /// Errors: [`Error::EventNotFound`], [`Error::BadgeNotFound`].
    pub fn revoke(env: Env, event_id: u64, attendee: Address) -> Result<(), Error> {
        badges::revoke(&env, event_id, attendee)
    }

    /// Returns the event record.
    ///
    /// Errors: [`Error::EventNotFound`].
    pub fn get_event(env: Env, event_id: u64) -> Result<Event, Error> {
        badges::get_event(&env, event_id)
    }

    /// Returns whether `attendee` holds a badge for the event.
    ///
    /// Errors: [`Error::EventNotFound`].
    pub fn has_badge(env: Env, event_id: u64, attendee: Address) -> Result<bool, Error> {
        badges::has_badge(&env, event_id, &attendee)
    }

    /// Returns the attendee's badges for the event (bounded: at most one).
    ///
    /// Errors: [`Error::EventNotFound`].
    pub fn badges_of(env: Env, event_id: u64, attendee: Address) -> Result<Vec<Badge>, Error> {
        badges::badges_of(&env, event_id, &attendee)
    }
}
