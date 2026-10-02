# 0001. NFT approach: OpenZeppelin vs plain soroban-sdk 28

Date: 2026-10-02
Status: accepted

## Context

Playbook v3 section 9 scopes v0: organizers create events and attendees claim
**non-transferable** attendance badges, and "transfers must always fail". It
directs us to read the OpenZeppelin non-fungible docs and record the approach
here, including how badges are made non-transferable. This repo resolves
`soroban-sdk 28.0.0` and builds with Stellar CLI 28.1.0. The sibling
`schoolfees-contracts` decision 0001 (2026-09-30) rejected the OpenZeppelin
Stellar crates because the stable line required `soroban-sdk ^26` and could
not be resolved; this decision re-checks that today against the live index
rather than assuming it still holds.

## What was checked (2026-10-02, real sources)

- **Published versions, from the crates.io sparse index** (`index.crates.io/st/el/stellar-tokens`):
  `stellar-tokens` and `stellar-access` end at **0.7.2** (2026). There is no
  0.8 or release-candidate line published as of today.
- **SDK requirement of 0.7.2, from the index metadata and the extracted
  crate source**: `soroban-sdk ^26.1.0` (earlier lines: 0.6.0 → `^23.4.0`,
  0.7.0/0.7.1 → `^25.3.0`). No published line targets SDK 27 or 28.
- **Resolution probe** (scratch crate: `soroban-sdk = "28"` plus
  `stellar-tokens = "0.7.2"`, `cargo tree` exit 0): cargo **does not fail**
  the build. It resolves **two SDK majors side by side** — our contract on
  `soroban-sdk 28.0.0` and OpenZeppelin's code on `soroban-sdk 26.1.1`. This
  refines the schoolfees finding: the conflict is not a hard resolution
  failure but a dual-SDK build. Adopting the crate would still mean writing
  our contract's types against one SDK while calling traits implemented for
  another, and `wasm32v1-none` would carry both majors.
- **Trait surface, from the extracted `stellar-tokens 0.7.2` source**
  (`src/non_fungible/`): `pub trait NonFungibleToken` with `transfer` and
  `transfer_from` as **inherent trait methods** (not optional extensions),
  plus an approvals/`authorize` model and an overrides mechanism designed to
  replace default behavior. Transferability is the design's default state;
  "cannot ever transfer" is not a supported configuration.
- **SDK 28 APIs, from the installed `soroban-sdk-28.0.0` source**: the
  `#[contractevent]` macro (soroban-sdk-macros 28.0.0), `Event::publish(&env)`
  (`src/events.rs`), `env.crypto().sha256(&Bytes) -> Hash<32>`
  (`src/crypto.rs`), and `Persistent::extend_ttl(key, threshold, extend_to)`
  / `Instance::extend_ttl(threshold, extend_to)` (`src/storage.rs`).

## Decision

- **Build badges as a purpose-written minimal claim registry on plain
  `soroban-sdk 28`. No OpenZeppelin dependency, and no crate dependency at
  all beyond `soroban-sdk`.**
- **How badges are non-transferable:** the contract has **no transfer,
  approve or operator entrypoint at all**. A badge is one persistent entry
  per `(event_id, attendee)` recording the issuer, the claim time and the
  event; ownership cannot be moved because nothing can move it — the only
  mutating entrypoints are `claim`, `award` and `revoke`, each guarded by
  the participant's or organizer's own `require_auth`. This is stronger than
  "override transfer to revert": there is no transfer path in the code.
- Events (`event_created`, `badge_claimed`, `badge_awarded`, `badge_revoked`)
  are `#[contractevent]` types documented in `docs/events.md`.
- `docs/decisions/README.md` lists this decision for future contributors.

## Consequences

- Easy: one SDK major in the build; a dependency surface of exactly one
  crate; storage and TTL rules kept in our own hands (TTL computed from each
  event's `closes_at` deadline); no mapping our state model onto
  transferable-token idioms.
- Hard / accepted: **no OpenZeppelin audit claim** for this contract — the
  badge logic is ours, unreviewed, and the README says so. We forgo
  OZ's enumerable/extensions tooling; `badges_of` is our own bounded lookup.
- What would change our mind: an **audited** OpenZeppelin Stellar line
  targeting `soroban-sdk ^28` with a non-transferable (soulbound-style)
  configuration; a second human reviewer recommending it; or a deliberate
  SDK move. Re-evaluate before any funded or public event uses this
  contract on testnet with strangers.
