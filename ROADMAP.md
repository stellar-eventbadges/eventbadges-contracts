# Roadmap

What is next for `eventbadges-contracts`, in order. Anything not listed as done is
**not implemented**.

## Status

- [x] Repository governance: AGENTS.md, CONTRIBUTING.md, ROADMAP.md, LICENSE,
      .gitignore, .gitattributes (2026-10-01).
- [x] v0 contract from the project's playbook section (2026-10-02): the
      six entrypoints, ranged error codes, four documented events, 24 tests
      including one error-path test per variant, all six local checks green
      (`cargo fmt --check`, clippy `-D warnings`, `cargo test`, `node
      --test`, `node scripts/check-errors.mjs`, `stellar contract build`).

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
      check-errors, `stellar contract build` (CLI v28.1.0). Written on
      2026-10-02; proves itself on GitHub on the next push.

## Deliberately unimplemented (from playbook section 9)

Listed there as the v0 boundary; each will get a draft issue when the v0
contract lands:

- Unique per-attendee claim codes (research Merkle proofs first).
- Badge metadata and images following the OpenZeppelin metadata approach.
- Batch awarding.
- Event series and streak badges.
- Pagination for `badges_of`.

## Decisions needed from Tim

1. **Build standard — decided (2026-10-02).** v3 section 9 is the scope
   authority for what the contract does; v4 plus the schoolfees repos are
   the standard for how it is built (doc set, AGENTS.md, CI, checkers).

## Explicitly out of scope

Mainnet deployment. Anything the v0 design does not ask for.
