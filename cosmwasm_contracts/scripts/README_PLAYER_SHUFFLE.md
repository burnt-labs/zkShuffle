# PlayerShuffle Test Script

## Quick Start

```bash
cd cosmwasm_contracts/scripts
./test_player_shuffle.sh
```

## What This Script Does

1. **Parses `shuffle_encrypt.json`** - Extracts proof and compressedDeck data
2. **Converts field names** - Maps X0/X1 → x0/x1, selector[] → selector0/selector1
3. **Builds PlayerShuffle message** - Creates properly formatted transaction
4. **Shows preview** - Displays the message before execution
5. **Executes transaction** - Submits to the contract (with confirmation)

## Key Features

- **Automatic deck config detection** - Determines deck5_card, deck30_card, or deck52_card based on array length
- **JSON string parsing** - Handles the stringified compressedDeck correctly
- **Safety confirmation** - Shows message preview and asks for confirmation before executing
- **Current state query** - Checks cur_player_index before executing

## Prerequisites

1. Game ID 1 must be in `Shuffle` state
2. Player must be registered
3. PlayerRegister must be completed
4. `.env.local` must be configured with:
   - CONTRACT_ADDRESS
   - RPC_URL
   - CHAIN_ID
   - SATYAM2 (or other account)

## Environment Variables

- `GAME_ID` - Game ID to test (default: 1)
- `CONTRACT_ADDRESS` - Contract address
- `RPC_URL` - XION RPC endpoint
- `CHAIN_ID` - Chain ID (default: xion-testnet-2)

## Troubleshooting

### Check game state before running:
```bash
./query_game.sh state
./query_game.sh cur-player
```

### If you get "Not your turn":
- Check `cur_player_index` output
- Make sure you're the current player

### If you get "Invalid state":
- Verify game is in `Shuffle` state
- Check that all players have registered

## Example Output

```
=== Testing PlayerShuffle ===
Contract: xion1...
Game ID: 1
From: xion1...

Step 1: Parsing shuffle_encrypt.json...
  ✓ Proof data extracted

Step 2: Parsing compressedDeck...
  ✓ Compressed deck parsed
    Number of cards: 52
    Deck config: deck52_card

Step 3: Building PlayerShuffle message...
  ✓ Message built

Step 4: Message preview:
{
  "player_shuffle": {
    "game_id": 1,
    "proof": {...},
    "deck": {
      "config": "deck52_card",
      "x0": [...],
      "x1": [...],
      "selector0": "4503599627370495",
      "selector1": "3075935501959818"
    }
  }
}

Execute PlayerShuffle transaction? (y/n)
```
