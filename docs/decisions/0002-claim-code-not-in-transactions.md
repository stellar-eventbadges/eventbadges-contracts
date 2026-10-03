# 0002. Keep the raw claim code out of the claim transaction

Date: 2026-10-03
Status: proposed — not implemented, awaiting a maintainer decision

## Context

`claim` takes the attendee's claim code as a `Bytes` argument and hashes it
on-chain:

```rust
pub fn claim(env: Env, event_id: u64, attendee: Address, claim_code: Bytes) -> Result<(), Error>
```

```rust
let digest = env.crypto().sha256(claim_code);
if BytesN::from(digest) != event.claim_code_hash {
    return Err(Error::ClaimCodeMismatch);
}
```

The code therefore rides inside the signed transaction, in the clear, in a
public mempool and then in the permanent ledger. The app sends the raw bytes
(`prepareClaim` in `eventbadges-app/src/lib/contract.ts`), and the docs book
currently claims the opposite — that "the chain never holds the claim code".
Writing the attendee privacy notice (`eventbadges-app/docs/attendee-notice.md`)
made the contradiction unavoidable: the notice exists to tell attendees the
code is published, so it had to be said.

Nothing is deployed, so any fix is free of migration cost. The question is
only which one.

## The problem is two problems, and the roadmap answers the wrong one

The ROADMAP lists "per-attendee Merkle claim codes" as the fix. Merkle fixes
**sharing** — each attendee holds their own code, so one person's leak does not
let anyone else take a slot. It does **not** fix **publicity**: under Merkle
the attendee still presents their code in the transaction, so the raw secret
still lands in the mempool. If the question is "keep the raw code out of public
transaction arguments", Merkle is the wrong tool.

They are independent, and each is fixed differently.

| Defect | Who is harmed | Fix |
|---|---|---|
| **Publicity** — the code is in the transaction | the attendee, and every future attendee | present the digest instead (§Decision) |
| **Sharing** — one code per event, so any holder may claim, and may front-run | the organizer's cap and fairness | per-attendee Merkle codes (not proposed here) |

This decision takes **publicity** only, and takes it completely.

## What was checked

- **`claim_code_hash` is already public.** `Event` stores it, and `get_event`
  returns the whole record with no `require_auth` (no auth call exists in
  `get_event` or `load_event` in `src/badges.rs`). So `SHA256(code)` is
  readable by anyone, for any event, from the moment it is created — today,
  before this decision does anything. Publishing the digest in a transaction
  therefore reveals **no value that is not already public**.
- **The digest is safe to publish.** SHA-256 is preimage-resistant; the
  digest does not yield the code. It is the same commitment the contract
  already stores and already returns.
- **The client can compute it already.**
  `hashClaimCode` in `eventbadges-app/src/lib/claimCode.ts` runs
  `crypto.subtle.digest('SHA-256', ...)` on the pasted code, and
  `claimCode.test.ts` cross-checks it against Node's own SHA-256. ADR 0002
  already established that the browser and the contract agree.
- **No migration.** Nothing is deployed; there is no state to migrate and no
  deployed contract id to keep compatible.
- **No error churn.** The failure condition is unchanged — a presented value
  that is not the stored one still returns `ClaimCodeMismatch` — so
  `ERRORS.md`, `src/error_paths.rs` and the app's vendored
  `docs/contract-errors.md` are all untouched.

### Options considered and rejected

- **Merkle proofs.** Solves sharing, not publicity. Correctly listed as
  separate future work; it is not this fix.
- **Commit–reveal.** The reveal is the leak. It only helps if the reveal
  happens off-chain and someone attests on-chain — which is `award`.
- **A zero-knowledge proof of preimage knowledge.** Genuinely hides the
  preimage, but it needs verification, and Soroban's `env.crypto()` offers
  hashing and BLS operations rather than SNARK verification. Verifying
  off-chain reintroduces a trusted party, and a trusted party that attests
  "this address may have a badge" is exactly what `award` already is, with
  more machinery. Collapses into `award`.
- **Private or encrypted transactions.** Not available on public Stellar
  networks. Not an option today at any cost.
- **Hiding `claim_code_hash` from `get_event`.** Would genuinely help the
  *sharing* defect, and is worth considering separately. It is not compatible
  with this decision being merely neutral: if the digest later stops being
  public, this design still holds, because it never puts anything secret in
  the transaction.

## Decision

**`claim` takes the claim code's SHA-256, not the claim code.** The type
narrows from `Bytes` to `BytesN<32>`, and the comparison is direct instead of
hashed:

```rust
pub fn claim(
    env: Env,
    event_id: u64,
    attendee: Address,
    claim_code_hash: BytesN<32>,
) -> Result<(), Error> {
    attendee.require_auth();

    let mut event = load_event(env, event_id)?;
    check_event_open(&event)?;
    if env.ledger().timestamp() > event.closes_at {
        return Err(Error::EventClosed);
    }

    let badge_key = DataKey::Badge(event_id, attendee.clone());
    if env.storage().persistent().has(&badge_key) {
        return Err(Error::AlreadyHeld);
    }

    if claim_code_hash != event.claim_code_hash {
        return Err(Error::ClaimCodeMismatch);
    }
    // ... unchanged: count, issue_badge, BadgeClaimed publish
}
```

The check order is untouched — auth, exists, cap, deadline, already-held, code —
and `env.crypto()` is no longer called at claim time.

Consequences of the type change, all of them desirable:

- the ABI rejects any length other than 32 bytes, so a malformed code is not a
  runtime failure;
- `Bytes` appears in `src/badges.rs` and `src/lib.rs` only as the import and as
  this one parameter, so both imports shrink;
- no new storage key, no new event, no new error variant, no change to
  `docs/events.md`;
- the claim transaction stops being a place where a reusable secret is
  published, which is what makes the privacy notice's second section
  unnecessary.

### The app side, when this is implemented

Three lines, and the helper already exists:

- `prepareClaim` sends `bytes32ToScVal(digest)` instead of
  `nativeBytesToScVal(hexToBytes(code))`;
- `AttendeePage` awaits `hashClaimCode` and passes the digest — the raw code is
  then discarded from the form state as it already is after a successful claim;
- `docs/claim-codes.md` in this repo changes: the Stellar CLI path now passes
  `sha256(code)`, not the code, and the "never uploaded" wording needs to say
  why it is now true rather than asserting it.

### What this does **not** fix

Stated plainly, because the notice still has to be honest:

- **Sharing and front-running remain.** The digest is a bearer credential. It
  is already public today via `get_event`, so anyone can read it and claim a
  slot against the cap. This decision does not make the code secret — it stops
  the transaction from pretending to be where the leak happens.
- **A leaked organizer-held code still lets anyone claim** until the cap fills
  or the window closes.
- **`claim_code_hash` remains readable** through `get_event`. Hiding it is a
  separate decision with its own trade-offs.

### The attendee-facing message changes

The drafted notice says the code is published in the transaction. After this
change that sentence is wrong, and the honest replacement is different rather
than merely softer: the event's claim digest is public on the network, so the
code is not secret to anyone who reads the chain, and a code shared too widely
can be used to take a slot. `eventbadges-app/docs/attendee-notice.md`,
`docs/claim-codes.md`, the app README and the docs book's privacy page all need
updating in the same change, or they become the new drift.

## Consequences

- **Easy:** the leak the notice exists to disclose disappears, in one
  comparison, with no new state and no migration. The change is small enough to
  review line by line, which matters for unaudited code.
- **Easy:** the app already has the exact helper this needs, cross-checked
  against Node's SHA-256, so the client side is not new code.
- **Hard / accepted:** `claim`'s signature changes, so any hand-written
  integration — the Stellar CLI path in `docs/claim-codes.md`, a future
  integrator — must hash first. v0 is undeployed and has no users, so this is
  cheap now and expensive later, which is the argument for doing it now.
- **Hard / accepted:** the tests change shape. Sixteen `claim` invocations across
  `src/test.rs` (ten) and `src/error_paths.rs` (six) pass
  `synthetic_code(&env, 0xC7)` today and will pass a `BytesN<32>` digest
  instead; `mock_attendee_auth_for_claim` takes the new type at its two call
  sites in `src/test.rs`, and `hash_of` stays only for `setup_event`.
- **Unchanged:** the sharing defect stays open and still needs per-attendee
  Merkle codes. This decision makes the privacy notice shorter; it does not
  make the flow fair.
- **What would change our mind:** evidence that a claim transaction's contents
  are handled by something that treats the digest as confidential — that
  would mean the digest is a secret after all, which contradicts `get_event`,
  and would need `get_event` re-examined first. Or a decision to implement
  per-attendee Merkle codes instead, in which case this should be sequenced
  with them rather than shipped alone.
