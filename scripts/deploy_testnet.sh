#!/usr/bin/env bash
set -euo pipefail

# AjoPay testnet deploy script.
# Requires: stellar-cli configured with a funded testnet identity named "ajopay-admin".
# https://developers.stellar.org/docs/tools/stellar-cli for setup instructions.

NETWORK="testnet"
SOURCE="ajopay-admin"

echo "Building contracts..."
cargo build --target wasm32-unknown-unknown --release

SETTLEMENT_WASM="target/wasm32-unknown-unknown/release/ajopay_settlement.wasm"
GOVERNANCE_WASM="target/wasm32-unknown-unknown/release/ajopay_governance.wasm"

echo "Deploying governance contract..."
GOVERNANCE_ID=$(stellar contract deploy \
  --wasm "$GOVERNANCE_WASM" \
  --source "$SOURCE" \
  --network "$NETWORK")
echo "Governance contract: $GOVERNANCE_ID"

echo "Deploying settlement contract..."
SETTLEMENT_ID=$(stellar contract deploy \
  --wasm "$SETTLEMENT_WASM" \
  --source "$SOURCE" \
  --network "$NETWORK")
echo "Settlement contract: $SETTLEMENT_ID"

ADMIN_ADDRESS=$(stellar keys address "$SOURCE")

echo "Initializing governance..."
stellar contract invoke \
  --id "$GOVERNANCE_ID" \
  --source "$SOURCE" \
  --network "$NETWORK" \
  -- initialize --admin "$ADMIN_ADDRESS" --initial_fee_bps 150

echo "Initializing settlement..."
stellar contract invoke \
  --id "$SETTLEMENT_ID" \
  --source "$SOURCE" \
  --network "$NETWORK" \
  -- initialize --admin "$ADMIN_ADDRESS" --governance_addr "$GOVERNANCE_ID"

echo ""
echo "Done. Save these addresses to your README:"
echo "  Settlement: $SETTLEMENT_ID"
echo "  Governance: $GOVERNANCE_ID"
echo "  Admin:      $ADMIN_ADDRESS"
