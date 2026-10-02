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
