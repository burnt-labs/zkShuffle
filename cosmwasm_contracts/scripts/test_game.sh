#!/bin/bash

# zkShuffle Contract Test Script
# Tests: CreateGame, Register, PlayerRegister, Shuffle, PlayerShuffle, DealCardsTo, PlayerDealCards

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

# Player accounts
PLAYER1="${SATYAM2}"
PLAYER2="${SATYAM3}"

# Validate required environment variables
required_vars=("CONTRACT_ADDRESS" "RPC_URL" "CHAIN_ID" "PLAYER1" "PLAYER2")
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
echo "Player 1: $PLAYER1"
echo "Player 2: $PLAYER2"
echo ""

# Game configuration
GAME_ID=${GAME_ID:-1}
NUM_PLAYERS=2

# Function to execute transaction
execute_tx() {
    local from_account="$1"
    local msg="$2"
    echo "Executing from $from_account: $msg"
    xiond tx wasm execute "$CONTRACT_ADDRESS" "$msg" \
        --from "$from_account" \
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
# Step 1: CreateGame (already done if game exists, skip if needed)
# ============================================================================
echo "=== Step 1: CreateGame ==="
echo "Creating game with $NUM_PLAYERS players..."
CREATE_MSG='{"create_game": {"num_players": '"$NUM_PLAYERS"'}}'
execute_tx "$PLAYER1" "$CREATE_MSG"

sleep 10

# Query game state to verify
echo "Verifying game creation..."
GAME_STATE_MSG='{"game_state": {"game_id": '"$GAME_ID"'}}'
query_contract "$GAME_STATE_MSG"

sleep 10

# ============================================================================
# Step 2: Register (Start registration phase)
# ============================================================================
echo "=== Step 2: Register (Start Registration Phase) ==="
REGISTER_MSG='{"register": {"game_id": '"$GAME_ID"'}}'
execute_tx "$PLAYER1" "$REGISTER_MSG"

sleep 10

# Query game state to check current state
echo "Checking game state after Register..."
query_contract "$GAME_STATE_MSG"

sleep 10

# ============================================================================
# Step 3: PlayerRegister (Player 1)
# ============================================================================
echo "=== Step 3: PlayerRegister (Player 1) ==="
echo "Player 1 registering with public key..."

# Example public key for Player 1 (replace with actual values from key generation)
PK1_X="10031262171927540148667355526369034398030886437092045105752248699557385197826"
PK1_Y="633281375905621697187330766174974863687049529291089048651929454608812697683"

PLAYER1_REGISTER_MSG='{"player_register": {
    "game_id": '"$GAME_ID"',
    "signing_addr": "'"$PLAYER1"'",
    "pk_x": "'"$PK1_X"'",
    "pk_y": "'"$PK1_Y"'"
}}'
execute_tx "$PLAYER1" "$PLAYER1_REGISTER_MSG"

sleep 10

# ============================================================================
# Step 4: PlayerRegister (Player 2)
# ============================================================================
echo "=== Step 4: PlayerRegister (Player 2) ==="
echo "Player 2 registering with public key..."

# Example public key for Player 2 (replace with actual values from key generation)
PK2_X="2763488322167937039616325905516046217694264098671987087929565332380420898366"
PK2_Y="15305195750036305661220525648961313310481046260814497672243197092298550508693"

PLAYER2_REGISTER_MSG='{"player_register": {
    "game_id": '"$GAME_ID"',
    "signing_addr": "'"$PLAYER2"'",
    "pk_x": "'"$PK2_X"'",
    "pk_y": "'"$PK2_Y"'"
}}'
execute_tx "$PLAYER2" "$PLAYER2_REGISTER_MSG"

sleep 10

# Query game state to check registration status
echo "Checking game state after all players registered..."
query_contract "$GAME_STATE_MSG"

sleep 3

# ============================================================================
# Step 5: Shuffle (Initiate shuffle phase)
# ============================================================================
echo "=== Step 5: Shuffle (Initiate Shuffle Phase) ==="
SHUFFLE_MSG='{"shuffle": {"game_id": '"$GAME_ID"'}}'
execute_tx "$PLAYER1" "$SHUFFLE_MSG"

sleep 10

# Query current player index
# echo "Checking current player index..."
# CUR_PLAYER_MSG='{"cur_player_index": {"game_id": '"$GAME_ID"'}}'
# query_contract "$CUR_PLAYER_MSG"

sleep 10

# ============================================================================
# Step 6: PlayerShuffle (Player 1)
# ============================================================================
echo "=== Step 6: PlayerShuffle (Player 1) ==="
echo "Loading proof data from $SCRIPT_DIR/data/shuffle_encrypt.json..."

if [ -f "$SCRIPT_DIR/data/shuffle_encrypt.json" ]; then
    # Extract proof data from the file
    PROOF_DATA=$(cat "$SCRIPT_DIR/data/shuffle_encrypt.json")

    # Parse pi_a
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

    echo "Building PlayerShuffle message for Player 1..."
    echo "NOTE: This requires actual deck data from your circuit output!"
    echo "The x0 and x1 arrays should contain the shuffled deck card coordinates."
    echo ""
    echo "For a complete test, you need to:"
    echo "1. Run your shuffle circuit to get the actual shuffled deck"
    echo "2. Extract the 52 card coordinates (x0 and x1 arrays)"
    echo "3. Generate selector bitmaps (BitMap256 format)"
    echo ""
    echo "Example structure (fill in actual values):"

    # Template for PlayerShuffle
    cat <<'EOF'
{
  "player_shuffle": {
    "game_id": 1,
    "proof": {
      "a": ["pi_a_0", "pi_a_1"],
      "b": [
        ["pi_b_0_0", "pi_b_0_1"],
        ["pi_b_1_0", "pi_b_1_1"]
      ],
      "c": ["pi_c_0", "pi_c_1"]
    },
    "deck": {
      "config": "deck52_card",
      "x0": [ /* 52 Uint256 values for card x0 coordinates */ ],
      "x1": [ /* 52 Uint256 values for card x1 coordinates */ ],
      "selector0": "bitmap_uint256_0",
      "selector1": "bitmap_uint256_1"
    }
  }
}
EOF
    echo ""
    echo "Skipping PlayerShuffle execution - requires real deck data."
    echo "Update this script with actual deck values from your circuit."
else
    echo "Warning: shuffle_encrypt.json not found at $SCRIPT_DIR/data/shuffle_encrypt.json"
fi

sleep 2

# ============================================================================
# Step 7: PlayerShuffle (Player 2)
# ============================================================================
echo "=== Step 7: PlayerShuffle (Player 2) ==="
echo "Player 2 would also shuffle with their own proof and deck."
echo "Skipping - requires real deck data."
echo ""

sleep 2

# ============================================================================
# Step 8: DealCardsTo (Deal cards to players)
# ============================================================================
echo "=== Step 8: DealCardsTo ==="
echo "Dealing 5 cards to Player 1..."

# BitMap256 representing cards 0,1,2,3,4 (first 5 cards)
# In hex: 0x1F = 31 decimal = binary 11111
CARDS_BITMAP="0x1f"

DEAL_MSG='{"deal_cards_to": {
    "game_id": '"$GAME_ID"',
    "cards": "'"$CARDS_BITMAP"'",
    "player_id": 0
}}'
execute_tx "$PLAYER1" "$DEAL_MSG"

sleep 3

echo "Dealing 5 cards to Player 2..."
# Deal cards 5,6,7,8,9 (next 5 cards)
# In hex: 0x3E0 = 992 decimal
CARDS_BITMAP_P2="0x3e0"

DEAL_MSG_P2='{"deal_cards_to": {
    "game_id": '"$GAME_ID"',
    "cards": "'"$CARDS_BITMAP_P2"'",
    "player_id": 1
}}'
execute_tx "$PLAYER1" "$DEAL_MSG_P2"

sleep 3

# Query game state
echo "Checking game state after dealing..."
query_contract "$GAME_STATE_MSG"

sleep 3

# ============================================================================
# Step 9: PlayerDealCards (Player 1 decrypts their cards)
# ============================================================================
echo "=== Step 9: PlayerDealCards (Player 1) ==="
echo "Loading decryption proof from $SCRIPT_DIR/data/decrypt.json..."

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

    echo "Building PlayerDealCards message for Player 1..."
    echo "NOTE: This requires actual decrypted card values and init_deltas!"
    echo ""
    echo "For a complete test, you need to:"
    echo "1. Run your decrypt circuit for each card being dealt"
    echo "2. Get the decrypted card coordinates (x, y)"
    echo "3. Get the init_deltas for each card"
    echo ""
    echo "Example structure (fill in actual values):"

    # Template for PlayerDealCards
    cat <<'EOF'
{
  "player_deal_cards": {
    "game_id": 1,
    "proofs": [
      {
        "a": ["decrypt_pi_a_0", "decrypt_pi_a_1"],
        "b": [
          ["decrypt_pi_b_0_0", "decrypt_pi_b_0_1"],
          ["decrypt_pi_b_1_0", "decrypt_pi_b_1_1"]
        ],
        "c": ["decrypt_pi_c_0", "decrypt_pi_c_1"]
      }
      /* Add more proofs if dealing multiple cards */
    ],
    "decrypted_cards": [
      {"x": "card_x_value", "y": "card_y_value"}
      /* Add more cards as needed */
    ],
    "init_deltas": [
      {"delta0": "delta0_value", "delta1": "delta1_value"}
      /* Add more deltas as needed */
    ]
  }
}
EOF
    echo ""
    echo "Skipping PlayerDealCards execution - requires real decryption data."
    echo "Update this script with actual card values from your circuit."
else
    echo "Warning: decrypt.json not found at $SCRIPT_DIR/data/decrypt.json"
fi

sleep 2

# ============================================================================
# Step 10: PlayerDealCards (Player 2 decrypts their cards)
# ============================================================================
echo "=== Step 10: PlayerDealCards (Player 2) ==="
echo "Player 2 would decrypt their cards with their own proofs."
echo "Skipping - requires real decryption data."
echo ""

# ============================================================================
# Summary and Debugging Commands
# ============================================================================
echo ""
echo "=== Test Summary ==="
echo "Game setup test completed!"
echo ""
echo "Executed steps:"
echo "  1. CreateGame - Created game with $NUM_PLAYERS players"
echo "  2. Register - Started registration phase"
echo "  3. PlayerRegister (Player 1) - Registered Player 1"
echo "  4. PlayerRegister (Player 2) - Registered Player 2"
echo "  5. Shuffle - Started shuffle phase"
echo "  6. PlayerShuffle - Skipped (requires real deck data)"
echo "  7. PlayerShuffle - Skipped (requires real deck data)"
echo "  8. DealCardsTo - Dealt cards to Player 1"
echo "  9. DealCardsTo - Dealt cards to Player 2"
echo "  10. PlayerDealCards - Skipped (requires real decryption data)"
echo ""
echo "Next Steps to Complete the Game:"
echo "1. Generate actual ZK proofs using your circuits:"
echo "   - Run shuffle_encrypt circuit for Player 1"
echo "   - Run shuffle_encrypt circuit for Player 2"
echo "   - Run decrypt circuit for each card being dealt"
echo ""
echo "2. Extract the required data:"
echo "   - Shuffled deck coordinates (x0, x1 arrays for 52 cards)"
echo "   - Selector bitmaps (BitMap256 format)"
echo "   - Decrypted card coordinates"
echo "   - Init deltas for each decrypted card"
echo ""
echo "3. Update this script with the actual values and uncomment the execute commands"
echo ""
echo "Query commands for debugging:"
echo "  Game State:"
echo "  xiond query wasm contract-state smart $CONTRACT_ADDRESS '{\"game_state\": {\"game_id\": $GAME_ID}}' --node $RPC_URL"
echo ""
echo "  Current Player Index:"
echo "  xiond query wasm contract-state smart $CONTRACT_ADDRESS '{\"cur_player_index\": {\"game_id\": $GAME_ID}}' --node $RPC_URL"
echo ""
echo "  Deck:"
echo "  xiond query wasm contract-state smart $CONTRACT_ADDRESS '{\"deck\": {\"game_id\": $GAME_ID}}' --node $RPC_URL"
echo ""
echo "  Aggregated Public Key:"
echo "  xiond query wasm contract-state smart $CONTRACT_ADDRESS '{\"aggregated_pk\": {\"game_id\": $GAME_ID}}' --node $RPC_URL"
