#!/bin/bash
set -e

# Upgrade soroban-user-profile contract in place (preserves address and data)

# Configuration
NETWORK="local"
DEPLOYER="local-deployer"
ENV_FILE=".deployed-contract.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}Error: $ENV_FILE not found${NC}"
    echo "Run ./deploy-local.sh first to deploy the contract"
    exit 1
fi

# Load existing contract address
source "$ENV_FILE"

echo -e "${GREEN}=== Soroban User Profile Contract Upgrade ===${NC}"
echo -e "Network: ${YELLOW}$NETWORK${NC}"
echo -e "Profile ID: ${YELLOW}$PROFILE_ID${NC}"
echo ""

# Build contract using stellar contract build (ensures correct WASM target)
echo -e "${YELLOW}Building contract...${NC}"
stellar contract build
echo -e "${GREEN}Build complete${NC}"
echo ""

# Install WASM and get hash
WASM_FILE="target/wasm32v1-none/release/soroban_user_profile.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo -e "${RED}Error: $WASM_FILE not found${NC}"
    exit 1
fi

echo -e "${BLUE}Installing new WASM...${NC}"
OUTPUT=$(stellar contract install \
    --wasm "$WASM_FILE" \
    --source $DEPLOYER \
    --network $NETWORK 2>&1)

# Extract the WASM hash (64 hex chars)
WASM_HASH=$(echo "$OUTPUT" | grep -E '^[a-f0-9]{64}$' | tail -1)

if [ -z "$WASM_HASH" ]; then
    echo -e "${RED}Error: Failed to install WASM${NC}"
    echo "$OUTPUT"
    exit 1
fi

echo -e "WASM hash: ${BLUE}$WASM_HASH${NC}"

# Upgrade the contract
echo ""
echo -e "${YELLOW}Upgrading contract...${NC}"
stellar contract invoke \
    --id "$PROFILE_ID" \
    --source $DEPLOYER \
    --network $NETWORK \
    -- upgrade \
    --new_wasm_hash "$WASM_HASH" 2>&1 | grep -v "^ℹ" || true

echo -e "${GREEN}Contract upgraded successfully${NC}"

echo ""
echo -e "${GREEN}=== Upgrade Complete! ===${NC}"
echo ""
echo "Contract address: $PROFILE_ID (unchanged)"
echo "All existing profiles and data preserved."
echo ""
