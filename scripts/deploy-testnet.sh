#!/usr/bin/env bash
#
# deploy-testnet.sh — deploy the eventbadges contract to Stellar testnet.
#
# WRITTEN FOR THE HUMAN TO RUN. NEVER RUN BY AN AGENT.
# The agent that works in this repository must not execute this script.
#
# Three gates apply, the same ones the app repo's deploy script enforces:
#   1. `PILOT_CONFIRMED=yes` — only set once a real organizer has agreed.
#   2. `STELLAR_ACCOUNT` — the signing identity, from `stellar keys`.
#   3. A human terminal. This script is never run by an agent.
#
# Reads the deployer identity from the environment; no secret is ever read,
# printed or stored by this script beyond what `stellar` itself does in the
# user's own `stellar keys` store.
#
# Usage:
#   STELLAR_ACCOUNT=your-identity-name bash scripts/deploy-testnet.sh
#
set -euo pipefail

if [ "${1:-}" = "--help" ]; then
  echo "Usage: PILOT_CONFIRMED=yes STELLAR_ACCOUNT=your-identity-name bash scripts/deploy-testnet.sh"
  echo
  echo "The identity must already exist in \`stellar keys\`. The script builds"
  echo "the wasm, deploys it to testnet, and prints the contract id and the"
  echo "commands to record it in the app's .env (which you fill in yourself)."
  exit 0
fi

if [ "${PILOT_CONFIRMED:-no}" != "yes" ]; then
  cat >&2 <<'EOF'
refusing to deploy: the pilot gate is not cleared.

No eventbadges contract is deployed until a real organizer or community has
agreed to try the flow. When that has actually happened, re-run with:

  PILOT_CONFIRMED=yes STELLAR_ACCOUNT=<identity> bash scripts/deploy-testnet.sh
EOF
  exit 1
fi

if [ -z "${STELLAR_ACCOUNT:-}" ]; then
  echo "set STELLAR_ACCOUNT to a stellar keys identity name, for example:" >&2
  echo "  PILOT_CONFIRMED=yes STELLAR_ACCOUNT=dev bash scripts/deploy-testnet.sh" >&2
  exit 1
fi

set -x
stellar contract build
stellar contract deploy \
  --wasm target/wasm32v1-none/release/eventbadges.wasm \
  --network testnet \
  --source-account "$STELLAR_ACCOUNT"
set +x

echo
echo "Deployed. Record the printed contract id in eventbadges-app/.env as the"
echo "contract address value the app reads. Never commit that .env."
