# AGENTS.md

Rules for any AI agent working in this repository (`{REPO}`). Read this file at the start of every task.

## Project context

`eventbadges` is a Stellar/Soroban project with three repos: `eventbadges-contracts` (Rust contract, this repo), `eventbadges-app` (web app) and `eventbadges-docs` (mdBook docs). It is built by one person, will be public and open to outside contributors. Testnet only, never mainnet.


**Never put attendee names, phone numbers, emails or IDs on-chain. Opaque references or hashes only.**

**Pilot honesty:** no `eventbadges` contract is deployed and no pilot has happened. Never invent users, partners, addresses, hashes or outcomes. Deploying is Tim's step, not an agent's.

## Source of truth

Read these before changing anything, in this order:

1. `README.md` — honest status and limitations.
2. `ROADMAP.md` — what v0 is and what is deliberately unimplemented.
3. `ERRORS.md` (once it exists) — one row per error variant; the "user-facing message" column is the single source of truth for every other repo, and its wording is never re-typed elsewhere.

System-level architecture will live in the docs repo; link to it, never restate it here.

## Commit rule

- One logical change per commit. Subject: `type: imperative summary`, 72 characters or fewer. Stage by explicit file name and read the staged diff before committing. NO Codebuff or co-author trailers. No history rewrites. No filler, empty or backdated commits. Commit counts are never a goal.

## Toolchain and structure

- soroban-sdk 28; Stellar CLI 28.1.0 (pinned in CI when CI lands).
- Structure once code exists: thin `lib.rs`; `types.rs` for the error enum, stored types and events; `storage.rs` for keys and TTL helpers; `error_paths.rs` with one test per error variant; `test.rs` for lifecycle tests. TTLs derived from real deadlines. No unwrap/expect outside tests. Tests at least as large as implementation.
- `ERRORS.md` plus `scripts/check-errors.mjs` and its tests keep the error table in sync.
- Checks before finishing any task: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `node --test scripts/`, `node scripts/check-errors.mjs`, `stellar contract build`. If cargo is not found, add `~/.cargo/bin` to PATH.
- Never use the old `soroban` CLI or `stellar contract test`.

## Collaboration rules

- Lead with the result or the next action; detail comes after.
- Call out incorrect assumptions plainly, in one sentence, and continue with what is true.
- Ask before anything destructive, legal, security-related, payment-related or irreversible; record high-stakes questions under "Decisions needed from Tim" in `ROADMAP.md` and carry on with the rest.
- Honest completion report: what was tested, what was not, any defect found.
- Do not invent requirements, and do not add scope beyond the task.
