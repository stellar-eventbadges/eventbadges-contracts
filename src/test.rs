#![cfg(test)]

//! Happy-path and integration tests. Error paths live in `error_paths.rs`.

use super::*;
use soroban_sdk::{
    map,
    testutils::{Address as _, Events as _, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal, Symbol,
};

use crate::badges::{MAX_CLAIMS_PER_EVENT, MAX_PROOF_DEPTH};
use crate::storage::{DataKey, MIN_TTL_LEDGERS, SECONDS_PER_LEDGER};
use crate::test_helpers::{
    advance_time, hash_of, instance_ttl, merkle_tree, mock_attendee_auth_for_claim,
    mock_organizer_auth_for_create_event, persistent_ttl, setup, setup_claim_code_hash,
    setup_event, synthetic_code, synthetic_hash, to_val, DAY_SECONDS,
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

    client.claim(
        &event_id,
        &attendee,
        &setup_claim_code_hash(&env),
        &Vec::new(&env),
    );

    assert!(client.has_badge(&event_id, &attendee));
    let badges = client.badges_of(&event_id, &attendee);
    assert_eq!(badges.len(), 1);
    assert_eq!(badges.get(0).unwrap().attendee, attendee);
    assert_eq!(badges.get(0).unwrap().organizer, organizer);
    let event = client.get_event(&event_id);
    assert_eq!(event.claim_count, 1);
}

#[test]
fn claim_rejects_the_raw_code_as_a_leaf() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    // `synthetic_hash(0xC7)` carries the same 32 bytes as the raw claim code;
    // every leaf is the SHA-256 of its code. Handing the contract the bytes
    // themselves must fail — this is what keeps the code out of the
    // transaction, and it fails if anyone restores hashing on-chain.
    assert_eq!(
        client.try_claim(
            &event_id,
            &attendee,
            &synthetic_hash(&env, 0xC7),
            &Vec::new(&env)
        ),
        Err(Ok(Error::ClaimProofInvalid))
    );
    assert!(!client.has_badge(&event_id, &attendee));
}

#[test]
fn a_single_leaf_tree_claims_with_an_empty_proof() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let now = env.ledger().timestamp();

    // Pins the documented leaf formula: the leaf of a code is SHA-256 of the
    // code, exactly what `hashClaimCode` produces in the app. A one-attendee
    // event's root is that leaf, so the proof is empty.
    let code = synthetic_code(&env, 0x5A);
    let leaf: BytesN<32> = env.crypto().sha256(&code).into();
    let event_id = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &leaf,
        &1,
        &(now + DAY_SECONDS),
    );

    client.claim(&event_id, &attendee, &leaf, &Vec::new(&env));

    assert!(client.has_badge(&event_id, &attendee));
    assert_eq!(client.get_event(&event_id).claim_count, 1);
}

#[test]
fn claim_extends_the_badge_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    client.claim(
        &event_id,
        &attendee,
        &setup_claim_code_hash(&env),
        &Vec::new(&env),
    );

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

    client.claim(
        &event_id,
        &attendee,
        &setup_claim_code_hash(&env),
        &Vec::new(&env),
    );
    // Move past the deadline (7 days) but stay inside the record's TTL.
    advance_time(&env, 31 * DAY_SECONDS);

    assert_eq!(
        client.try_claim(
            &event_id,
            &attendee,
            &setup_claim_code_hash(&env),
            &Vec::new(&env)
        ),
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
fn each_attendee_claims_with_their_own_leaf() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    let now = env.ledger().timestamp();
    let (root, entries) = merkle_tree(&env, &[0x11, 0x22]);
    let event_id = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &root,
        &100,
        &(now + DAY_SECONDS),
    );

    let (first_leaf, first_proof) = entries.get(0).unwrap();
    let (second_leaf, second_proof) = entries.get(1).unwrap();
    client.claim(&event_id, &first, &first_leaf, &first_proof);
    client.claim(&event_id, &second, &second_leaf, &second_proof);

    assert!(client.has_badge(&event_id, &first));
    assert!(client.has_badge(&event_id, &second));
    assert_eq!(client.get_event(&event_id).claim_count, 2);
}

#[test]
fn a_leaf_can_take_only_one_place() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let holder = Address::generate(&env);
    let bystander = Address::generate(&env);
    let now = env.ledger().timestamp();
    let (root, entries) = merkle_tree(&env, &[0x11, 0x22]);
    let event_id = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &root,
        &100,
        &(now + DAY_SECONDS),
    );

    let (leaf, proof) = entries.get(0).unwrap();
    client.claim(&event_id, &holder, &leaf, &proof);

    // The leaf is in the first transaction's arguments, which are public. A
    // second address presenting the same leaf must not get a second place:
    // that is what the nullifier is for.
    assert_eq!(
        client.try_claim(&event_id, &bystander, &leaf, &proof),
        Err(Ok(Error::ClaimCodeUsed))
    );
    assert!(!client.has_badge(&event_id, &bystander));

    // The rest of the tree is untouched: only the spent leaf is refused.
    let (other_leaf, other_proof) = entries.get(1).unwrap();
    client.claim(&event_id, &bystander, &other_leaf, &other_proof);
    assert!(client.has_badge(&event_id, &bystander));
    assert_eq!(client.get_event(&event_id).claim_count, 2);
}

#[test]
fn a_leaf_is_not_spent_on_other_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    let now = env.ledger().timestamp();
    let (root, entries) = merkle_tree(&env, &[0x11, 0x22]);
    let (leaf, proof) = entries.get(0).unwrap();

    let first_event = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &root,
        &100,
        &(now + DAY_SECONDS),
    );
    let second_event = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA2),
        &root,
        &100,
        &(now + DAY_SECONDS),
    );

    client.claim(&first_event, &first, &leaf, &proof);
    // The nullifier is keyed by event as well as leaf, so the same code
    // committed by a second event is not silently spent by the first.
    client.claim(&second_event, &second, &leaf, &proof);

    assert!(client.has_badge(&first_event, &first));
    assert!(client.has_badge(&second_event, &second));
}

#[test]
fn an_odd_level_tree_claims_every_leaf() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let now = env.ledger().timestamp();

    // Three leaves: the middle level is odd, so the last node is hashed with
    // itself. Every attendee must still be able to claim.
    let (root, entries) = merkle_tree(&env, &[0x31, 0x32, 0x33]);
    let event_id = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &root,
        &3,
        &(now + DAY_SECONDS),
    );

    for index in 0..entries.len() {
        let attendee = Address::generate(&env);
        let (leaf, proof) = entries.get(index).unwrap();
        client.claim(&event_id, &attendee, &leaf, &proof);
        assert!(client.has_badge(&event_id, &attendee));
    }

    assert_eq!(client.get_event(&event_id).claim_count, 3);
}

#[test]
fn a_missing_proof_is_refused() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let now = env.ledger().timestamp();
    let (root, entries) = merkle_tree(&env, &[0x11, 0x22]);
    let event_id = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &root,
        &100,
        &(now + DAY_SECONDS),
    );

    // The leaf is real, but a leaf outside a one-attendee tree needs its
    // siblings to reach the root.
    let (leaf, _) = entries.get(0).unwrap();
    assert_eq!(
        client.try_claim(&event_id, &attendee, &leaf, &Vec::new(&env)),
        Err(Ok(Error::ClaimProofInvalid))
    );
    assert!(!client.has_badge(&event_id, &attendee));
}

#[test]
fn a_proof_from_another_tree_is_refused() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let now = env.ledger().timestamp();
    let (root, _) = merkle_tree(&env, &[0x11, 0x22]);
    let (_, other_entries) = merkle_tree(&env, &[0x91, 0x92]);
    let event_id = client.create_event(
        &organizer,
        &synthetic_hash(&env, 0xA1),
        &root,
        &100,
        &(now + DAY_SECONDS),
    );

    let (other_leaf, other_proof) = other_entries.get(0).unwrap();
    assert_eq!(
        client.try_claim(&event_id, &attendee, &other_leaf, &other_proof),
        Err(Ok(Error::ClaimProofInvalid))
    );
    assert!(!client.has_badge(&event_id, &attendee));
}

#[test]
fn a_proof_longer_than_the_depth_bound_is_refused() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    let event_id = setup_event(&env, &client, &organizer);

    // Longer than any tree this contract can hold, so it is refused before a
    // single hash is computed.
    let mut proof = Vec::new(&env);
    for _ in 0..(MAX_PROOF_DEPTH + 1) {
        proof.push_back(synthetic_hash(&env, 0x5A));
    }

    assert_eq!(
        client.try_claim(&event_id, &attendee, &setup_claim_code_hash(&env), &proof),
        Err(Ok(Error::ClaimProofInvalid))
    );
}

#[test]
fn proof_depth_covers_the_maximum_claims() {
    assert!(
        (1u64 << MAX_PROOF_DEPTH) >= u64::from(MAX_CLAIMS_PER_EVENT),
        "a tree over {} leaves is deeper than MAX_PROOF_DEPTH of {}",
        MAX_CLAIMS_PER_EVENT,
        MAX_PROOF_DEPTH
    );
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
        &setup_claim_code_hash(&env),
        &Vec::new(&env),
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
        &setup_claim_code_hash(&env),
        &Vec::new(&env),
    );
    client.claim(
        &event_id,
        &attendee,
        &setup_claim_code_hash(&env),
        &Vec::new(&env),
    );

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
        &setup_claim_code_hash(&env),
        &Vec::new(&env),
    );
    client.claim(
        &event_id,
        &attendee,
        &setup_claim_code_hash(&env),
        &Vec::new(&env),
    );

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

    client.claim(&event_id, &attendee, &hash_of(&env, &code), &Vec::new(&env));
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
