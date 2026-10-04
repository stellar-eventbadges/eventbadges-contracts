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
      must always fail. `name_hash` and the claim commitment (`claim_root`
      since [ADR 0003](docs/decisions/0003-per-attendee-claim-codes.md)) stay
      hashes; no personal data on-chain. Planned per the program stack: thin `lib.rs`;
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

- ~~Unique per-attendee claim codes (research Merkle proofs first).~~ **Built
  2026-10-04** — [ADR 0003](docs/decisions/0003-per-attendee-claim-codes.md)
  and [draft 01](docs/issue-drafts/01-unique-claim-codes-via-merkle-proofs.md):
  one leaf per attendee, a stored root, and a spent-leaf record so one code
  takes one place. The leaf formula is still unbound, so a leaked code can be
  used by whoever holds it first.
- Address-bound leaves (`SHA-256(address || code)`), which would make a leaked
  code useless to anyone else and remove that last exposure. Rejected for now
  in ADR 0003: it needs every attendee's address before the event is created,
  and the documented flow hands codes out at the door. It waits on a product
  decision about pre-registration, not on any contract work.
- Badge metadata and images following the OpenZeppelin metadata approach.
- Batch awarding.
- Event series and streak badges.
- Pagination for `badges_of`.

## Decisions needed from Tim

1. **Build standard — decided (2026-10-02).** v3 section 9 is the scope
   authority for what the contract does; v4 plus the schoolfees repos are
   the standard for how it is built (doc set, AGENTS.md, CI, checkers).

### Legal review of the on-chain privacy model — needed before any pilot with real attendees

The privacy page in the docs book raises questions only a human can answer.
They are collected here as a checklist so the decision is concrete. Working
through it produces a written position, not a compliance certificate, and is
not legal advice. Source: [Privacy: what is on-chain](https://github.com/stellar-eventbadges/eventbadges-docs/blob/main/src/privacy.md);
the same checklist is recorded in the ROADMAPs of all three eventbadges
repos, so they stay in sync.

- [ ] **Applicable law.** Which regimes apply to a pilot — GDPR (any EU/EEA
      attendee?), NDPR/NDPA (Nigeria), other local law — and does the answer
      change when the pilot group crosses a border?
- [ ] **Who is the controller?** For an attendance record on a public
      ledger: the organizer (they choose the event and who gets a badge),
      the project, both, or neither? Write the position down before
      recruiting anyone.
- [ ] **Lawful basis.** What basis covers putting a wallet address and an
      attendance fact on an immutable public ledger — consent, legitimate
      interest, something else? Can consent be freely given when nothing can
      ever be deleted, and what must the claim flow say before the wallet
      signs?
- [ ] **Erasure vs. immutability.** `revoke` removes the badge, but the
      `badge_claimed` event stays on-chain forever. Is that defensible under
      erasure and objection rights? If not, is the mitigation — no personal
      data on-chain, fresh-address guidance, declining unsuitable pilots —
      enough, and who signs off?
- [ ] **Are the hashes personal data?** `name_hash` is low-entropy and
      brute-forceable; an address becomes identifying the moment it is
      linked off-chain. Does "it is only a hash" or "pseudonymous" actually
      hold, or must both be treated as personal data?
- [ ] **Children.** The privacy page says events involving minors must never
      be pointed at this system without the organizer fully understanding
      everything is public. Make it operational: is "no under-18 events in a
      pilot" a hard rule, who checks, and what does the organizer attest to?
- [ ] **What attendees are told — copy drafted (2026-10-03), decisions
      open.** The text is answered in
      [`docs/attendee-notice.md` in the app repo](https://github.com/stellar-eventbadges/eventbadges-app/blob/main/docs/attendee-notice.md),
      with every line traced to the code that makes it true: a short
      on-screen version with a required acknowledgement, a full version one
      tap away, and the paragraph nobody had written — **the claim code is
      published inside the `claim` transaction and is permanent**, which the
      docs book currently denies. **Partly delivered 2026-10-03:** the copy
      exists and is implemented on the claim screen
      ([draft 11 in the app repo](https://github.com/stellar-eventbadges/eventbadges-app/blob/main/docs/issue-drafts/11-attendee-privacy-notice.md)),
      so the "what must a person be told" half of the question is answered.
      Four decisions remain, and none of them is a copy question: who
      delivers it (the app, the organizer, or both — recommended both, with
      the organizer pointing at this text rather than writing their own);
      whether a missing notice blocks the first pilot (recommended yes — a
      claim is an irreversible public write of a linkable identifier, and the
      claim-code paragraph cannot be consented to if it was never disclosed);
      whether to change the contract instead of disclosing the exposure
      (bigger than any one repo should decide alone); and whether an
      acknowledgement recorded nowhere should be described as consent.
      Shipping the notice settles none of them — the gate is `aria-disabled`
      plus a refusal in `submitClaim`, recorded nowhere, which is deliberate
      rather than a stopgap. **Implemented 2026-10-04:**
      [ADR 0002](docs/decisions/0002-claim-code-not-in-transactions.md) landed
      in the contracts repo and the app together — `claim` now takes the code's
      SHA-256 instead of the code, so the paragraph above is obsolete by
      construction and the notice was rewritten in the same change. That
      answers the "change the contract instead of disclosing" decision on this
      list; the other three still stand.
- [ ] **Third parties in the path.** The app sends addresses — and the claim
      code's digest, which is already public, inside the `claim` transaction —
      to the Stellar RPC endpoint, and explorers index events. How are RPC
      operators and explorers characterised (processor, independent
      controller), and can a pilot simply accept the public testnet RPC?
- [ ] **Off-chain handling by organizers.** Claim codes, attendee lists and
      check-in spreadsheets never touch the chain but stay with the
      organizer. Does the project owe organizers written data-handling
      guidance (what to keep, what to delete, how to share codes), and is
      that guidance a precondition for the first pilot?
- [ ] **The pilot's own records.** Pilot notes name participants only at
      their chosen level of detail and link real transactions. What consent
      does that require, and how long are pilot notes kept?

## Explicitly out of scope

Mainnet deployment. Anything the v0 design does not ask for.
