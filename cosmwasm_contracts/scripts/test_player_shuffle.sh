#!/bin/bash

# Test script for PlayerShuffle method
# This script parses the shuffle_encrypt.json file and executes PlayerShuffle

set -e

# Source environment variables
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

if [ -f "$PROJECT_ROOT/.env.local" ]; then
    source "$PROJECT_ROOT/.env.local"
else
    echo "Error: .env.local file not found at $PROJECT_ROOT/.env.local"
    exit 1
fi

# Configuration
CONTRACT_ADDRESS="${CONTRACT_ADDRESS:-$NEXT_PUBLIC_CONTRACT_ADDRESS}"
RPC_URL="${RPC_URL:-$NEXT_PUBLIC_RPC_URL}"
CHAIN_ID="${CHAIN_ID:-xion-testnet-2}"
FROM_ACCOUNT="${SATYAM2}"
GAME_ID=${GAME_ID:-1}

# Validate required environment variables
required_vars=("CONTRACT_ADDRESS" "RPC_URL" "CHAIN_ID" "FROM_ACCOUNT")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "Error: Required environment variable $var is not set"
        exit 1
    fi
done

echo "=== Testing PlayerShuffle ==="
echo "Contract: $CONTRACT_ADDRESS"
echo "Game ID: $GAME_ID"
echo "From: $FROM_ACCOUNT"
echo ""

# Check if shuffle_encrypt.json exists
SHUFFLE_DATA_FILE="$SCRIPT_DIR/data/shuffle_encrypt.json"
if [ ! -f "$SHUFFLE_DATA_FILE" ]; then
    echo "Error: $SHUFFLE_DATA_FILE not found"
    exit 1
fi

echo "Step 1: Parsing shuffle_encrypt.json..."

# Extract proof data
PI_A_0=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.proof.pi_a[0]')
PI_A_1=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.proof.pi_a[1]')

PI_B_0_0=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.proof.pi_b[0][0]')
PI_B_0_1=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.proof.pi_b[0][1]')
PI_B_1_0=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.proof.pi_b[1][0]')
PI_B_1_1=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.proof.pi_b[1][1]')

PI_C_0=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.proof.pi_c[0]')
PI_C_1=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.proof.pi_c[1]')

echo "  ✓ Proof data extracted"
echo "    pi_a[0]: ${PI_A_0:0:50}..."
echo ""

# Parse the compressedDeck (it's a stringified JSON)
echo "Step 2: Parsing compressedDeck..."

# Extract the compressedDeck string and parse it
COMPRESSED_DECK_STR=$(cat "$SHUFFLE_DATA_FILE" | jq -r '.compressedDeck')

# Create a temporary file for the parsed compressed deck
echo "$COMPRESSED_DECK_STR" > /tmp/compressed_deck_parsed.json

# Now extract the arrays from the parsed compressed deck
# X0 and X1 are the card coordinates
X0_ARRAY=$(cat /tmp/compressed_deck_parsed.json | jq -c '.X0')
X1_ARRAY=$(cat /tmp/compressed_deck_parsed.json | jq -c '.X1')

# selector has 2 elements: selector[0] and selector[1] correspond to selector0 and selector1
SELECTOR_0=$(cat /tmp/compressed_deck_parsed.json | jq -r '.selector[0]')
SELECTOR_1=$(cat /tmp/compressed_deck_parsed.json | jq -r '.selector[1]')

# Count the number of cards to determine deck config
NUM_CARDS=$(echo "$COMPRESSED_DECK_STR" | jq '.X0 | length')
echo "  ✓ Compressed deck parsed"
echo "    Number of cards: $NUM_CARDS"

# Determine deck config based on number of cards
if [ "$NUM_CARDS" = "5" ]; then
    DECK_CONFIG="deck5_card"
elif [ "$NUM_CARDS" = "30" ]; then
    DECK_CONFIG="deck30_card"
elif [ "$NUM_CARDS" = "52" ]; then
    DECK_CONFIG="deck52_card"
else
    echo "Error: Unknown number of cards: $NUM_CARDS"
    exit 1
fi

echo "    Deck config: $DECK_CONFIG"
echo ""

# Build the PlayerShuffle message
echo "Step 3: Building PlayerShuffle message..."

# Create the message JSON
cat > /tmp/player_shuffle_msg.json <<EOF
{
  "player_shuffle": {
    "game_id": $GAME_ID,
    "proof": {
      "a": ["$PI_A_0", "$PI_A_1"],
      "b": [
        ["$PI_B_0_0", "$PI_B_0_1"],
        ["$PI_B_1_0", "$PI_B_1_1"]
      ],
      "c": ["$PI_C_0", "$PI_C_1"]
    },
    "deck": {
      "config": "$DECK_CONFIG",
      "x0": $X0_ARRAY,
      "x1": $X1_ARRAY,
      "selector0": "$SELECTOR_0",
      "selector1": "$SELECTOR_1"
    }
  }
}
EOF

echo "  ✓ Message built"
echo ""

# Display the message (pretty printed)
echo "Step 4: Message preview:"
jq '.' /tmp/player_shuffle_msg.json
echo ""

# Query current game state before executing
echo "Step 5: Current game state:"
CUR_PLAYER_MSG="{\"cur_player_index\": {\"game_id\": $GAME_ID}}"
xiond query wasm contract-state smart "$CONTRACT_ADDRESS" "$CUR_PLAYER_MSG" \
    --node "$RPC_URL" \
    --output json 2>/dev/null | jq '.'
echo ""

# Confirm before executing
read -p "Execute PlayerShuffle transaction? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Execute the transaction
echo "Step 6: Executing PlayerShuffle transaction..."
echo ""

# Read the message content
MSG_CONTENT=$(cat /tmp/player_shuffle_msg.json)

xiond tx wasm execute "$CONTRACT_ADDRESS" \
    "$MSG_CONTENT" \
    --from "$FROM_ACCOUNT" \
    --gas-prices 0.025uxion \
    --gas auto \
    --gas-adjustment 1.3 \
    -y \
    --node "$RPC_URL" \
    --chain-id "$CHAIN_ID"

TX_RESULT=$?

echo ""
echo "============================================================================"
if [ $TX_RESULT -eq 0 ]; then
    echo "✓ Transaction submitted successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Wait for the transaction to be included in a block"
    echo "2. Query the game state to verify: ./query_game.sh state"
    echo "3. If there are more players, they should now execute their PlayerShuffle"
else
    echo "✗ Transaction failed"
    echo "Check the error messages above"
fi
echo "============================================================================"

# Clean up temporary files
rm -f /tmp/compressed_deck_parsed.json
rm -f /tmp/player_shuffle_msg.json

exit $TX_RESULT
