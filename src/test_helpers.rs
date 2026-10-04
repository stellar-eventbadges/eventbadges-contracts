#![cfg(test)]

//! Shared setup helpers for `test.rs` and `error_paths.rs`.

use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Ledger as _, MockAuth, MockAuthInvoke,
    },
    Address, Bytes, BytesN, Env, IntoVal, Val, Vec,
};

use crate::{storage::SECONDS_PER_LEDGER, Contract, ContractClient};

/// Seconds in a day, for readable test deadlines.
pub const DAY_SECONDS: u64 = 24 * 60 * 60;

/// Registers the contract and returns its address and a client for it.
pub fn setup(env: &Env) -> (Address, ContractClient<'_>) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    (contract_id, client)
}

/// Creates an event with a default cap of 100 and a deadline one day out.
/// The claim root is `SHA-256(32 x 0xC7)` — a one-leaf tree, so the leaf *is*
/// the root and its proof is empty.
pub fn setup_event(env: &Env, client: &ContractClient<'_>, organizer: &Address) -> u64 {
    let now = env.ledger().timestamp();
    client.create_event(
        organizer,
        &synthetic_hash(env, 0xA1),
        &setup_claim_code_hash(env),
        &100,
        &(now + DAY_SECONDS),
    )
}

/// The leaf — and, for the one-attendee event `setup_event` creates, the root —
/// that a claim against that event presents. Its proof is empty.
pub fn setup_claim_code_hash(env: &Env) -> BytesN<32> {
    hash_of(env, &synthetic_code(env, 0xC7))
}

/// A synthetic 32-byte hash. Tests never use a value derived from a real
/// person.
pub fn synthetic_hash(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

/// A synthetic claim code: 32 bytes of the same seed byte. Random-looking
/// values in production; synthetic here.
pub fn synthetic_code(env: &Env, seed: u8) -> Bytes {
    Bytes::from_array(env, &[seed; 32])
}

/// Converts a value to a `Val` with the target type pinned, for building
/// expected event data maps (plain `into_val` is ambiguous there).
pub fn to_val<T: IntoVal<Env, Val>>(env: &Env, value: T) -> Val {
    value.into_val(env)
}

/// SHA-256 of `code`, using the env's real host crypto, matching what the
/// contract stores in `claim_code_hash`.
pub fn hash_of(env: &Env, code: &Bytes) -> BytesN<32> {
    env.crypto().sha256(code).into()
}

/// Moves the ledger forward by `seconds`, keeping sequence and timestamp in
/// step (at roughly 5 seconds per ledger).
pub fn advance_time(env: &Env, seconds: u64) {
    let ledgers = u32::try_from(seconds / SECONDS_PER_LEDGER).unwrap();
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + ledgers);
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + seconds);
}

/// Mocks only the organizer's signature for the `create_event` invocation.
pub fn mock_organizer_auth_for_create_event(
    env: &Env,
    contract_id: &Address,
    organizer: &Address,
    name_hash: &BytesN<32>,
    claim_code_hash: &BytesN<32>,
    max_claims: u32,
    closes_at: u64,
) {
    env.mock_auths(&[MockAuth {
        address: organizer,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "create_event",
            args: (
                organizer.clone(),
                name_hash.clone(),
                claim_code_hash.clone(),
                max_claims,
                closes_at,
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Mocks only the attendee's signature for one `claim` invocation.
pub fn mock_attendee_auth_for_claim(
    env: &Env,
    contract_id: &Address,
    attendee: &Address,
    event_id: u64,
    leaf_hash: &BytesN<32>,
    proof: &Vec<BytesN<32>>,
) {
    env.mock_auths(&[MockAuth {
        address: attendee,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "claim",
            args: (event_id, attendee.clone(), leaf_hash.clone(), proof.clone()).into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// One Merkle step, mirroring `hash_pair` in `src/badges.rs`: the two nodes are
/// sorted before hashing, so a proof is an unordered list of siblings.
pub fn merkle_pair(env: &Env, a: &BytesN<32>, b: &BytesN<32>) -> BytesN<32> {
    let (low, high) = if a <= b { (a, b) } else { (b, a) };
    let mut input = Bytes::from_array(env, &low.to_array());
    input.extend_from_array(&high.to_array());
    BytesN::from(env.crypto().sha256(&input))
}

/// Every level of a Merkle tree over `leaves`, leaf level first. An odd level
/// is closed by hashing its last node with itself, which is what keeps every
/// proof `ceil(log2(leaves))` long.
fn merkle_levels(env: &Env, leaves: &Vec<BytesN<32>>) -> Vec<Vec<BytesN<32>>> {
    let mut levels = Vec::new(env);
    levels.push_back(leaves.clone());

    while levels.get(levels.len() - 1).unwrap().len() > 1 {
        let current = levels.get(levels.len() - 1).unwrap();
        let mut next = Vec::new(env);
        let mut index = 0;
        while index < current.len() {
            let left = current.get(index).unwrap();
            let right = if index + 1 < current.len() {
                current.get(index + 1).unwrap()
            } else {
                left.clone()
            };
            next.push_back(merkle_pair(env, &left, &right));
            index += 2;
        }
        levels.push_back(next);
    }

    levels
}

/// The proof for the leaf at `index`: one sibling per level, where a lone node
/// acts as its own sibling.
fn merkle_proof(env: &Env, levels: &Vec<Vec<BytesN<32>>>, mut index: u32) -> Vec<BytesN<32>> {
    let mut proof = Vec::new(env);

    for depth in 0..levels.len() {
        let level = levels.get(depth).unwrap();
        if level.len() == 1 {
            break;
        }
        let sibling = index ^ 1;
        proof.push_back(if sibling < level.len() {
            level.get(sibling).unwrap()
        } else {
            level.get(index).unwrap()
        });
        index >>= 1;
    }

    proof
}

/// A ready-made event tree: `seeds` name one claim code per attendee, so the
/// leaf for each is `SHA-256(32 x seed)` — exactly `hashClaimCode`'s output in
/// the app. Returns the root and each attendee's `(leaf, proof)`, in seed
/// order.
pub fn merkle_tree(env: &Env, seeds: &[u8]) -> (BytesN<32>, Vec<(BytesN<32>, Vec<BytesN<32>>)>) {
    let mut leaves = Vec::new(env);
    for seed in seeds {
        leaves.push_back(hash_of(env, &synthetic_code(env, *seed)));
    }

    let levels = merkle_levels(env, &leaves);
    let root = levels.get(levels.len() - 1).unwrap().get(0).unwrap();

    let mut entries = Vec::new(env);
    for index in 0..leaves.len() {
        entries.push_back((
            leaves.get(index).unwrap(),
            merkle_proof(env, &levels, index),
        ));
    }

    (root, entries)
}

/// Remaining TTL of a persistent entry, read inside the contract's context.
pub fn persistent_ttl(env: &Env, contract_id: &Address, key: &crate::storage::DataKey) -> u32 {
    env.as_contract(contract_id, || env.storage().persistent().get_ttl(key))
}

/// Remaining TTL of the contract instance.
pub fn instance_ttl(env: &Env, contract_id: &Address) -> u32 {
    env.as_contract(contract_id, || env.storage().instance().get_ttl())
}
