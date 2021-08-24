import { program } from "commander";
program.version("0.0.1");
program.option("-n, --network <type>", "network type to deploy", "local");
program.option("-s, --store", "store all contracts in artifacts");
program.option("-h, --hash <type>", "store tx hash to instantiate");
program.option("-a, --args <type>", "instantiate arguments");
program.parse();
const { network: networkType, store, hash, args } = program.opts();

import {
  BlockTxBroadcastResult,
  isTxError,
  LCDClient,
  MnemonicKey,
  MsgStoreCode,
  MsgInstantiateContract,
  TxInfo,
  Wallet,
} from "@terra-money/terra.js";
import * as fs from "fs";
import * as path from "path";
import * as util from "util";
import { Network } from "./network.config";
import { exit } from "process";

const network = Network[networkType];
const terra = new LCDClient({
  URL: network.URL,
  chainID: network.chainID, 
  gasPrices: { uluna: 0.15 },
});

const key = new MnemonicKey({ mnemonic: network.accounts.mnemonic });
const wallet = new Wallet(terra, key);

function extOf(filename: string): string {
  if (/[.]/.exec(filename)) {
    const ext = /[^.]+$/.exec(filename)?.toString();
    if (ext) {
      return ext;
    }
  }
  throw new Error("failed to fetch extension from filename");
}

async function storeContracts(dir: string) {
  let contracts: { [contract: string]: {store_hash: string, address: string} } = {};
  const fileNames = fs.readdirSync(dir).filter((v) => extOf(v) == "wasm");
  let { sequence } = await terra.auth.accountInfo(wallet.key.accAddress);
  await fileNames.forEach(async function(fileName) {
    let path = `${dir}/${fileName}`;
    let sender;
    let codeId; 

    // read in the wasm file
    const file = fs.readFileSync(path);

    // create and sign the TX
    let result = await wallet.createAndSignTx({
      msgs: [new MsgStoreCode(wallet.key.accAddress, file.toString("base64"))],
      sequence: sequence,
    })
    .then(tx => terra.tx.broadcast(tx))
    .then(result => {
      console.log(`${fileName} - SEQ: ${sequence} - Store TX Hash: ${result.txhash}`);
      sequence += 1;
      return result;
    });
  });
}

async function init_contract(hash: string, args: string) {
  await wallet.createAndSignTx({
    msgs: [new MsgInstantiateContract(wallet.key.accAddress, undefined, 17, {
        accounts_code_id: 18,
        treasury: wallet.key.accAddress,
        tax_rate: 0.2,
      })]
    })
    .then(tx => terra.tx.broadcast(tx))
    .then(result => {
      console.log(`${"registrar"} - SEQ: ${'???'} - Init TX Hash: ${result.txhash}`);
    });
}

async function main(): Promise<void> {
  console.log(terra.config);
  const balance = await terra.bank.balance(wallet.key.accAddress);
  console.log(balance);
  try {
    if (store) {
      await storeContracts('../artifacts');
    }

    if (hash && args) {
      await init_contract(hash, args);
    }
  } catch(e) {
    console.log(e);
  }
}

main().catch(console.error);
