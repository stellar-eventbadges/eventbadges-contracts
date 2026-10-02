#![cfg(test)]

//! Exactly one test per `Error` variant, named `error_path_<variant>` in
//! snake_case. Each test triggers the real failure path rather than
//! constructing the error value directly.
//!
//! Add a test here in the same commit that adds a variant to `src/types.rs`.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::test_helpers::{
    advance_time, hash_of, setup, setup_event, synthetic_code, synthetic_hash, DAY_SECONDS,
};

#[test]
fn error_path_event_not_found() {
    let env = Env::default();
    let (_, client) = setup(&env);

    assert_eq!(client.try_get_event(&99), Err(Ok(Error::EventNotFound)));
    assert_eq!(
        client.try_has_badge(&99, &Address::generate(&env)),
        Err(Ok(Error::EventNotFound))
    );
    assert_eq!(
        client.try_badges_of(&99, &Address::generate(&env)),
        Err(Ok(Error::EventNotFound))
    );
}

#[test]
fn error_path_badge_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    assert_eq!(
        client.try_revoke(&event_id, &Address::generate(&env)),
        Err(Ok(Error::BadgeNotFound))
    );
}

#[test]
fn error_path_event_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);
    let attendee = Address::generate(&env);

    advance_time(&env, DAY_SECONDS + 60);

    assert_eq!(
        client.try_claim(&event_id, &attendee, &synthetic_code(&env, 0xC7)),
        Err(Ok(Error::EventClosed))
    );
    assert_eq!(
        client.try_award(&event_id, &Address::generate(&env)),
        Err(Ok(Error::EventClosed))
    );
}

#[test]
fn error_path_claim_code_mismatch() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);
    let attendee = Address::generate(&env);

    // The event's hash came from seed 0xC7; this code hashes differently.
    assert_eq!(
        client.try_claim(&event_id, &attendee, &synthetic_code(&env, 0x01)),
        Err(Ok(Error::ClaimCodeMismatch))
    );
}

#[test]
fn error_path_cap_reached() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let now = env.ledger().timestamp();
    let event_id = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &hash_of(&env, &synthetic_code(&env, 0xC7)),
        &1,
        &(now + DAY_SECONDS),
    );

    client.claim(
        &event_id,
        &Address::generate(&env),
        &synthetic_code(&env, 0xC7),
    );

    assert_eq!(
        client.try_claim(
            &event_id,
            &Address::generate(&env),
            &synthetic_code(&env, 0xC7)
        ),
        Err(Ok(Error::CapReached))
    );
    assert_eq!(
        client.try_award(&event_id, &Address::generate(&env)),
        Err(Ok(Error::CapReached))
    );
}

#[test]
fn error_path_already_held() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);
    let attendee = Address::generate(&env);

    client.claim(&event_id, &attendee, &synthetic_code(&env, 0xC7));

    assert_eq!(
        client.try_claim(&event_id, &attendee, &synthetic_code(&env, 0xC7)),
        Err(Ok(Error::AlreadyHeld))
    );
    assert_eq!(
        client.try_award(&event_id, &attendee),
        Err(Ok(Error::AlreadyHeld))
    );
}

#[test]
fn error_path_max_claims_too_large() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let now = env.ledger().timestamp();

    assert_eq!(
        client.try_create_event(
            &organizer,
            &synthetic_hash(&env, 0xA1),
            &hash_of(&env, &synthetic_code(&env, 0xC7)),
            &0,
            &(now + DAY_SECONDS),
        ),
        Err(Ok(Error::MaxClaimsTooLarge))
    );
    assert_eq!(
        client.try_create_event(
            &organizer,
            &synthetic_hash(&env, 0xA1),
            &hash_of(&env, &synthetic_code(&env, 0xC7)),
            &10_001,
            &(now + DAY_SECONDS),
        ),
        Err(Ok(Error::MaxClaimsTooLarge))
    );
}

#[test]
fn error_path_closes_at_in_past() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let now = env.ledger().timestamp();

    assert_eq!(
        client.try_create_event(
            &organizer,
            &synthetic_hash(&env, 0xA1),
            &hash_of(&env, &synthetic_code(&env, 0xC7)),
            &100,
            &now,
        ),
        Err(Ok(Error::ClosesAtInPast))
    );
}
