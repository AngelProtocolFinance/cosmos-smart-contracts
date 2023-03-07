/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import fs from "fs";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import {
  instantiateContract,
  getWalletAddress,
  storeCode,
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

  // store wasm
  process.stdout.write("Uploading Gift Cards Wasm");
  const giftcardsCodeId = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/gift_cards.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${giftcardsCodeId}`
  );

  // instantiate gift card contract
  process.stdout.write("Instantiating Gift Cards contract");
  const giftcardsResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    giftcardsCodeId,
    {
      registrar_contract: registrar,
      keeper,
    },
    "ap-GiftCards"
  );
  const giftcards = giftcardsResult.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${giftcards}`
  );
}
