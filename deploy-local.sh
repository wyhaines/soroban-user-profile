#!/bin/bash
set -e

# Deploy soroban-user-profile to local Stellar network
# Optionally registers with soroban-boards registry if available

# Configuration
NETWORK="local"
RPC_URL="http://localhost:8000/soroban/rpc"
DEPLOYER="local-deployer"
BOARDS_ENV="../soroban-boards/.deployed-contracts.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Soroban User Profile Local Deployment ===${NC}"

# Check if friendbot is available for funding
fund_account() {
    local addr=$1
    echo -e "${YELLOW}Funding account $addr...${NC}"
    curl -s "http://localhost:8000/friendbot?addr=$addr" > /dev/null 2>&1 || true
}

# Get deployer address
ADMIN=$(stellar keys address $DEPLOYER 2>/dev/null || true)
if [ -z "$ADMIN" ]; then
    echo -e "${YELLOW}Creating deployer identity...${NC}"
    stellar keys generate $DEPLOYER --network $NETWORK 2>/dev/null || true
    ADMIN=$(stellar keys address $DEPLOYER)
fi
echo -e "Admin: ${YELLOW}$ADMIN${NC}"

# Fund the deployer account
fund_account $ADMIN

# Build contract using stellar contract build (ensures correct WASM target)
echo ""
echo -e "${YELLOW}Building contract...${NC}"
stellar contract build
echo -e "${GREEN}Build complete${NC}"

# Deploy contract (stellar contract build outputs to wasm32v1-none)
WASM_FILE="target/wasm32v1-none/release/soroban_user_profile.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo -e "${RED}Error: $WASM_FILE not found${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}=== Deploying User Profile Contract ===${NC}"
OUTPUT=$(stellar contract deploy \
    --wasm "$WASM_FILE" \
    --source $DEPLOYER \
    --network $NETWORK 2>&1)

# Extract contract ID
PROFILE_ID=$(echo "$OUTPUT" | grep -E '^C[A-Z0-9]{55}$' | tail -1)

if [ -z "$PROFILE_ID" ]; then
    echo -e "${RED}Error: Failed to deploy contract${NC}"
    echo "$OUTPUT"
    exit 1
fi

echo -e "${GREEN}Profile contract deployed: ${YELLOW}$PROFILE_ID${NC}"

# Save contract ID
echo ""
echo -e "${GREEN}=== Saving Contract ID ===${NC}"
cat > .deployed-contract.env << EOF
# Soroban User Profile - Local Deployment
# Generated: $(date)

NETWORK=$NETWORK
RPC_URL=$RPC_URL
ADMIN=$ADMIN

PROFILE_ID=$PROFILE_ID
EOF

echo -e "${GREEN}Contract ID saved to .deployed-contract.env${NC}"

# Initialize contract
echo ""
echo -e "${GREEN}=== Initializing Profile Contract ===${NC}"
stellar contract invoke \
    --id $PROFILE_ID \
    --source $DEPLOYER \
    --network $NETWORK \
    -- init \
    --admin $ADMIN

echo -e "${GREEN}Profile contract initialized${NC}"

# Check if soroban-boards is deployed and register with it
if [ -f "$BOARDS_ENV" ]; then
    echo ""
    echo -e "${GREEN}=== Registering with Soroban Boards ===${NC}"
    source "$BOARDS_ENV"

    if [ -n "$REGISTRY_ID" ]; then
        echo -e "Found boards registry: ${YELLOW}$REGISTRY_ID${NC}"

        stellar contract invoke \
            --id "$REGISTRY_ID" \
            --source $DEPLOYER \
            --network $NETWORK \
            -- set_contract \
            --alias profile \
            --address $PROFILE_ID 2>&1 | grep -v "^ℹ" || true

        echo -e "${GREEN}Profile contract registered as 'profile' alias${NC}"

        # Append to boards env file
        echo "" >> "$BOARDS_ENV"
        echo "# User Profile Contract (registered via set_contract)" >> "$BOARDS_ENV"
        echo "PROFILE_ID=$PROFILE_ID" >> "$BOARDS_ENV"
    else
        echo -e "${YELLOW}Registry ID not found in $BOARDS_ENV${NC}"
    fi
else
    echo ""
    echo -e "${YELLOW}Note: soroban-boards not found at $BOARDS_ENV${NC}"
    echo -e "${YELLOW}To register with boards, run:${NC}"
    echo -e "  stellar contract invoke --id \$REGISTRY_ID --source $DEPLOYER --network $NETWORK -- set_contract --alias profile --address $PROFILE_ID"
fi

# Create a sample profile
echo ""
echo -e "${GREEN}=== Creating Sample Profile ===${NC}"

# Register a sample user (hex for "alice001")
# alice001 = 616c696365303031
stellar contract invoke \
    --id $PROFILE_ID \
    --source $DEPLOYER \
    --network $NETWORK \
    -- register \
    --username 616c696365303031 \
    --display_name "Alice" \
    --caller $ADMIN

echo -e "${GREEN}Sample profile 'alice001' created${NC}"

# Set a bio field
stellar contract invoke \
    --id $PROFILE_ID \
    --source $DEPLOYER \
    --network $NETWORK \
    -- set_string_field \
    --field bio \
    --value "Hello from Soroban!" \
    --caller $ADMIN

echo -e "${GREEN}Bio field set${NC}"

echo ""
echo -e "${GREEN}=== Deployment Complete! ===${NC}"
echo ""
echo "Contract ID: $PROFILE_ID"
echo ""
echo "To interact with the contract:"
echo "  source .deployed-contract.env"
echo ""
echo "Example commands:"
echo ""
echo "  # Get profile by username (alice001 = 616c696365303031)"
echo "  stellar contract invoke --id \$PROFILE_ID --source $DEPLOYER --network local -- get_by_username --username 616c696365303031"
echo ""
echo "  # Get profile by address"
echo "  stellar contract invoke --id \$PROFILE_ID --source $DEPLOYER --network local -- get_by_address --address $ADMIN"
echo ""
echo "  # Render home page"
echo "  ./render.sh /"
echo ""
echo "  # Render profile"
echo "  ./render.sh /u/alice001"
echo ""
