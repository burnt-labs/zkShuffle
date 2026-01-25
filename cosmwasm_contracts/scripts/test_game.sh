#!/bin/bash

# zkShuffle Contract Test Script
# Tests: CreateGame, Register, PlayerRegister, Shuffle, PlayerShuffle, PlayerDealCards

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

# Validate required environment variables
required_vars=("CONTRACT_ADDRESS" "RPC_URL" "CHAIN_ID" "FROM_ACCOUNT")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "Error: Required environment variable $var is not set"
        exit 1
    fi
done

echo "=== zkShuffle Contract Test Script ==="
echo "Contract: $CONTRACT_ADDRESS"
echo "RPC: $RPC_URL"
echo "Chain ID: $CHAIN_ID"
echo "From: $FROM_ACCOUNT"
echo ""

# Game configuration
GAME_ID=${GAME_ID:-1}
NUM_PLAYERS=${NUM_PLAYERS:-2}

# Function to execute transaction
execute_tx() {
    local msg="$1"
    echo "Executing: $msg"
    xiond tx wasm execute "$CONTRACT_ADDRESS" "$msg" \
        --from "$FROM_ACCOUNT" \
        --gas-prices 0.025uxion \
        --gas auto \
        --gas-adjustment 1.3 \
        -y \
        --node "$RPC_URL" \
        --chain-id "$CHAIN_ID"
    echo ""
}

# Function to query contract state
query_contract() {
    local msg="$1"
    echo "Querying: $msg"
    xiond query wasm contract-state smart "$CONTRACT_ADDRESS" "$msg" \
        --node "$RPC_URL" \
        --output json
    echo ""
}

# ============================================================================
# Test 1: CreateGame
# ============================================================================
echo "=== Test 1: CreateGame ==="
CREATE_MSG='{"create_game": {"num_players": '"$NUM_PLAYERS"'}}'
execute_tx "$CREATE_MSG"

# Query game info to verify
# echo "Verifying game creation..."
# GAME_INFO_MSG='{"game_info": {"game_id": '"$GAME_ID"'}}'
# query_contract "$GAME_INFO_MSG"

# Wait for transaction to be processed
sleep 2

# ============================================================================
# Test 2: Register (First Player)
# ============================================================================
# echo "=== Test 2: Register (Player 1) ==="
# REGISTER_MSG='{"register": {"game_id": '"$GAME_ID"'}}'
# execute_tx "$REGISTER_MSG"

# sleep 10

# ============================================================================
# Test 3: Register (Second Player - using different account if available)
# ============================================================================
# echo "=== Test 3: Register (Player 2) ==="
# if [ -n "$SATYAM3" ]; then
#     xiond tx wasm execute "$CONTRACT_ADDRESS" "$REGISTER_MSG" \
#         --from "$SATYAM3" \
#         --gas-prices 0.025uxion \
#         --gas auto \
#         --gas-adjustment 1.3 \
#         -y \
#         --node "$RPC_URL" \
#         --chain-id "$CHAIN_ID"
# else
#     echo "Warning: SATYAM3 not set, skipping second player registration"
#     echo "You'll need another account to complete the game"
# fi
# echo ""

# sleep 10

# # Query game state to check current state
# echo "Checking game state..."
# GAME_STATE_MSG='{"game_state": {"game_id": '"$GAME_ID"'}}'
# query_contract "$GAME_STATE_MSG"

# ============================================================================
# Test 4: PlayerRegister (with public key)
# ============================================================================
echo "=== Test 4: PlayerRegister ==="
echo "Note: This requires the actual public key coordinates from your key generation"

# Example public key (replace with actual values)
# These are example BLS12-381 curve points
PK_X=10031262171927540148667355526369034398030886437092045105752248699557385197826
PK_Y=633281375905621697187330766174974863687049529291089048651929454608812697683

PK1_X=2763488322167937039616325905516046217694264098671987087929565332380420898366
PK1_Y=15305195750036305661220525648961313310481046260814497672243197092298550508693


PLAYER_REGISTER_MSG='{"player_register": {
    "game_id": '"$GAME_ID"',
    "signing_addr": "'"$FROM_ACCOUNT"'",
    "pk_x": "'"$PK_X"'",
    "pk_y": "'"$PK_Y"'"
}}'
execute_tx "$PLAYER_REGISTER_MSG"

sleep 10

# ============================================================================
# Test 5: Shuffle (initiate shuffle phase)
# ============================================================================
echo "=== Test 5: Shuffle ==="
SHUFFLE_MSG='{"shuffle": {"game_id": '"$GAME_ID"'}}'
execute_tx "$SHUFFLE_MSG"

sleep 10

# Query current player index
echo "Checking current player index..."
CUR_PLAYER_MSG='{"cur_player_index": {"game_id": '"$GAME_ID"'}}'
query_contract "$CUR_PLAYER_MSG"

# ============================================================================
# Test 6: PlayerShuffle (with proof and deck)
# ============================================================================
echo "=== Test 6: PlayerShuffle ==="
echo "Note: This requires actual proof and shuffled deck data"
echo "Loading proof data from scripts/data/shuffle_encrypt.json..."

if [ -f "$SCRIPT_DIR/data/shuffle_encrypt.json" ]; then
    # Extract proof data from the file
    PROOF_DATA=$(cat "$SCRIPT_DIR/data/shuffle_encrypt.json")

    # Parse pi_a (convert to Uint256 array format)
    PI_A_0=$(echo "$PROOF_DATA" | jq -r '.proof.pi_a[0]')
    PI_A_1=$(echo "$PROOF_DATA" | jq -r '.proof.pi_a[1]')

    # Parse pi_b (2x2 array)
    PI_B_0_0=$(echo "$PROOF_DATA" | jq -r '.proof.pi_b[0][0]')
    PI_B_0_1=$(echo "$PROOF_DATA" | jq -r '.proof.pi_b[0][1]')
    PI_B_1_0=$(echo "$PROOF_DATA" | jq -r '.proof.pi_b[1][0]')
    PI_B_1_1=$(echo "$PROOF_DATA" | jq -r '.proof.pi_b[1][1]')

    # Parse pi_c
    PI_C_0=$(echo "$PROOF_DATA" | jq -r '.proof.pi_c[0]')
    PI_C_1=$(echo "$PROOF_DATA" | jq -r '.proof.pi_c[1]')

    echo "Building PlayerShuffle message..."
    echo "Note: You need to provide actual deck data (x0, x1 arrays, selector0, selector1)"
    echo "This is a template - replace the deck values with your actual shuffled deck"

    # Example deck structure (52 cards for deck52)
    # You'll need to generate actual deck data from your circuit
    cat > /tmp/player_shuffle_template.json <<EOF
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
      "config": "deck52_card",
      "x0": [
        "x0_0_value", "x0_1_value", ...
      ],
      "x1": [
        "x1_0_value", "x1_1_value", ...
      ],
      "selector0": "bitmap_value_0",
      "selector1": "bitmap_value_1"
    }
  }
}
EOF
    echo "Template saved to /tmp/player_shuffle_template.json"
    echo ""
    echo "To execute PlayerShuffle, you need to:"
    echo "1. Generate actual shuffled deck data (52 x0 values, 52 x1 values)"
    echo "2. Generate selector bitmaps (BitMap256 format)"
    echo "3. Update the template and execute manually"
else
    echo "Warning: shuffle_encrypt.json not found"
fi

sleep 2

# ============================================================================
# Test 7: DealCardsTo (initiate deal phase)
# ============================================================================
echo "=== Test 7: DealCardsTo ==="
echo "Dealing cards to player 0..."

# Example: Deal 5 cards (bitmap format)
# BitMap256 representing cards 0,1,2,3,4
CARDS_BITMAP="31"  # Binary: 11111 = first 5 cards

DEAL_MSG='{"deal_cards_to": {
    "game_id": '"$GAME_ID"',
    "cards": "'"$CARDS_BITMAP"'",
    "player_id": 0
}}'
execute_tx "$DEAL_MSG"

sleep 2

# ============================================================================
# Test 8: PlayerDealCards (with decryption proofs)
# ============================================================================
echo "=== Test 8: PlayerDealCards ==="
echo "Loading decryption proof from scripts/data/decrypt.json..."

if [ -f "$SCRIPT_DIR/data/decrypt.json" ]; then
    DECRYPT_DATA=$(cat "$SCRIPT_DIR/data/decrypt.json")

    # Extract proof
    DECRYPT_PI_A_0=$(echo "$DECRYPT_DATA" | jq -r '.proof.pi_a[0]')
    DECRYPT_PI_A_1=$(echo "$DECRYPT_DATA" | jq -r '.proof.pi_a[1]')

    DECRYPT_PI_B_0_0=$(echo "$DECRYPT_DATA" | jq -r '.proof.pi_b[0][0]')
    DECRYPT_PI_B_0_1=$(echo "$DECRYPT_DATA" | jq -r '.proof.pi_b[0][1]')
    DECRYPT_PI_B_1_0=$(echo "$DECRYPT_DATA" | jq -r '.proof.pi_b[1][0]')
    DECRYPT_PI_B_1_1=$(echo "$DECRYPT_DATA" | jq -r '.proof.pi_b[1][1]')

    DECRYPT_PI_C_0=$(echo "$DECRYPT_DATA" | jq -r '.proof.pi_c[0]')
    DECRYPT_PI_C_1=$(echo "$DECRYPT_DATA" | jq -r '.proof.pi_c[1]')

    echo "Building PlayerDealCards message..."
    echo "Note: You need actual decrypted card values and init_deltas"

    cat > /tmp/player_deal_template.json <<EOF
{
  "player_deal_cards": {
    "game_id": $GAME_ID,
    "proofs": [{
      "a": ["$DECRYPT_PI_A_0", "$DECRYPT_PI_A_1"],
      "b": [
        ["$DECRYPT_PI_B_0_0", "$DECRYPT_PI_B_0_1"],
        ["$DECRYPT_PI_B_1_0", "$DECRYPT_PI_B_1_1"]
      ],
      "c": ["$DECRYPT_PI_C_0", "$DECRYPT_PI_C_1"]
    }],
    "decrypted_cards": [
      {"x": "card_x_value", "y": "card_y_value"}
    ],
    "init_deltas": [
      {"delta0": "delta0_value", "delta1": "delta1_value"}
    ]
  }
}
EOF
    echo "Template saved to /tmp/player_deal_template.json"
    echo ""
    echo "To execute PlayerDealCards, you need to:"
    echo "1. Generate decryption proofs for each card being dealt"
    echo "2. Provide decrypted card values (x, y coordinates)"
    echo "3. Provide init_deltas for each card"
else
    echo "Warning: decrypt.json not found"
fi

# ============================================================================
# Summary and Next Steps
# ============================================================================
echo ""
echo "=== Test Summary ==="
echo "Basic transaction tests completed!"
echo ""
echo "Next Steps:"
echo "1. Generate actual ZK proofs using your circuit"
echo "2. Create proper deck data with shuffled card positions"
echo "3. Fill in the templates in /tmp/ with real values"
echo "4. Execute PlayerShuffle and PlayerDealCards with complete data"
echo ""
echo "Query commands for debugging:"
echo "  Game State: xiond query wasm contract-state smart $CONTRACT_ADDRESS '{\"game_state\": {\"game_id\": $GAME_ID}}' --node $RPC_URL"
echo "  Deck: xiond query wasm contract-state smart $CONTRACT_ADDRESS '{\"deck\": {\"game_id\": $GAME_ID}}' --node $RPC_URL"
echo "  Aggregated PK: xiond query wasm contract-state smart $CONTRACT_ADDRESS '{\"aggregated_pk\": {\"game_id\": $GAME_ID}}' --node $RPC_URL"
