# zkShuffle Test Scripts

This directory contains scripts to help test and interact with the zkShuffle CosmWasm contract on the XION blockchain.

## Prerequisites

1. **xiond CLI** installed and configured
2. **jq** for JSON processing
3. Accounts with funds for gas fees
4. Contract deployed on XION testnet

## Environment Setup

All scripts source environment variables from `.env.local` in the project root:

```bash
# Required variables in .env.local:
CONTRACT_ADDRESS=xion1shlkx2h2lwjdce5qcaylvwzsfmmn8hclk04v05xdgep58nxn7kjsx4qhx2
RPC_URL=https://rpc.xion-testnet-2.burnt.com:443
CHAIN_ID=xion-testnet-2
SATYAM2=xion1g6u0d3e025u2vkvum0c8npdx4jnc4sn2egt7u4
SATYAM3=xion13uvuntkvstwlqptd4hmn8pqv3pe0458l459lr8
```

## Scripts

### test_game.sh

Main test script that runs through the complete game flow:

```bash
cd scripts
./test_game.sh
```

**Tests:**
1. CreateGame - Creates a new game with specified number of players
2. Register - Registers players (requires 2+ accounts)
3. PlayerRegister - Registers players with public keys
4. Shuffle - Initiates the shuffle phase
5. PlayerShuffle - Each player shuffles with ZK proof (template)
6. DealCardsTo - Deals cards to a specific player
7. PlayerDealCards - Completes dealing with decryption proofs (template)

**Optional environment variables:**
- `GAME_ID` - Game ID to test (default: 1)
- `NUM_PLAYERS` - Number of players (default: 2)

### query_game.sh

Query helper for inspecting contract state:

```bash
cd scripts
./query_game.sh <command> [options]
```

**Commands:**
| Command | Description | Example |
|---------|-------------|---------|
| `game-info`, `info` | Query game information | `./query_game.sh info` |
| `game-state`, `state` | Query complete game state | `./query_game.sh state` |
| `deck` | Query current deck state | `./query_game.sh deck` |
| `num-cards` | Query number of cards | `./query_game.sh num-cards` |
| `cur-player`, `current-player` | Query current player index | `./query_game.sh cur-player` |
| `player-index <addr>` | Query player index for address | `./query_game.sh player-index $SATYAM2` |
| `aggregated-pk`, `agg-pk` | Query aggregated public key | `./query_game.sh agg-pk` |
| `decrypt-record <idx>` | Query decrypt record for card | `./query_game.sh decrypt-record 5` |
| `card-value <idx>` | Query value of a card | `./query_game.sh card-value 0` |
| `all` | Query all game data | `./query_game.sh all` |

**Examples:**
```bash
# Query game info for game 1
./query_game.sh info

# Query game info for game 2
GAME_ID=2 ./query_game.sh state

# Query specific player's index
./query_game.sh player-index xion1g6u0d3e025u2vkvum0c8npdx4jnc4sn2egt7u4

# Query all data
./query_game.sh all
```

## Data Files

### Proof Data

Located in `scripts/data/`:

- **`shuffle_encrypt.json`** - Sample Groth16 proof for shuffle encryption
- **`decrypt.json`** - Sample Groth16 proof for card decryption

### Sample Templates

- **`player_register_sample.json`** - Template for PlayerRegister message
- **`player_shuffle_sample.json`** - Template for PlayerShuffle message (52-card deck)
- **`player_deal_cards_sample.json`** - Template for PlayerDealCards message

## Complete Test Flow

### Step 1: Create Game

```bash
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"create_game": {"num_players": 2}}' \
  --from $SATYAM2 \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node $RPC_URL \
  --chain-id $CHAIN_ID
```

### Step 2: Register Players

```bash
# Player 1
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"register": {"game_id": 1}}' \
  --from $SATYAM2 \
  --gas auto -y --node $RPC_URL --chain-id $CHAIN_ID

# Player 2 (different account)
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"register": {"game_id": 1}}' \
  --from $SATYAM3 \
  --gas auto -y --node $RPC_URL --chain-id $CHAIN_ID
```

### Step 3: PlayerRegister with Public Key

```bash
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"player_register": {
    "game_id": 1,
    "signing_addr": "xion1g6u0d3e025u2vkvum0c8npdx4jnc4sn2egt7u4",
    "pk_x": "your_public_key_x",
    "pk_y": "your_public_key_y"
  }}' \
  --from $SATYAM2 --gas auto -y --node $RPC_URL --chain-id $CHAIN_ID
```

### Step 4: Initiate Shuffle

```bash
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"shuffle": {"game_id": 1}}' \
  --from $SATYAM2 --gas auto -y --node $RPC_URL --chain-id $CHAIN_ID
```

### Step 5: PlayerShuffle (with ZK Proof)

```bash
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"player_shuffle": {
    "game_id": 1,
    "proof": {
      "a": ["proof_a_0", "proof_a_1"],
      "b": [["proof_b_0_0", "proof_b_0_1"], ["proof_b_1_0", "proof_b_1_1"]],
      "c": ["proof_c_0", "proof_c_1"]
    },
    "deck": {
      "config": "deck52_card",
      "x0": ["card_0_x0", "card_1_x0", ...],
      "x1": ["card_0_x1", "card_1_x1", ...],
      "selector0": "bitmap_0",
      "selector1": "bitmap_1"
    }
  }}' \
  --from $SATYAM2 --gas auto -y --node $RPC_URL --chain-id $CHAIN_ID
```

### Step 6: Deal Cards

```bash
# Deal 5 cards (bitmap: 31 = binary 11111)
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"deal_cards_to": {
    "game_id": 1,
    "cards": "31",
    "player_id": 0
  }}' \
  --from $SATYAM2 --gas auto -y --node $RPC_URL --chain-id $CHAIN_ID
```

### Step 7: PlayerDealCards (with Decryption Proofs)

```bash
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"player_deal_cards": {
    "game_id": 1,
    "proofs": [
      {
        "a": ["proof_0_a_0", "proof_0_a_1"],
        "b": [["proof_0_b_0_0", "proof_0_b_0_1"], ["proof_0_b_1_0", "proof_0_b_1_1"]],
        "c": ["proof_0_c_0", "proof_0_c_1"]
      }
    ],
    "decrypted_cards": [{"x": "card_x", "y": "card_y"}],
    "init_deltas": [{"delta0": "delta0", "delta1": "delta1"}]
  }}' \
  --from $SATYAM2 --gas auto -y --node $RPC_URL --chain-id $CHAIN_ID
```

## ZK Proof Generation

To execute **PlayerShuffle** and **PlayerDealCards**, you need to:

1. **Generate ZK proofs** using your circuit:
   - Shuffle circuit for `PlayerShuffle`
   - Decrypt circuit for `PlayerDealCards`

2. **Extract the proof data** in Groth16 format:
   ```json
   {
     "a": ["field_element_1", "field_element_2"],
     "b": [["fe_1", "fe_2"], ["fe_3", "fe_4"]],
     "c": ["field_element_1", "field_element_2"]
   }
   ```

3. **Format field elements as Uint256 strings** (decimal or hex)

## Troubleshooting

### Query game state to debug

```bash
# Check current state and player turn
./query_game.sh state
./query_game.sh cur-player

# Check deck state
./query_game.sh deck

# Check player index
./query_game.sh player-index $SATYAM2
```

### Common Errors

1. **Invalid state transition**: Ensure you're following the correct state machine order
2. **Not your turn**: Check `cur_player_index` before acting
3. **Invalid proof format**: Ensure all field elements are valid Uint256 strings
4. **Gas limit**: Increase `--gas-adjustment` if transactions fail

## Notes

- All transaction scripts use `--gas auto` with `--gas-adjustment 1.3`
- Modify gas settings based on actual transaction costs
- The `test_game.sh` script provides templates for complex operations (PlayerShuffle, PlayerDealCards)
- Fill in templates with actual proof and deck data from your ZK circuit
