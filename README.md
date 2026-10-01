# eventbadges — event badges on Stellar testnet

Status: **scaffold only.** No contract, no app, no book has been written yet.
Testnet only, no real money, **no pilot has happened**, and this project has
never run against a deployed contract. Do not use it with real funds.

eventbadges is a Stellar/Soroban project in three repositories:

| Repo | Purpose | Status |
|---|---|---|
| `eventbadges-contracts` | Soroban contract (Rust) | scaffold only |
| `eventbadges-app` | web app (Vite + React + TypeScript) | scaffold only |
| `eventbadges-docs` | mdBook documentation | scaffold only |


## What is here now

Repository governance only, adapted from the completed `schoolfees` project:
[AGENTS.md](AGENTS.md) (the rulebook for agents and humans),
[CONTRIBUTING.md](CONTRIBUTING.md), [ROADMAP.md](ROADMAP.md) (what v0 will be,
from the project's playbook section), MIT [LICENSE](LICENSE), `.gitignore`,
`.gitattributes` (LF everywhere). No code yet; the first CI workflow lands
with the first code that can pass it.

## What v0 will be

See [ROADMAP.md](ROADMAP.md). The scope is defined in the project's section of
the build playbook; it is not invented here.

## Honest limitations

- Nothing is implemented, tested, audited or deployed.
- The contract has never been compiled; the app has never run; the book has
  never been built.
- No pilot has happened and none is claimed anywhere in these repositories.
