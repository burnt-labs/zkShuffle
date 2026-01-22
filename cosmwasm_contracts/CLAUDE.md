# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

zkShuffle is a **CosmWasm smart contract** for the XION blockchain that implements a **zero-knowledge proof-based card shuffling protocol**. It enables privacy-preserving card games (like poker) where cards can be shuffled, dealt, and revealed with cryptographic guarantees without exposing private information.

## Build Commands

### Compile to WebAssembly
```bash
cargo wasm
```
Builds the contract as a WASM library targeting `wasm32-unknown-unknown`.

### Run Unit Tests
```bash
cargo unit-test
```
Runs all unit tests in the `src/tests.rs` file.

### Optimize Contract for Deployment
```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0
```
Uses the CosmWasm optimizer to reduce binary size. Output: `artifacts/zkshuffle_cw.wasm`.

### Run a Single Test
```bash
cargo test --lib <test_name>
```

## Architecture

### Game State Machine

The contract implements a strict state machine that games progress through:

1. **Created** → Initial state after game creation
2. **Registration** → Players register with public keys
3. **Shuffle** → Each player shuffles the deck with ZK proofs
4. **Deal** → Cards are dealt to specific players
5. **Open** → Players reveal their cards
6. **Complete** → Game finished

State transitions are enforced in `contract.rs` via the `ensure_state!` macro.

### Key Source Files

| File | Purpose |
|------|---------|
| `src/contract.rs` | Main entry points: `instantiate`, `execute`, `query`. Handles all state transitions and game orchestration. |
| `src/state.rs` | Storage structures: `Config`, `GameInfo`, `ShuffleGameState`. Uses `cw-storage-plus` for efficient CosmWasm storage. |
| `src/msg.rs` | Message type definitions for `InstantiateMsg`, `ExecuteMsg`, `QueryMsg`. |
| `src/types.rs` | Core types: `Card`, `Groth16Proof`, `DeckConfig`, `CompressedDeck`. Cards are represented as elliptic curve points. |
| `src/curve.rs` | Elliptic curve operations (BLS12-381). Point addition, doubling, scalar multiplication, field arithmetic. |
| `src/deck.rs` | Deck operations: initialization, shuffling, card encryption/decryption logic. |
| `src/bitmaps.rs` | `BitMap256` utility for compact representation of card sets (which cards are selected, dealt, decrypted, etc.). |
| `src/error.rs` | Contract error types using `thiserror`. |
| `src/backup.rs` | Backup functionality (see file for details). |

### Storage Layout (from `state.rs`)

- `CONFIG`: Singleton - Global contract configuration (verifier addresses, next game ID)
- `GAME_INFOS`: Map<u64, GameInfo> - Static game metadata (deck config, player count, verifier addresses)
- `GAME_STATES`: Map<u64, ShuffleGameState> - Dynamic game state (players, deck, current phase)
- `ACTIVE_GAMES`: Map<u64, Addr> - Game owner addresses (authorization)
- `NEXT_CALLBACK`: Map<u64, Binary> - Callback messages for async operations

### Cryptographic System

The contract uses **Groth16 zk-SNARKs** for proving correct shuffling:

- **Deck Verification**: Separate verifiers for different deck sizes (5, 30, 52 cards)
- **Decryption Verification**: Verifier for card decryption proofs
- **Key Aggregation**: Players' public keys are aggregated via elliptic curve point addition
- **Card Representation**: Each card is a pair of elliptic curve points (x0, y0) and (x1, y1)

Verifier contract addresses are set during instantiation and validated against the deck configuration.

### Player Management

The contract supports two types of player addresses:
- **Player Address**: The address that submitted the registration (receives cards)
- **Signing Address**: An alternative address authorized to act on behalf of the player

This is tracked in `state.player_addrs` and `state.signing_addrs` arrays in `ShuffleGameState`.

### Turn-Based Protocol

Most game phases (`Shuffle`, `Deal`) are turn-based:
- `cur_player_index` tracks whose turn it is
- `ensure_player_turn()` in `contract.rs:567` validates the sender
- After each player action, the index increments and wraps around
- Callbacks are triggered when all players have completed their turn (index returns to 0)

### Deployment

The README.md contains XION-specific deployment instructions:
1. Install `xiond` CLI
2. Compile with Docker optimizer
3. Upload WASM to XION blockchain
4. Instantiate contract with verifier addresses
5. Interact via standard CosmWasm messages

**Note**: The README.md appears to be a template from `cw-counter` example - references to counter functionality are not applicable to this contract.

## Important Notes

- **XION-Specific Dependency**: Uses a custom fork of `cosmos-sdk-proto` from `burnt-labs/cosmos-rust` branch `feat/xion-zk` for ZK proof verification
- **Gas Optimization**: The contract is highly optimized (LTO, opt-level 3) to minimize gas costs on-chain
- **Error Handling**: Invalid state transitions return `ContractError::InvalidState` with expected/actual states
- **Authorization**: Game owners (stored in `ACTIVE_GAMES`) can trigger phase transitions; players can only act during their turn
