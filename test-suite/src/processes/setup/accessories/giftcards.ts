/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import fs from "fs";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import {
  storeAndInstantiateContract,
  getWalletAddress,
} from "../../../utils/juno/helpers";
import { wasm_path } from "../../../config/wasmPaths";

let client: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTeamAddr: string;

// setup charity endowments
export async function setupGiftcards(
  _networkInfo: any,
  juno: SigningCosmWasmClient,
  apTeamWallet: DirectSecp256k1HdWallet,
  keeper: string,
  registrar: string
): Promise<void> {
  const apTeamAddr = await getWalletAddress(apTeamWallet);
  const giftcardsResult = await storeAndInstantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    `gift_cards.wasm`,
    {
      registrar_contract: registrar,
      keeper,
    },
    "ap-GiftCards"
  );
}
