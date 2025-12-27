#!/bin/bash

# Render output from the User Profile contract
# Usage: ./render.sh <path> [viewer_address]
#
# Examples:
#   ./render.sh /                    # Home page
#   ./render.sh /u/alice001          # Profile by username
#   ./render.sh /a/GABC...           # Profile by address
#   ./render.sh /register            # Registration form
#   ./render.sh /edit $ADMIN         # Edit form (requires viewer)

# Configuration
NETWORK="local"
DEPLOYER="local-deployer"
ENV_FILE=".deployed-contract.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}Error: $ENV_FILE not found${NC}" >&2
    echo "Run ./deploy-local.sh first to deploy the contract" >&2
    exit 1
fi

# Load contract address
source "$ENV_FILE"

# Get path from arguments
PATH_ARG="${1:-/}"
VIEWER_ARG="$2"

# Build the invoke command
CMD="stellar contract invoke --id $PROFILE_ID --source $DEPLOYER --network $NETWORK -- render"

# Add path if not root
if [ "$PATH_ARG" != "/" ]; then
    CMD="$CMD --path \"$PATH_ARG\""
fi

# Add viewer if provided
if [ -n "$VIEWER_ARG" ]; then
    CMD="$CMD --viewer $VIEWER_ARG"
fi

# Execute and decode output
OUTPUT=$(eval "$CMD" 2>&1)

# Check for errors
if echo "$OUTPUT" | grep -q "error"; then
    echo -e "${RED}Error:${NC}" >&2
    echo "$OUTPUT" >&2
    exit 1
fi

# Extract hex string (remove quotes and any prefix)
HEX=$(echo "$OUTPUT" | tr -d '"' | grep -oE '[0-9a-fA-F]+' | tail -1)

if [ -z "$HEX" ]; then
    echo -e "${YELLOW}No output or empty response${NC}" >&2
    echo "$OUTPUT"
    exit 0
fi

# Decode hex to text
echo "$HEX" | xxd -r -p

echo ""
