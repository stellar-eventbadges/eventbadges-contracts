<!-- project-brand -->
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="brand/logo-dark.svg">
  <img src="brand/logo.svg" alt="EventBadges" height="72">
</picture>

# eventbadges — contracts

Soroban contract for **eventbadges**: organizers create events and attendees
claim a **non-transferable** attendance badge. Part of a three-repo project
with `eventbadges-app` (web app) and `eventbadges-docs` (mdBook book).

**Status: v0 contract built and locally verified; synthetic testnet demonstration deployed.**
See the [verified deployment record](docs/TESTNET_DEMONSTRATION.md). No real pilot,
audit or browser-wallet business flow has been completed.

## What the contract does

- `create_event(organizer, name_hash, claim_root, max_claims, closes_at)`: the
  organizer records an event with an opaque name hash, a Merkle root over its
  attendees' claim-code hashes, a badge cap (1–10,000) and a claim deadline.
  Organizer-authorized. Returns the event id.
- `claim(event_id, attendee, leaf_hash, proof)`: the attendee claims their own
  badge by presenting the SHA-256 of **their** claim code and the proof that
  it belongs to the event's root. The raw code is never sent to the chain
  ([decisions/0002](docs/decisions/0002-claim-code-not-in-transactions.md)),
  the leaf is not derivable from the public root, and it is spent on the first
  claim so one code takes one place
  ([decisions/0003](docs/decisions/0003-per-attendee-claim-codes.md)). One
  badge per attendee per event; fails after `closes_at` or when the cap is
  reached.
- `award(event_id, attendee)`: the organizer issues a badge directly, for
  attendees who cannot claim. Same window and cap rules.
- `revoke(event_id, attendee)`: the organizer removes a badge. Allowed at any
  time — a badge issued in error must be removable after the window closes.
- `get_event(event_id)`, `has_badge(event_id, attendee)`,
  `badges_of(event_id, attendee)` (bounded: at most one badge per attendee).
- **There is no transfer, approve or operator entrypoint.** A badge cannot
  move after issuance because nothing can move it. The reasoning and the
  alternatives are recorded in
  [docs/decisions/0001-nft-approach.md](docs/decisions/0001-nft-approach.md).

## Privacy

No names, emails or personal IDs touch the chain. `name_hash` and `claim_root`
are opaque hashes; claim codes are random secrets generated off-chain, and the
contract receives only one leaf of their tree per claim — never a code, and
never anything from which another attendee's code could be derived (see
[docs/claim-codes.md](docs/claim-codes.md)). Test fixtures use synthetic bytes
only.

## Structure

Standard layout, shared with this program's other contract repos: thin
[`src/lib.rs`](src/lib.rs) (`#[contractimpl]` delegation only),
[`src/types.rs`](src/types.rs) (error enum in numbered ranges, stored types,
`#[contractevent]` events), [`src/storage.rs`](src/storage.rs) (keys and TTL
helpers computed from each event's real `closes_at` deadline),
[`src/badges.rs`](src/badges.rs) (logic), [`src/error_paths.rs`](src/error_paths.rs)
(exactly one test per error variant), [`src/test.rs`](src/test.rs)
(lifecycle, auth and event-layout tests).

- [`ERRORS.md`](ERRORS.md) — one row per error variant; the user-facing
  wording there is the source of truth for the app.
- [`docs/events.md`](docs/events.md) — topic and data layout of all four
  events, asserted exactly in tests.
- [`scripts/check-errors.mjs`](scripts/check-errors.mjs) and
  [`scripts/check-events.mjs`](scripts/check-events.mjs) — fail when
  `ERRORS.md` or `docs/events.md` drifts from `src/types.rs`; `node --test`
  covers both checkers.
- [`scripts/deploy-testnet.sh`](scripts/deploy-testnet.sh) — **written, never
  run.** Deploying is Tim's step.

## Checks

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test              # 35 tests: 9 error paths + 26 lifecycle/auth/merkle
node --test             # 23 tests over the ERRORS.md and events checkers
node scripts/check-errors.mjs
node scripts/check-events.mjs
stellar contract build  # wasm32v1-none; verified with Stellar CLI 28.1.0
```

Toolchain: `soroban-sdk = "28"` (28.0.0 in `Cargo.lock`), Rust stable
(1.84.0+), target `wasm32v1-none`, release profile with
`overflow-checks = true`.

## Deliberately not built

Badge metadata and images, batch awarding, event series and streak badges,
pagination for `badges_of`. Unique per-attendee claim codes via Merkle proofs
are implemented; address-bound leaves remain a product decision in ADR 0003.
Each has a draft under [docs/issue-drafts/](docs/issue-drafts/); the list is
mirrored in [ROADMAP.md](ROADMAP.md). Testnet only — no mainnet, ever, in this
phase.

## License

MIT — see [LICENSE](LICENSE).
