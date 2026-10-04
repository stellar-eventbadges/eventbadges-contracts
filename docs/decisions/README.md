# Design decisions

One file per decision that a future contributor should not have to re-derive:
`NNNN-short-title.md`. Keep each one short — context, decision, consequences —
and dated.

The convention is a lightweight ADR:

```markdown
# NNNN. Short title

Date: YYYY-MM-DD
Status: accepted | superseded by NNNN

## Context
What forced a choice, in two or three sentences.

## Decision
What was chosen, precisely. Pin versions where a library is involved.

## Consequences
What this makes easy, what it makes hard, and what would change our mind.
```

## Recorded so far

- **0001 — NFT approach: OpenZeppelin vs plain soroban-sdk 28** (*accepted*,
  2026-10-02): no OpenZeppelin dependency; badges are a purpose-written
  non-transferable claim registry on plain `soroban-sdk 28`, with no transfer,
  approve or operator entrypoint.
- **0002 — Keep the raw claim code out of the claim transaction** (*accepted*,
  2026-10-03, **implemented 2026-10-04**): `claim` takes the claim code's
  SHA-256 instead of the code itself. Separate from Merkle per-attendee codes,
  which fix sharing rather than the transaction's contents.
- **0003 — Per-attendee claim codes via Merkle proofs** (*accepted*,
  2026-10-04, **implemented 2026-10-04**): one leaf per attendee, a stored
  root, a sorted-pair tree and a spent-leaf record, so one code takes one
  place and the public root hands nobody the means to claim. Address-bound
  leaves are recorded as the rejected alternative and the next step.
