# Roadmap

What is next for `eventbadges-contracts`, in order. Anything not listed as done is
**not implemented**.

## Status

- [x] Repository governance: AGENTS.md, CONTRIBUTING.md, ROADMAP.md, LICENSE,
      .gitignore, .gitattributes (2026-10-01).
- [ ] v0 contract from the project's playbook section — **blocked on the
      playbook file, see below.**

## Next

- [ ] v0 contract, scoped by playbook section 9 when Tim supplies
      `~/Desktop/Drips/_reference/playbooks/STELLAR-BUILD-PLAYBOOK-v3.md`.
      Planned per the program stack: thin `lib.rs`; `types.rs` (error enum,
      stored types, events); `storage.rs`; `error_paths.rs` with one test per
      variant; `test.rs` lifecycle tests; `ERRORS.md` +
      `scripts/check-errors.mjs` and its tests; rust-toolchain pinned to
      `wasm32v1-none`; release profile with `overflow-checks = true`.
- [ ] CI (`contract.yml`): fmt, clippy -D warnings, cargo test, node --test
      scripts/, check-errors, `stellar contract build` (CLI v28.1.0). Lands
      with the first code that can pass it.

## Blocked on the playbook

The project's v0 scope is defined in the playbook, which was not found on this
machine at Session 0. Until Tim supplies it, no feature scope is invented here.

## Decisions needed from Tim

1. **Playbook location.** Provide
   `~/Desktop/Drips/_reference/playbooks/STELLAR-BUILD-PLAYBOOK-v3.md` (or the
   correct path) so section 9 can scope v0.

## Explicitly out of scope

Mainnet deployment. Anything the v0 design does not ask for.
