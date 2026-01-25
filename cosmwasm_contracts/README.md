# ZK Shuffle Contract

This is a basic CosmWasm smart contract that allows you to shuffle and ecnrypt using zk shuffle circuits and decrypt.

---

## **Prerequisites**

Before deploying the contract, ensure you have the following:

1. **XION Daemon (`xiond`)**  
   Follow the official guide to install `xiond`:  
   [Interact with XION Chain: Setup XION Daemon](https://docs.burnt.com/xion/developers/featured-guides/setup-local-environment/interact-with-xion-chain-setup-xion-daemon)

2. **Docker**  
   Install and run [Docker](https://www.docker.com/get-started), as it is required to compile the contract.

---

### **Step 2: Compile and Optimize the Wasm Bytecode**

Run the following command to compile and optimize the contract:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0
```

> **Note:**  
> This step uses **CosmWasm's Optimizing Compiler**, which reduces the contract's binary size, making it more efficient for deployment.  
> Learn more [here](https://github.com/CosmWasm/optimizer).

The optimized contract will be stored as:

```
./artifacts/zkshuffle_cw.wasm
```

---

### **Step 3: Upload the Bytecode to the Blockchain**

First load your env file using `source .env.local`

Now, upload the contract to the blockchain:

```sh
RES=$(xiond tx wasm store ./artifacts/zkshuffle_cw.wasm \
      --chain-id xion-testnet-2 \
      --gas-adjustment 1.3 \
      --gas-prices 0.1uxion \
      --gas auto \
      -y --output json \
      --node $RPC_URL \
      --from $SATYAM2)
```

After running the command, **extract the transaction hash**:

```sh
echo $RES
```

Example output:

```json
{
  "height": "0",
  "txhash": "B557242F3BBF2E68D228EBF6A792C3C617C8C8C984440405A578FBBB8A385035"
}
```

Copy the transaction hash for the next step.

---

### **Step 4: Retrieve the Code ID**

Set your transaction hash:

```sh
TXHASH="your-txhash-here"
```

Query the blockchain to get the **Code ID**:

```bash
CODE_ID=$(xiond query tx $TX_HASH --node $RPC_URL --output json | jq -r '.events[-1].attributes[1].value')
```

Now, display the retrieved Code ID:

```sh
echo $CODE_ID
```

Example output:

```
1213
```

---

### **Step 5: Instantiate the Contract**

Set the contract's initialization message:

```sh
MSG='{ "count": 1 }'
```

````
MSG='{ "decrypt_verifier": "xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs","deck5_verifier": "xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs","deck30_verifier": "xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs","deck52_verifier": "xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs"}'

Instantiate the contract with the **Code ID** from the previous step:

```bash
xiond tx wasm instantiate $CODE_ID "$MSG" \
  --from $SATYAM2 \
  --label "cw-counter" \
  --gas-prices 0.025uxion \
  --gas auto \
  --gas-adjustment 1.3 \
  -y --no-admin \
  --chain-id xion-testnet-2 \
  --node $RPC_URL
```
Query Tx

```bash
xiond q tx E164205599003239112A5DE82993A1B63734E66975213DDF3D95BE9A8DD28DEF --node $RPC_URL
```
Example output:

```
gas estimate: 217976
code: 0
txhash: 09D48FE11BE8D8BD4FCE11D236D80D180E7ED7707186B1659F5BADC4EC116F30
```

Copy the new transaction hash for the next step.

---

### **Step 6: Retrieve the Contract Address**

Set the new transaction hash:

```sh
TXHASH="your-txhash-here"
```

Query the blockchain to get the **contract address**:

```sh
CONTRACT=$(xiond query tx $TXHASH \
  --node https://rpc.xion-testnet-1.burnt.com:443 \
  --output json | jq -r '.events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
```

Display the contract address:

```sh
echo $CONTRACT
```

Example output:

```
xion1v6476wrjmw8fhsh20rl4h6jadeh5sdvlhrt8jyk2szrl3pdj4musyxj6gl
```

Create Game
```bash
xiond tx wasm execute $CONTRACT_ADDRESS '{"create_game": {"num_players": 2, "deck_config": "deck52_card"}}' \
--from $SATYAM2 \
--gas-prices 0.025uxion \
--gas auto \
--gas-adjustment 1.3 \
-y \
--node $RPC_URL \
--chain-id $CHAIN_ID
```
/\*\*

-
-
- let verification_response: Binary = deps.querier.query_grpc(
  "/xion.zk.v1.Query/ProofVerify".to_string(),
  Binary::from(verification_request_bytes),
  )?;

  let res: ProofVerifyResponse = ProofVerifyResponse::decode(verification_response.as_slice())?;
  \*/
````

```
NEXT_PUBLIC_CONTRACT_ADDRESS="xion13wfpzsm45wwl2qnqdxp4je07pyxn3aqlw02jsn2m336plmrk25uspxznxh"
NEXT_PUBLIC_TREASURY_ADDRESS="xion1yssz0jv60fxu75e22l8gry7849qr0pk7vdxtn9882xtj62swsvsq34mg3q"
NEXT_PUBLIC_RPC_URL="https://rpc.xion-testnet-2.burnt.com:443"
NEXT_PUBLIC_REST_URL="https://api.xion-testnet-2.burnt.com"
RPC_URL="https://rpc.xion-testnet-2.burnt.com:443"
ACC1=xion1n44pwyfczvkutwpn87e2mn2d0udht5n8mjp5yg
WALLET=xion1n44pwyfczvkutwpn87e2mn2d0udht5n8mjp5yg
ACC2=xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs
TX_HASH=F0A6E3F5A28A9C6B5ACCCE763027E4D8632C08E7A453F7094F1594AB22F449AF
CODE_ID=1886
CHAIN_ID=xion-testnet-2
DEPLOY_TXHASH=AB9B9235CE7B6F36AC01168D0B1C59A68CBC5DB8BC60FA0E328A4F3376509991
CONTRACT_ADDRESS=xion1ax0j60ry0wj9f0ze8l54dqzw22g93a5xfaw0jv90p06mcn569g5sm3ln7l
SATYAM2=xion1g6u0d3e025u2vkvum0c8npdx4jnc4sn2egt7u4
SATYAM3=xion13uvuntkvstwlqptd4hmn8pqv3pe0458l459lr8
SATYAM4=xion1aqgk0s3vm9qkec52djw93fel9e5cfll8gezmuq
MSG='{ "decrypt_verifier": "xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs","deck5_verifier": "xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs","deck30_verifier": "xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs","deck52_verifier": "xion14gv06unzqwmg8x6ktpd0zskt50m39nzqynr4zs"}'

```
