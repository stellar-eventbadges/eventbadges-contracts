# eventbadges synthetic testnet demonstration

Deployed October 8, 2026. Reverified `2026-10-08T18:52:08.061Z`.
The maintainer explicitly authorized this demonstration using synthetic data.
No real pilot, partner agreement, security audit or production readiness is claimed.

- Network: Stellar **testnet**, protocol 29 at verification.
- Contract ID: `CAHAB7ABUENVP7SZW37V3VRSAAASB6SV7FOBNWS2JYGUU7W6GHYY6JWR`.
- [Stellar Expert contract](https://stellar.expert/explorer/testnet/contract/CAHAB7ABUENVP7SZW37V3VRSAAASB6SV7FOBNWS2JYGUU7W6GHYY6JWR).
- [Stellar Lab contract explorer](https://lab.stellar.org/r/testnet/contract/CAHAB7ABUENVP7SZW37V3VRSAAASB6SV7FOBNWS2JYGUU7W6GHYY6JWR).
- Source commit: `cae1bd867433ed3dca1463a961ae306f848a1aac`; runtime source matched the rebuilt artifact. The worktree also contains contributor documentation changes.
- Wasm SHA-256: `671a29a1cc65df5b7de8952b276b765db3b5a2e048cfacd7d8ae707f4bfb62bf`. Local and deployed hashes matched.
- Public deployer account: `GBHVPV4S3JRAOPQYODNULNPBW57REIJ2SDNAHYHN6SXGOKJCS3XLPVZD`. Signing material stays outside Git.

| Transaction | Hash / explorer | RPC verification |
|---|---|---|
| creation | [9c769b49e9d26e4a3895dac51402f546b46f6ae86529e512ff3e831f7f67deb5](https://stellar.expert/explorer/testnet/tx/9c769b49e9d26e4a3895dac51402f546b46f6ae86529e512ff3e831f7f67deb5) | SUCCESS; ledger 5078364 |
| upload | [6f24bd81fd49b3e655a1cde8565dfc436978667c08b95a026de3a700de18fe49](https://stellar.expert/explorer/testnet/tx/6f24bd81fd49b3e655a1cde8565dfc436978667c08b95a026de3a700de18fe49) | SUCCESS; ledger 5078362 |

## Observed checks

Before deployment, formatting, locked clippy with warnings denied, locked Rust tests,
Node checker tests, error/documentation checks and the Stellar CLI 28.1.0 Wasm build passed.
35 Rust tests and 23 Node tests; event synchronization also passed.
During verification, all listed transactions were `SUCCESS` through the testnet RPC
and the deployed Wasm hash matched the tested local artifact.

This contract has no initializer.

## Remaining boundaries

[Live synthetic CLI/RPC smoke tests](LIVE_TESTNET_SMOKE.md) have now passed.
Browser-wallet integration has not been tested.
Local tests do not establish real-user outcomes. No mainnet deployment is authorized.
Pilot deployment still requires the documented real-user agreement. Testnet resets
and storage expiry can make this address or its state unavailable later.
