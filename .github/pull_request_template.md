# Change

Describe the problem, resulting behavior, and linked issue or draft.

# Scope and acceptance criteria

List what this change covers and the checkable outcomes. Explain new dependencies when applicable.

# Validation

Record the result of each relevant command below, or explain why it was not run.

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
node --test
node scripts/check-errors.mjs
node scripts/check-events.mjs
stellar contract build
```

# Review checklist

- [ ] I read AGENTS.md and CONTRIBUTING.md.
- [ ] The diff contains no secret material, .env contents, or personal data; examples are synthetic.
- [ ] Testnet-only status and limitations remain accurate; no deployment or pilot evidence is invented.
- [ ] For interface, errors, events, or storage changes, I updated the relevant documentation and regression coverage.
- [ ] I described remaining risks and any checks not run.
