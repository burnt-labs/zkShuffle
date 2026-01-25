# zkShuffle Contract - Execute Methods Test Scripts

This directory contains test scripts for testing the execute methods of the zkShuffle CosmWasm contract.

## Prerequisites

1. **xiond CLI** - Install and configure the XION CLI
2. **jq** - JSON processor for parsing proof data
3. **Configured environment** - Contract must be deployed and `.env.local` configured

## Scripts

### Individual Test Scripts

#### `test_verify_shuffle_proof.sh`
Tests the `VerifyShuffleProof` execute method with proof data from `data/shuffle_encrypt.json`.

**Usage:**
```bash
./scripts/test_verify_shuffle_proof.sh
```

**What it does:**
- Loads Groth16 proof data from `data/shuffle_encrypt.json`
- Constructs a `verify_shuffle_proof` execute message
- Submits the transaction to the contract
- Queries and displays the verification count after execution

#### `test_verify_decrypt_proof.sh`
Tests the `VerifyDecryptProof` execute method with proof data from `data/decrypt.json`.

**Usage:**
```bash
./scripts/test_verify_decrypt_proof.sh
```

**What it does:**
- Loads Groth16 proof data from `data/decrypt.json`
- Constructs a `verify_decrypt_proof` execute message
- Submits the transaction to the contract
- Queries and displays the verification count after execution

### Combined Test Script

#### `test_all_proofs.sh`
Runs all proof verification tests in sequence.

**Usage:**
```bash
./scripts/test_all_proofs.sh
```

## Configuration

Environment variables are sourced from `.env.local` in the project root:

```bash
CONTRACT_ADDRESS=xion14gg9cnyzww9x572car54wp8pqskza6g5xwv0rpqnddl3gjdrknszcz0mz
RPC_URL=https://rpc.xion-testnet-2.burnt.com:443
CHAIN_ID=xion-testnet-2
SATYAM2=xion1g6u0d3e025u2vkvum0c8npdx4jnc4sn2egt7u4
```

You can override the sender account:
```bash
FROM_ACCOUNT=xion1n44pwyfczvkutwpn87e2mn2d0udht5n8mjp5yg ./scripts/test_verify_shuffle_proof.sh
```

## Data Files

The scripts use proof data from:

- **`data/shuffle_encrypt.json`** - Contains Groth16 proof for shuffle encryption verification
  - `proof.pi_a[0..1]` - Proof A values (2 x-coordinates)
  - `proof.pi_b[0..1][0..1]` - Proof B values (2x2 matrix)
  - `proof.pi_c[0..1]` - Proof C values (2 x-coordinates)
  - `publicSignals` - Array of 215 public inputs

- **`data/decrypt.json`** - Contains Groth16 proof for card decryption verification
  - `proof.pi_a[0..1]` - Proof A values (2 x-coordinates)
  - `proof.pi_b[0..1][0..1]` - Proof B values (2x2 matrix)
  - `proof.pi_c[0..1]` - Proof C values (2 x-coordinates)
  - `publicSignals` - Array of 8 public inputs

## Expected Output

### Successful Execution

```
=== zkShuffle VerifyShuffleProof Test ===
Contract: xion14gg9cnyzww9x572car54wp8pqskza6g5xwv0rpqnddl3gjdrknszcz0mz
RPC: https://rpc.xion-testnet-2.burnt.com:443
Chain ID: xion-testnet-2
From: xion1g6u0d3e025u2vkvum0c8npdx4jnc4sn2egt7u4

Loading proof data from scripts/data/shuffle_encrypt.json...
Found 215 public inputs
Proof data loaded successfully
Building execute message...

Execute Message:
{
  "verify_shuffle_proof": {
    "proof": {
      "a": ["11146830427774146572768907612957090615437997839228955028597419408343305460577", ...],
      "b": [[...], [...]],
      "c": [...]
    },
    "public_inputs": [...]
  }
}

Executing VerifyShuffleProof transaction...
---
gas estimate: 123456
...
✓ VerifyShuffleProof transaction executed successfully!

Querying verification count...
{
  "data": {
    "shuffle_verifications": "1",
    "decrypt_verifications": "0"
  }
}
```

### Error Handling

If the proof verification fails:
```
✗ VerifyShuffleProof transaction failed
Error: Invalid proof
```

This indicates:
- The proof is invalid or malformed
- The proof was generated for different public inputs
- Circuit parameters don't match the contract's verifier

## Troubleshooting

### jq not found
```bash
# Install jq on macOS
brew install jq

# Install jq on Linux
sudo apt-get install jq
```

### Permission denied
```bash
chmod +x scripts/test_verify_shuffle_proof.sh
chmod +x scripts/test_verify_decrypt_proof.sh
chmod +x scripts/test_all_proofs.sh
```

### Contract not found
- Verify `CONTRACT_ADDRESS` in `.env.local`
- Check that the contract is deployed: `xiond query wasm contract-list`

### Account not configured
- Add your account to xiond: `xiond keys add <key-name>`
- Ensure the account has funds: `xiond query bank balances <address>`

## Contract Execute Methods Reference

### VerifyShuffleProof
```json
{
  "verify_shuffle_proof": {
    "proof": {
      "a": ["Uint256", "Uint256"],
      "b": [["Uint256", "Uint256"], ["Uint256", "Uint256"]],
      "c": ["Uint256", "Uint256"]
    },
    "public_inputs": ["Uint256", ...]
  }
}
```

### VerifyDecryptProof
```json
{
  "verify_decrypt_proof": {
    "proof": {
      "a": ["Uint256", "Uint256"],
      "b": [["Uint256", "Uint256"], ["Uint256", "Uint256"]],
      "c": ["Uint256", "Uint256"]
    },
    "public_inputs": ["Uint256", ...]
  }
}
```

## Manual Execution Example

You can also execute manually using xiond:

```bash
# VerifyShuffleProof
xiond tx wasm execute $CONTRACT_ADDRESS \
  '{"verify_shuffle_proof": {"proof": {...}, "public_inputs": [...]}}' \
  --from $SATYAM2 \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y \
  --node $RPC_URL \
  --chain-id xion-testnet-2

# Query verification count
xiond query wasm contract-state smart $CONTRACT_ADDRESS \
  '{"verification_count": {}}' \
  --node $RPC_URL
```
