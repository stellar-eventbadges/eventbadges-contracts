# Roadmap

What is next for `eventbadges-contracts`, in order. Anything not listed as done is
**not implemented**.

## Status

- [x] Repository governance: AGENTS.md, CONTRIBUTING.md, ROADMAP.md, LICENSE,
      .gitignore, .gitattributes (2026-10-01).
- [ ] v0 contract from the project's playbook section.

## Next

- [ ] v0 contract from `STELLAR-BUILD-PLAYBOOK-v3.md` section 9
      (present in `~/Desktop/Drips/_reference/playbooks/`, confirmed
      2026-10-02). v0 scope from that section: Soroban contract where
      organizers create events and attendees claim a non-transferable
      attendance badge. Entrypoints: `create_event`, `claim`, `award`,
      `revoke`, `has_badge`, `badges_of` (bounded), `get_event`; transfers
      must always fail. `name_hash` and `claim_code_hash` stay hashes; no
      personal data on-chain. Planned per the program stack: thin `lib.rs`;
      `types.rs` (error enum, stored types, events); `storage.rs`;
      `error_paths.rs` with one test per variant; `test.rs` lifecycle tests;
      `ERRORS.md` + `scripts/check-errors.mjs` and its tests;
      rust-toolchain pinned to `wasm32v1-none`; release profile with
      `overflow-checks = true`.
- [ ] CI (`contract.yml`): fmt, clippy -D warnings, cargo test, node --test
      scripts/, check-errors, `stellar contract build` (CLI v28.1.0). Lands
      with the first code that can pass it.

## Deliberately unimplemented (from playbook section 9)

Listed there as the v0 boundary; each will get a draft issue when the v0
contract lands:

- Unique per-attendee claim codes (research Merkle proofs first).
- Badge metadata and images following the OpenZeppelin metadata approach.
- Batch awarding.
- Event series and streak badges.
- Pagination for `badges_of`.

## Decisions needed from Tim

1. **Playbook v3 section 9 vs v4 doc set.** v3 section 9 defines the v0
   contract scope and is authoritative for it; v4 adds the standard doc set
   and error-sync checker on top. Build v0 from v3 section 9 plus the v4
   layer, or wait for Tim's call.

## Explicitly out of scope

Mainnet deployment. Anything the v0 design does not ask for.
