#!/usr/bin/env bash
#
# deploy-testnet.sh — deploy the eventbadges contract to Stellar testnet.
#
# WRITTEN FOR THE HUMAN TO RUN. NEVER RUN BY AN AGENT.
# The agent that works in this repository must not execute this script.
#
# Reads the deployer identity from the environment; no secret is ever read,
# printed or stored by this script beyond what `stellar` itself does in the
# user's own `stellar keys` store.
#
# Usage:
#   STELLAR_ACCOUNT=your-identity-name bash scripts/deploy-testnet.sh
#
set -euo pipefail

if [ "${1:-}" = "--help" ] || [ -z "${STELLAR_ACCOUNT:-}" ]; then
  echo "Usage: STELLAR_ACCOUNT=your-identity-name bash scripts/deploy-testnet.sh"
  echo
  echo "The identity must already exist in \`stellar keys\`. The script builds"
  echo "the wasm, deploys it to testnet, and prints the contract id and the"
  echo "commands to record it in the app's .env (which you fill in yourself)."
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
