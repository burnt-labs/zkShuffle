import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import { ethers } from "hardhat";
import { exit } from "process";
import { GameTurn, ZKShuffle } from "@zk-shuffle/jssdk/src/shuffle/zkShuffle";
import { Hilo, Hilo__factory, ShuffleManager } from "../types";
import { deploy_shuffle_manager } from "helper/deploy";
import { dnld_aws, P0X_DIR, sleep } from "@zk-shuffle/jssdk";
import { resolve } from "path";

async function player_turn(SM: ShuffleManager, owner: SignerWithAddress, gameId: number) {
  console.log("Player ", owner.address.slice(0, 6).concat("..."), "init shuffle context!");
  const numCards = (await SM.getNumCards(gameId)).toNumber();

  let encryp_wasm = resolve(P0X_DIR, "./wasm/encrypt.wasm");
  let encrypt_zkey = resolve(P0X_DIR, "./zkey/encrypt.zkey");

  const player = await ZKShuffle.create(
    SM.address,
    owner,
    await ZKShuffle.generateShuffleSecret(),
    resolve(P0X_DIR, "./wasm/decrypt.wasm"),
    resolve(P0X_DIR, "./zkey/decrypt.zkey"),
    encryp_wasm,
    encrypt_zkey,
  );

  let playerIdx = await player.joinGame(gameId);
  console.log(
    "Player ",
    owner.address.slice(0, 6).concat("..."),
    "Join Game ",
    gameId,
    "assigned playerId",
    playerIdx,
  );
}
