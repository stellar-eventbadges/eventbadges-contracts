# Contributing

Thanks for helping with `eventbadges`. **Read [AGENTS.md](AGENTS.md) first** — it is
the rulebook for this repository, for people and for AI agents alike. This page
is a shorter orientation.

## Before you change anything

- **Testnet only.** Never write anything that suggests mainnet use.
- **Nothing is deployed and no pilot has happened.** Do not describe a
  deployment, a contract id, a user, a tester or a result that does not exist.
- **No personal data, ever** — not in examples, not in tests, not in issue
  drafts. Use obvious placeholders.
- **Never commit `.env`**, a secret key or a seed phrase.

## Commits

- One logical change per commit; subject `type: imperative summary`, 72
  characters or fewer.
- Stage by explicit file name and read the staged diff before committing.
- No "Generated with" or co-author trailers of any kind.

## Checks to run before you push

Once this repo has code, its checks are listed in `AGENTS.md` and run in CI.
A change that breaks any of them is not ready. Until then, keep commits to
documentation and hygiene so the history stays honest.

## Contributor templates

The [bug report](.github/ISSUE_TEMPLATE/bug_report.yml),
[feature request](.github/ISSUE_TEMPLATE/feature_request.yml), and
[pull request template](.github/pull_request_template.md) describe the evidence
and checks to include. These are templates only; preparing one does not publish
an issue or authorize deployment. Keep public reports free of secrets and personal data.
