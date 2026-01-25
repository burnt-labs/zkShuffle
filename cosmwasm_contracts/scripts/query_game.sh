#!/bin/bash

# zkShuffle Query Helper Script
# Provides easy access to common contract queries

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

# Default game ID
GAME_ID=${GAME_ID:-1}

# Function to query contract
query() {
    local msg="$1"
    xiond query wasm contract-state smart "$CONTRACT_ADDRESS" "$msg" \
        --node "$RPC_URL" \
        --output json
}

# Function to format and display JSON
display_json() {
    local title="$1"
    local json="$2"
    echo ""
    echo "=== $title ==="
    echo "$json" | jq '.'
}

# Parse command line arguments
COMMAND="${1:-help}"

case "$COMMAND" in
    game-info|info)
        echo "Querying Game Info for game_id: $GAME_ID"
        RESULT=$(query '{"game_info": {"game_id": '"$GAME_ID"'}}')
        display_json "Game Info" "$RESULT"
        ;;

    game-state|state)
        echo "Querying Game State for game_id: $GAME_ID"
        RESULT=$(query '{"game_state": {"game_id": '"$GAME_ID"'}}')
        display_json "Game State" "$RESULT"
        ;;

    deck)
        echo "Querying Deck for game_id: $GAME_ID"
        RESULT=$(query '{"deck": {"game_id": '"$GAME_ID"'}}')
        display_json "Deck" "$RESULT"
        ;;

    num-cards)
        echo "Querying NumCards for game_id: $GAME_ID"
        RESULT=$(query '{"num_cards": {"game_id": '"$GAME_ID"'}}')
        display_json "Number of Cards" "$RESULT"
        ;;

    cur-player|current-player)
        echo "Querying Current Player Index for game_id: $GAME_ID"
        RESULT=$(query '{"cur_player_index": {"game_id": '"$GAME_ID"'}}')
        display_json "Current Player Index" "$RESULT"
        ;;

    player-index)
        ADDR="${2:-$SATYAM2}"
        echo "Querying Player Index for address: $ADDR"
        RESULT=$(query '{"player_index": {"game_id": '"$GAME_ID"', "address": "'"$ADDR"'"}}')
        display_json "Player Index" "$RESULT"
        ;;

    aggregated-pk|agg-pk)
        echo "Querying Aggregated Public Key for game_id: $GAME_ID"
        RESULT=$(query '{"aggregated_pk": {"game_id": '"$GAME_ID"'}}')
        display_json "Aggregated Public Key" "$RESULT"
        ;;

    decrypt-record)
        CARD_INDEX="${2:-0}"
        echo "Querying Decrypt Record for game_id: $GAME_ID, card_index: $CARD_INDEX"
        RESULT=$(query '{"decrypt_record": {"game_id": '"$GAME_ID"', "card_index": '"$CARD_INDEX"'}}')
        display_json "Decrypt Record" "$RESULT"
        ;;

    card-value)
        CARD_INDEX="${2:-0}"
        echo "Querying Card Value for game_id: $GAME_ID, card_index: $CARD_INDEX"
        RESULT=$(query '{"card_value": {"game_id": '"$GAME_ID"', "card_index": '"$CARD_INDEX"'}}')
        display_json "Card Value" "$RESULT"
        ;;

    all)
        echo "=== All Queries for game_id: $GAME_ID ==="
        echo ""

        echo "1. Game Info"
        query '{"game_info": {"game_id": '"$GAME_ID"'}}' | jq '.'
        echo ""

        echo "2. Game State"
        query '{"game_state": {"game_id": '"$GAME_ID"'}}' | jq '.'
        echo ""

        echo "3. Deck"
        query '{"deck": {"game_id": '"$GAME_ID"'}}' | jq '.'
        echo ""

        echo "4. Current Player"
        query '{"cur_player_index": {"game_id": '"$GAME_ID"'}}' | jq '.'
        echo ""

        echo "5. Aggregated Public Key"
        query '{"aggregated_pk": {"game_id": '"$GAME_ID"'}}' | jq '.'
        ;;

    help|*)
        echo "zkShuffle Query Helper"
        echo ""
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Environment Variables:"
        echo "  GAME_ID       Game ID to query (default: 1)"
        echo "  CONTRACT_ADDRESS  Contract address (from .env.local)"
        echo "  RPC_URL       RPC URL (from .env.local)"
        echo ""
        echo "Commands:"
        echo "  game-info, info           Query game information"
        echo "  game-state, state         Query complete game state"
        echo "  deck                      Query current deck state"
        echo "  num-cards                 Query number of cards"
        echo "  cur-player, current-player Query current player index"
        echo "  player-index <address>    Query player index for address"
        echo "  aggregated-pk, agg-pk     Query aggregated public key"
        echo "  decrypt-record <idx>      Query decrypt record for card index"
        echo "  card-value <idx>          Query value of a card"
        echo "  all                       Query all game data"
        echo ""
        echo "Examples:"
        echo "  $0 game-info                      # Query game info"
        echo "  $0 state                          # Query game state"
        echo "  $0 player-index \$SATYAM2          # Query player index"
        echo "  GAME_ID=2 $0 deck                 # Query deck for game 2"
        echo "  $0 card-value 5                   # Query value of card 5"
        echo "  $0 all                            # Query everything"
        ;;
esac
