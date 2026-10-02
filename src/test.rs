#![cfg(test)]

//! Happy-path and integration tests. Error paths live in `error_paths.rs`.

use super::*;
use soroban_sdk::{
    map,
    testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal, Symbol,
};

use crate::storage::{DataKey, MIN_TTL_LEDGERS, SECONDS_PER_LEDGER};
use crate::test_helpers::{
    advance_time, hash_of, instance_ttl, mock_attendee_auth_for_claim,
    mock_organizer_auth_for_create_event, persistent_ttl, setup, setup_event, synthetic_code,
    synthetic_hash, to_val, DAY_SECONDS,
};

#[test]
fn create_event_records_the_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let name_hash = synthetic_hash(&env, 0xA1);
    let closes_at = env.ledger().timestamp() + 7 * DAY_SECONDS;

    let event_id = client.create_event(
        &organizer,
        &name_hash,
        &hash_of(&env, &synthetic_code(&env, 0xC7)),
        &250,
        &closes_at,
    );

    let event = client.get_event(&event_id);
    assert_eq!(event.id, 1);
    assert_eq!(event.organizer, organizer);
    assert_eq!(event.name_hash, name_hash);
    assert_eq!(event.max_claims, 250);
    assert_eq!(event.closes_at, closes_at);
    assert_eq!(event.claim_count, 0);

    // Ids are sequential.
    let second = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA2),
        &hash_of(&env, &synthetic_code(&env, 0xC7)),
        &250,
        &(env.ledger().timestamp() + 7 * DAY_SECONDS),
    );
    assert_eq!(second, 2);
}

#[test]
fn create_event_extends_the_instance_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);

    setup_event(&env, &client, &organizer);

    let ttl = instance_ttl(&env, &contract_id);
    assert!(
        ttl >= storage::INSTANCE_TTL_EXTEND_TO - 1,
        "expected the instance TTL to be extended to at least {} ledgers, got {}",
        storage::INSTANCE_TTL_EXTEND_TO,
        ttl
    );
}

#[test]
fn create_event_extends_the_event_record_ttl_toward_its_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let now = env.ledger().timestamp();
    let closes_at = now + 90 * DAY_SECONDS;

    let event_id = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &hash_of(&env, &synthetic_code(&env, 0xC7)),
        &100,
        &closes_at,
    );

    let ttl = persistent_ttl(&env, &contract_id, &DataKey::Event(event_id));
    // The record must reach at least the 7-day floor...
    assert!(
        ttl >= MIN_TTL_LEDGERS - 1,
        "expected the event TTL to reach at least {} ledgers, got {}",
        MIN_TTL_LEDGERS,
        ttl
    );
    // ...and stay under the full deadline horizon, which is far larger.
    let horizon_ledgers =
        u32::try_from((closes_at + storage::SETTLEMENT_MARGIN_SECONDS - now) / SECONDS_PER_LEDGER)
            .unwrap_or(u32::MAX);
    assert!(
        ttl <= horizon_ledgers,
        "expected the event TTL to stay under the {}-ledger horizon, got {}",
        horizon_ledgers,
        ttl
    );
}

#[test]
fn claim_issues_a_badge_to_the_caller() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    client.claim(&event_id, &attendee, &synthetic_code(&env, 0xC7));

    assert!(client.has_badge(&event_id, &attendee));
    let badges = client.badges_of(&event_id, &attendee);
    assert_eq!(badges.len(), 1);
    assert_eq!(badges.get(0).unwrap().attendee, attendee);
    assert_eq!(badges.get(0).unwrap().organizer, organizer);
    let event = client.get_event(&event_id);
    assert_eq!(event.claim_count, 1);
}

#[test]
fn claim_extends_the_badge_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    client.claim(&event_id, &attendee, &synthetic_code(&env, 0xC7));

    let ttl = persistent_ttl(&env, &contract_id, &DataKey::Badge(event_id, attendee));
    assert!(
        ttl >= MIN_TTL_LEDGERS - 1,
        "expected the badge TTL to reach at least {} ledgers, got {}",
        MIN_TTL_LEDGERS,
        ttl
    );
}

#[test]
fn claim_after_the_deadline_fails_but_revoke_still_works() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    client.claim(&event_id, &attendee, &synthetic_code(&env, 0xC7));
    // Move past the deadline (7 days) but stay inside the record's TTL.
    advance_time(&env, 31 * DAY_SECONDS);

    assert_eq!(
        client.try_claim(&event_id, &attendee, &synthetic_code(&env, 0xC7)),
        Err(Ok(Error::EventClosed))
    );

    // Revocation is not window-bound: a badge issued in error can be removed
    // at any time.
    client.revoke(&event_id, &attendee);
    assert!(!client.has_badge(&event_id, &attendee));
    let event = client.get_event(&event_id);
    assert_eq!(event.claim_count, 0);
}

#[test]
fn award_issues_a_badge_without_a_claim_code() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    client.award(&event_id, &attendee);

    assert!(client.has_badge(&event_id, &attendee));
    assert_eq!(client.get_event(&event_id).claim_count, 1);
}

#[test]
fn a_second_attendee_can_claim_with_the_same_code() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    client.claim(&event_id, &first, &synthetic_code(&env, 0xC7));
    client.claim(&event_id, &second, &synthetic_code(&env, 0xC7));

    assert!(client.has_badge(&event_id, &first));
    assert!(client.has_badge(&event_id, &second));
    assert_eq!(client.get_event(&event_id).claim_count, 2);
}

#[test]
fn award_then_revoke_then_award_issues_a_fresh_badge() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    client.award(&event_id, &attendee);
    client.revoke(&event_id, &attendee);
    assert!(!client.has_badge(&event_id, &attendee));

    client.award(&event_id, &attendee);
    assert!(client.has_badge(&event_id, &attendee));
    assert_eq!(client.get_event(&event_id).claim_count, 1);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn create_event_requires_the_organizer_signature() {
    let env = Env::default();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);

    // No auths are mocked, so the contract's `require_auth` must fail.
    client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &hash_of(&env, &synthetic_code(&env, 0xC7)),
        &100,
        &(env.ledger().timestamp() + DAY_SECONDS),
    );
}

/// Creates the event with only the organizer's signature mocked, so later
/// calls run against a real event with no blanket auth bypass.
fn setup_event_with_only_organizer_auth(
    env: &Env,
    contract_id: &Address,
    client: &ContractClient<'_>,
    organizer: &Address,
) -> u64 {
    let name_hash = synthetic_hash(env, 0xA1);
    let max_claims = 100_u32;
    let closes_at = env.ledger().timestamp() + DAY_SECONDS;
    mock_organizer_auth_for_create_event(
        env,
        contract_id,
        organizer,
        &name_hash,
        &hash_of(env, &synthetic_code(env, 0xC7)),
        max_claims,
        closes_at,
    );
    client.create_event(
        organizer,
        &name_hash,
        &hash_of(env, &synthetic_code(env, 0xC7)),
        &max_claims,
        &closes_at,
    )
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn claim_requires_the_attendee_signature() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let event_id = setup_event_with_only_organizer_auth(&env, &contract_id, &client, &organizer);

    // Only the organizer's signature was mocked for the creation call; the
    // attendee has not authorized this claim.
    client.claim(
        &event_id,
        &Address::generate(&env),
        &synthetic_code(&env, 0xC7),
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn award_requires_the_organizer_signature() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let event_id = setup_event_with_only_organizer_auth(&env, &contract_id, &client, &organizer);

    // The organizer's mock covered only the create_event invocation, not this
    // award call.
    client.award(&event_id, &Address::generate(&env));
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn revoke_requires_the_organizer_signature() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event_with_only_organizer_auth(&env, &contract_id, &client, &organizer);

    // The attendee's signature is mocked for the claim...
    mock_attendee_auth_for_claim(
        &env,
        &contract_id,
        &attendee,
        event_id,
        &synthetic_code(&env, 0xC7),
    );
    client.claim(&event_id, &attendee, &synthetic_code(&env, 0xC7));

    // ...but nobody has authorized this revoke.
    client.revoke(&event_id, &attendee);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn revoke_rejects_a_signature_from_someone_other_than_the_organizer() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let stranger = Address::generate(&env);
    let event_id = setup_event_with_only_organizer_auth(&env, &contract_id, &client, &organizer);

    mock_attendee_auth_for_claim(
        &env,
        &contract_id,
        &attendee,
        event_id,
        &synthetic_code(&env, 0xC7),
    );
    client.claim(&event_id, &attendee, &synthetic_code(&env, 0xC7));

    // A stranger authorizes the revoke invocation, but the contract demands
    // the organizer's signature, so the call must still fail.
    env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "revoke",
            args: (event_id, attendee.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.revoke(&event_id, &attendee);
}

#[test]
fn lifecycle_publishes_documented_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let name_hash = synthetic_hash(&env, 0xA1);
    let closes_at = env.ledger().timestamp() + DAY_SECONDS;
    let code = synthetic_code(&env, 0xC7);

    // `env.events().all()` returns the events of the last invocation, so each
    // step is asserted right after its call, filtered to this contract.
    let event_id = client.create_event(
        &organizer,
        &name_hash,
        &hash_of(&env, &code),
        &100,
        &closes_at,
    );
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "event_created"), event_id).into_val(&env),
                map![
                    &env,
                    (Symbol::new(&env, "closes_at"), to_val(&env, closes_at)),
                    (Symbol::new(&env, "max_claims"), to_val(&env, 100_u32)),
                    (
                        Symbol::new(&env, "name_hash"),
                        to_val(&env, name_hash.clone())
                    ),
                    (
                        Symbol::new(&env, "organizer"),
                        to_val(&env, organizer.clone())
                    ),
                ]
                .into_val(&env),
            ),
        ]
    );

    client.claim(&event_id, &attendee, &code);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "badge_claimed"),
                    event_id,
                    attendee.clone()
                )
                    .into_val(&env),
                map![
                    &env,
                    (
                        Symbol::new(&env, "organizer"),
                        to_val(&env, organizer.clone())
                    ),
                ]
                .into_val(&env),
            ),
        ]
    );

    client.revoke(&event_id, &attendee);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "badge_revoked"),
                    event_id,
                    attendee.clone()
                )
                    .into_val(&env),
                map![
                    &env,
                    (
                        Symbol::new(&env, "organizer"),
                        to_val(&env, organizer.clone())
                    ),
                ]
                .into_val(&env),
            ),
        ]
    );
}

#[test]
fn award_publishes_the_documented_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    client.award(&event_id, &attendee);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "badge_awarded"),
                    event_id,
                    attendee.clone()
                )
                    .into_val(&env),
                map![
                    &env,
                    (
                        Symbol::new(&env, "organizer"),
                        to_val(&env, organizer.clone())
                    ),
                ]
                .into_val(&env),
            ),
        ]
    );
}
