# EventBadges live synthetic testnet smoke

Completed 2026-10-08T23:18:47.179Z. **Synthetic CLI demonstration only: no browser wallet, real user, event partner, pilot, mainnet or audit is claimed.**

Contract: [CAHAB7ABUENVP7SZW37V3VRSAAASB6SV7FOBNWS2JYGUU7W6GHYY6JWR](https://stellar.expert/explorer/testnet/contract/CAHAB7ABUENVP7SZW37V3VRSAAASB6SV7FOBNWS2JYGUU7W6GHYY6JWR). Event id: **1**. Public organizer/attendee: `GDNQHPOYWNJ2ORLXWK2JR67JIAM3TKMSKUSKJMIHBOMAUR5BK4C22QRM`. The existing dedicated testnet identity was reused; no signing material is published. CLI 28.1.0 used the official Stellar testnet RPC through a temporary local relay that preserved upstream HTTPS/TLS verification.

| Action | RPC result | Ledger | Transaction |
|---|---|---:|---|
| create | SUCCESS | 5094449 | [fd738d5a2c8929c76cfb6b30b67c2267b5686f1d467252fa7d2be084c58257f7](https://stellar.expert/explorer/testnet/tx/fd738d5a2c8929c76cfb6b30b67c2267b5686f1d467252fa7d2be084c58257f7) |
| claim | SUCCESS | 5094494 | [3725c03da357f6db50ae378f4e0db2be28eb1b5c4d3eb92f2609fbfb27b2a368](https://stellar.expert/explorer/testnet/tx/3725c03da357f6db50ae378f4e0db2be28eb1b5c4d3eb92f2609fbfb27b2a368) |
| revoke | SUCCESS | 5094503 | [8607a58491de5a994d018243b0e10aa0674e4c7aa742e47cebc2ba6cb3ce0d06](https://stellar.expert/explorer/testnet/tx/8607a58491de5a994d018243b0e10aa0674e4c7aa742e47cebc2ba6cb3ce0d06) |
| award | SUCCESS | 5094522 | [81d8caf1f2da8c3e92757afed5f813789d71f77b8cf5ea14d7f4cf7ccf172312](https://stellar.expert/explorer/testnet/tx/81d8caf1f2da8c3e92757afed5f813789d71f77b8cf5ea14d7f4cf7ccf172312) |

All 7 actual-state assertions passed. After claim, has_badge was true, badges_of contained exactly the synthetic attendee and claim_count was 1. Revoke removed the badge and returned count to 0. Organizer award restored the badge and count to 1. The stored organizer, hashes, cap and deadline matched creation input. Each write was independently checked with getTransaction; writes were never automatically retried.

Invalid proof returned contract error **14** and duplicate claim returned **13**, both with `--send=no`: no failing transaction was broadcast. The tree intentionally had one leaf and an empty proof; this does not demonstrate invitation confidentiality. Both roles used one synthetic account, so this is not a multi-user authorization exercise.

To inspect evidence on Stellar Expert:

1. Open the contract link and confirm **Testnet** and the full C… contract id.
2. Open each transaction above; confirm SUCCESS, the ledger, and contract method (create_event, claim, revoke, award).
3. Inspect public invocation arguments and events. Match event id 1, public organizer/attendee, and the hashes in [the JSON evidence](eventbadges-live-smoke.json).
4. Remember that later revoke/award operations changed current state; the transaction history explains the earlier claimed badge. Explorer evidence proves these synthetic invocations, not adoption, a pilot or safety.
