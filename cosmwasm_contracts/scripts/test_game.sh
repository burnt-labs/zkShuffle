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
# TODO: Set this manually before running the script
GAME_ID="19"
NUM_PLAYERS=1

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
# Step 1: CreateGame
# ============================================================================
echo "=== Step 1: CreateGame ==="
echo "Creating game with $NUM_PLAYERS players..."
echo "NOTE: After running this, extract the game_id from the transaction logs"
echo "      and set it in the GAME_ID variable at the top of this script."
echo ""
CREATE_MSG='{"create_game": {"num_players": '"$NUM_PLAYERS"'}}'
# execute_tx "$PLAYER1" "$CREATE_MSG"

# Verify GAME_ID is set
if [ -z "$GAME_ID" ]; then
    echo "WARNING: GAME_ID is not set! Please extract it from the transaction"
    echo "         logs above and set it in the GAME_ID variable at line 47."
    exit 1
fi

echo "✓ Using Game ID: $GAME_ID"

sleep 10

# Query game state to verify
echo "Verifying game creation..."
GAME_STATE_MSG='{"game_state": {"game_id": '"$GAME_ID"'}}'
# query_contract "$GAME_STATE_MSG"

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
PK1_X="16440307615439000442269603422082392822244204358820265832909555845493953212588"
PK1_Y="6926910694785434576864816651706053545127762647763807566133344066895125573895"

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
# execute_tx "$PLAYER2" "$PLAYER2_REGISTER_MSG"

# sleep 10

# Query game state to check registration status
echo "Checking game state after all players registered..."
query_contract "$GAME_STATE_MSG"

sleep 10

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

# sleep 10

# ============================================================================
# Step 6: PlayerShuffle (Player 1)
# ============================================================================
echo "=== Step 6: PlayerShuffle (Player 1) ==="
echo "Loading proof and deck data from $SCRIPT_DIR/data/shuffle_encrypt.json..."

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

    echo "  ✓ Proof data extracted"

    # Parse the compressedDeck (it's a stringified JSON)
    echo "  Parsing compressedDeck..."
    COMPRESSED_DECK_STR=$(echo "$PROOF_DATA" | jq -r '.compressedDeck')

    # Create a temporary file for the parsed compressed deck
    echo "$COMPRESSED_DECK_STR" > /tmp/compressed_deck_parsed.json

    # Extract the arrays from the parsed compressed deck
    X0_ARRAY=$(cat /tmp/compressed_deck_parsed.json | jq -c '.X0')
    X1_ARRAY=$(cat /tmp/compressed_deck_parsed.json | jq -c '.X1')

    # selector has 2 elements: selector[0] and selector[1] correspond to selector0 and selector1
    SELECTOR_0=$(cat /tmp/compressed_deck_parsed.json | jq -r '.selector[0]')
    SELECTOR_1=$(cat /tmp/compressed_deck_parsed.json | jq -r '.selector[1]')

    # Count the number of cards to determine deck config
    NUM_CARDS=$(echo "$COMPRESSED_DECK_STR" | jq '.X0 | length')
    echo "  ✓ Compressed deck parsed (number of cards: $NUM_CARDS)"

    # Determine deck config based on number of cards
    # Note: Must use lowercase with underscores to match Rust enum variants
    if [ "$NUM_CARDS" = "5" ]; then
        DECK_CONFIG="deck5_card"
    elif [ "$NUM_CARDS" = "30" ]; then
        DECK_CONFIG="deck30_card"
    elif [ "$NUM_CARDS" = "52" ]; then
        DECK_CONFIG="deck52_card"
    else
        echo "  Warning: Unknown number of cards: $NUM_CARDS, defaulting to deck52_card"
        DECK_CONFIG="deck52_card"
    fi

    echo "  Deck config: $DECK_CONFIG"
    echo ""

    # Build the PlayerShuffle message
    echo "Building PlayerShuffle message for Player 1..."

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
      "selector0": {"data": "$SELECTOR_0"},
      "selector1": {"data": "$SELECTOR_1"}
    }
  }
}
EOF

    echo "  ✓ Message built"
    echo ""

    # Display the message (pretty printed)
    echo "Message preview:"
    jq '.' /tmp/player_shuffle_msg.json
    echo ""

    # Confirm before executing
    read -p "Execute PlayerShuffle transaction for Player 1? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Read the message content
        MSG_CONTENT=$(cat /tmp/player_shuffle_msg.json)

        xiond tx wasm execute "$CONTRACT_ADDRESS" \
            "$MSG_CONTENT" \
            --from "$PLAYER1" \
            --gas-prices 0.025uxion \
            --gas auto \
            --gas-adjustment 1.3 \
            -y \
            --node "$RPC_URL" \
            --chain-id "$CHAIN_ID"

        echo ""
        echo "✓ PlayerShuffle transaction submitted for Player 1!"
    else
        echo "Skipping PlayerShuffle execution for Player 1."
    fi

    # Clean up temporary files
    rm -f /tmp/compressed_deck_parsed.json
    rm -f /tmp/player_shuffle_msg.json
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

# Only deal to Player 2 if NUM_PLAYERS > 1
if [ "$NUM_PLAYERS" -gt 1 ]; then
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
else
    echo "Skipping dealing to Player 2 (NUM_PLAYERS=$NUM_PLAYERS)"
fi

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
echo "  1. CreateGame - Created game with $NUM_PLAYERS players (Game ID: $GAME_ID)"
echo "  2. Register - Started registration phase"
echo "  3. PlayerRegister (Player 1) - Registered Player 1"

if [ "$NUM_PLAYERS" -gt 1 ]; then
    echo "  4. PlayerRegister (Player 2) - Registered Player 2"
fi

echo "  5. Shuffle - Started shuffle phase"
echo "  6. PlayerShuffle - Skipped (requires real deck data)"

if [ "$NUM_PLAYERS" -gt 1 ]; then
    echo "  7. PlayerShuffle (Player 2) - Skipped (requires real deck data)"
fi

echo "  8. DealCardsTo - Dealt cards to Player 1"

if [ "$NUM_PLAYERS" -gt 1 ]; then
    echo "  9. DealCardsTo - Dealt cards to Player 2"
fi

echo "  10. PlayerDealCards - Skipped (requires real decryption data)"
echo ""
echo "Next Steps to Complete the Game:"
echo "1. Generate actual ZK proofs using your circuits:"
echo "   - Run shuffle_encrypt circuit for Player 1"

if [ "$NUM_PLAYERS" -gt 1 ]; then
    echo "   - Run shuffle_encrypt circuit for Player 2"
fi

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
