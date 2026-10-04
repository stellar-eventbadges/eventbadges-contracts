# AGENTS.md

Rules for any AI agent working in this repository. Read this file at the start of every task.

## Project context

`eventbadges` is a Stellar/Soroban project with three repos: `eventbadges-contracts` (Rust contract, this repo), `eventbadges-app` (web app) and `eventbadges-docs` (mdBook docs). It is built by one person, is public and will be open to outside contributors.

- **Testnet only.** No mainnet, ever, in this phase.
- **One builder.** There is no team; write for a solo maintainer.
- **Pilot users are real people**, not developers — community organizers and attendees.
- **Never put attendee names, phone numbers, emails or IDs on-chain. Opaque references or hashes only.**
- **No deployment until a real organizer or chapter has agreed to try the flow.** The pilot gate lives in `ROADMAP.md`.
- **Pilot honesty:** no `eventbadges` contract is deployed and no pilot has happened. Never invent users, partners, addresses, hashes or outcomes. Deploying is Tim's step, not an agent's.

## Source of truth

Read these before changing anything, in this order:

1. `README.md` — what works today, and the honest limitations.
2. `ROADMAP.md` — what v0 is, and what is deliberately unimplemented.
3. `docs/decisions/` — decisions already made, with their reasoning.
4. `ERRORS.md` (once it exists) — one row per error variant; the "user-facing message" column is the single source of truth for every other repo, and its wording is never re-typed elsewhere.
5. `docs/events.md` (once it exists) — one section per event the contract emits.

System-level architecture lives in the docs repo (`eventbadges-docs`); link to it, never restate it here.

## Collaboration rules

- **Lead with the result or the next action.** Say what happened or what you need first; detail comes after.
- **Call out incorrect assumptions plainly.** If a premise in the task is wrong, say so in one sentence and continue with what is true.
- **Ask before anything destructive, legal, security-related, payment-related or irreversible.** Do not guess on a high-stakes decision: record it under "Decisions needed from Tim" in `ROADMAP.md` and carry on with the rest.
- **Honest completion report.** Before saying done, state what you tested, what you did **not** test, and any defect you found. "It works" without evidence is a liability, not a signal.
- Do not invent requirements, and do not add scope beyond the task.

## Toolchain

Verified on this machine on 2026-10-02. Re-check against developers.stellar.org — do not trust version pins in other people's repos, including reference projects, without checking.

- Rust: stable **1.98.1** via rustup (docs require 1.84.0 or higher).
- Target: **`wasm32v1-none`** (`rustup target add wasm32v1-none`).
- **Windows host toolchain:** this machine has no MSVC linker; the default toolchain is `stable-x86_64-pc-windows-gnu` with a MinGW-w64 linker. `rust-toolchain.toml` deliberately does **not** pin a channel (see the comment in that file).
- Stellar CLI: **v28.1.0**. CI pins it with `uses: stellar/stellar-cli@v28.1.0`, so that is the version this project's build is verified against.
- SDK: `soroban-sdk = "28"` pinned in `Cargo.toml`; the resolved version is recorded in the committed `Cargo.lock`.

Commands (run from the repository root):

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
node --test
node scripts/check-errors.mjs
node scripts/check-events.mjs
stellar contract build
```

**NEVER use** the old `soroban` CLI, the `wasm32-unknown-unknown` target (unless a specific, stated reason requires dual-targeting — ask first), or `stellar contract test`.

## Contract structure (standard layout)

- `src/lib.rs`: `#[contract]`/`#[contractimpl]` only. Keep it thin; move logic into other modules.
- `src/types.rs`: the `#[contracterror] pub enum Error`, `#[contracttype]` structs, and `#[contractevent]` event types. Group error variants into numbered ranges by category (1-9 init/lookup, 10-29 lifecycle/timing, 30-49 validation/auth) with a comment stating each range, and keep the ranges reserved even while empty.
- `src/storage.rs`: storage keys, TTL constants, and `extend_ttl` helper functions. Compute TTL bumps from a real deadline where one exists (see the TTL rule below), not only a flat constant.
- `src/error_paths.rs`: exactly one test per `Error` variant, named `error_path_<variant_name_in_snake_case>`, that triggers the real failure path (not just constructs the error value).
- `src/test.rs`: happy-path and multi-step integration tests.
- `ERRORS.md` at the repo root: one row per `Error` variant. `scripts/check-errors.mjs` keeps it in sync with `types.rs` and CI fails when they drift.
- `docs/events.md`: one section per `#[contractevent]` type, with a `Topics` row and a `Data` row that each name every field as `` `field` (`Type`) ``. `scripts/check-events.mjs` keeps it in sync with `types.rs` — topics versus data map included — and CI fails when they drift.

## Contract rules (Soroban)

- `#![no_std]`; use `soroban-sdk` types only.
- Every function that acts on behalf of an address must call `require_auth` for that address.
- Return errors with the `#[contracterror]` enum described above; validation and auth failures must make the transaction fail. Never return `false` to signal a failed check.
- Storage: growing data gets one persistent entry per record. Instance storage is only for small contract-wide values. Extend TTL whenever an entry is read or written. Where a record has a natural deadline (an event's `closes_at`, a claim window), compute the TTL bump from that deadline plus a safety margin, not a single flat constant for everything. Read the State Archival and Contract Storage guides first.
- Use checked arithmetic. No unbounded loops or unbounded input lists; cap sizes and document the caps.
- Emit events for state changes using `#[contractevent]` types and document their layout in `docs/events.md` in the same commit; `node scripts/check-events.mjs` fails until the two agree.
- Tests must cover happy paths, every error path, and unauthorized callers. Aim for test code that is at least as large as implementation code; that ratio is a floor, not a target to pad past with trivial tests.
- Prefer audited libraries (OpenZeppelin's Stellar crates) over hand-written token, access-control or governance code. Evaluate before use, record the decision in `docs/decisions/`, and pin exact versions. For this repo that evaluation is `docs/decisions/0001-nft-approach.md` — non-transferable badges are custom logic here, not a copy of a transferable token standard.

## Documentation rules

- Every contracts repo ships `README.md`, `ERRORS.md`, `CONTRIBUTING.md`, `ROADMAP.md`.
- The docs repo (`eventbadges-docs`) ships, at minimum, `architecture.md`, `limitations.md`, `threat-model.md`, `pilot-playbook.md`, and one `pilots/{name}.md` per real pilot.
- Do not invent "investor-style" documents (funding asks, hiring plans, competitive positioning) unless the human explicitly asks for one — they do not fit a v0 pilot project and read as padding.
- `limitations.md` must be genuinely honest: state what is not proven, not what sounds acceptable. A limitations doc with nothing in it is a sign it was not written carefully.

## Privacy rules

- Never put personal data on-chain: no names, phone numbers, emails, attendee or member identifiers, or anything about children. Use opaque references or hashes of off-chain documents.
- Nothing that reaches the chain — argument names, event fields, error messages, test fixtures — should carry a real person's data. Event names, claim codes and metadata stay hashes or opaque values. Use synthetic values in tests.

## App rules

- Wallets sign; the app NEVER asks for, stores or logs a secret key or seed phrase.
- Always show a visible "TESTNET - no real money" banner and refuse other networks.
- Map contract errors to plain-language messages, using the `ERRORS.md` "user-facing message" column as the source of truth — do not invent different wording in the app.
- Show the transaction hash and an explorer link after each action.
- Mobile-first, accessible, no analytics or trackers, no backend in v0.
- Never hardcode contract ids, addresses or keys; read them from `.env` values the human provides.

## Rust and TypeScript rules

- No `unwrap()` or `expect()` outside tests unless you explain why it cannot fail.
- Include `rust-toolchain.toml`, `.gitignore` (`target/`, `node_modules/`, `.env`, `.stellar/`, `*.key`), README, CONTRIBUTING.md, ROADMAP.md, and CI.

## Commit rules

- One logical change per commit. Never bundle unrelated changes.
- Never create empty or filler commits.
- Every commit must pass this repository's checks (the commands listed under Toolchain above).
- Conventional format: `type: imperative summary`, where `type` is one of `feat`, `fix`, `docs`, `chore`, `test`, `refactor`, `style` or `perf`.
- Subject line: 72 characters or fewer, in the imperative mood. No trailing period.
- Stage files by explicit name. **Never** `git add -A` or `git add .`.
- Run `git status` and read the staged diff (`git diff --staged`) before every commit. Do not commit a file you did not intend to change.
- Never commit `.env` contents, key material, a secret, or a secret-looking string. If you see one in a diff, stop and say so.
- Do not rewrite history.
- Never add a "Generated with Codebuff" trailer or any co-author trailer to commit messages.

## Safety rules

- Testnet only. Never mainnet.
- NEVER read, print, log, commit, or ask for secret keys, seed phrases or `.env` contents. Scripts read identities from environment variables or `stellar keys`; any deploy script is written for the human to run, never by an agent.
- Do NOT deploy, push, change git remotes, create GitHub issues, install tools, run `sudo`, or pipe downloads into a shell. Write scripts and stop; the human runs them.
- Do not add dependencies without saying why, and check the crate or package (maintainer, recent releases) first.

## Truthfulness and evidence rules

- Never invent function names, flags, or crate or package APIs. If unsure, read developers.stellar.org, docs.rs or the installed package source. If you still cannot verify, write `TODO(verify)` and list it in your final summary.
- Docs must describe only what the code does. Anything not built is marked "Not implemented yet".
- Never invent contract addresses, transaction hashes, users, testers, quotes, or pilot outcomes. Evidence files (`docs/evidence/`, `docs/pilots/`) and the error-message table are written only from data the human provides or from code you can point to.
- When referencing a real third-party project as inspiration for structure (not code), never copy its name, branding, specific figures, or claims. Cite the pattern, not the source's content.

## Scope rules

- Build v0 only. Do NOT implement items listed under "Deliberately unimplemented" in `ROADMAP.md`; record them there. Those become contributor issues.
- Write each unimplemented item as a draft in `docs/issue-drafts/NN-title.md` using the template below. Do not create issues on GitHub.
- Do not add scope beyond what the task asks, even if a reference project does more. A working v0 with honest limitations beats a half-working v1.

## Issue draft template

```markdown
# Title (imperative, specific)
**Difficulty:** easy | medium | hard
**Labels:** good first issue | help wanted | area:<contracts|app|docs|ci>

## Problem
What is missing or wrong, and why it matters.

## Scope
What to change. What is explicitly out of scope.

## Acceptance criteria
- [ ] Checkable statements (tests pass, docs updated, behavior X).

## Where to start
Files or functions, and the docs to read.

## How to test
Exact commands.
```
