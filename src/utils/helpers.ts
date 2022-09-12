import * as fs from "fs";
import chalk from "chalk";
import BN from "bn.js";
import axios from "axios";
import { Coin, coin } from "@cosmjs/amino";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { wasm_path } from "../config/wasmPaths";

export type Member = {
  addr: string;
  weight: number;
};

export type Actor = {
  addr: string;
  client: SigningCosmWasmClient;
  wallet: DirectSecp256k1HdWallet;
};

export async function getWalletAddress(wallet: DirectSecp256k1HdWallet) {
  let [account] = await wallet.getAccounts();
  return account.address;
}

/**
 * @notice Encode a JSON object to base64 binary
 */
export function toEncodedBinary(obj: any): string {
  return Buffer.from(JSON.stringify(obj)).toString("base64");
}

export function datetimeStringToUTC(date: string): number {
  try {
    return Math.round(Date.parse(date) / 1000);
  } catch (err) {
    throw "Date given is not parsable";
  }
}

/**
 * @notice Send a transaction. Return result if successful, throw error if failed.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function sendTransaction(
  juno: SigningCosmWasmClient,
  sender: string,
  contract: string,
  msg: Record<string, unknown>,
  funds: Coin[] = [],
  memo = undefined,
  verbose = false
) {
  try { 
    const result = await juno.execute(sender, contract, msg, "auto", memo, funds);
    if (verbose) {
      console.log(chalk.yellow("\n~~~ TX HASH: ", result.transactionHash, "~~~~"));
      console.log(chalk.yellow(JSON.stringify(result.logs)));
    }
    return result;
  } catch (err: any) {
    throw new Error(`An error occured! | ${err.toString()}`);
  }
}

/**
 * @notice Send a transaction along with some amount of funds [Coin, ... ]. 
 * Returns result if successful, throw error if failed.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function sendTransactionWithFunds(
  juno: SigningCosmWasmClient,
  sender: string,
  contract: string,
  msg: Record<string, unknown>,
  funds: Coin[] = [],
  memo = undefined,
  verbose = false
) {
  try { 
    const result = await juno.execute(sender, contract, msg, "auto", memo, funds);
    if (verbose) {
      console.log(chalk.yellow("\n~~~ TX HASH: ", result.transactionHash, "~~~~"));
      console.log(chalk.yellow(JSON.stringify(result.logs)));
    }
    return result;
  } catch (err: any) {
    throw new Error(`An error occured! | ${err.toString()}`);
  }
}

/**
 * @notice Upload contract code to LocalJuno. Return code ID.
 */
export async function storeCode(
  juno: SigningCosmWasmClient,
  deployer: string,
  filepath: string
): Promise<number> {
  const code = fs.readFileSync(filepath);
  const result = await juno.upload(deployer, code, "auto");
  return result.codeId;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function instantiateContract(
  juno: SigningCosmWasmClient,
  deployer: string,
  admin: string, // leave this emtpy then contract is not migratable
  codeId: number,
  instantiateMsg: Record<string, unknown>
) {
  const result = await juno.instantiate(deployer, codeId, instantiateMsg, "instantiate", "auto", { admin: admin });
  return result;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function migrateContract(
  juno: SigningCosmWasmClient,
  sender: string,
  contract: string,
  new_code_id: number,
  migrateMsg: Record<string, unknown>
) {
  const result = await juno.migrate(sender, contract, new_code_id, migrateMsg, "auto");
  return result;
}

// --------------------------------------------------
// Wrapper Function:
// Stores wasm, gets new code, and migrates contract 
//---------------------------------------------------
export async function storeAndMigrateContract(
  juno: SigningCosmWasmClient,
  apTeam: string,
  contract: string,
  wasmFilename: string,
  msg = {}
): Promise<void> {
  process.stdout.write(`Uploading ${wasmFilename} Wasm`);
  const codeId = await storeCode(juno, apTeam, `${wasm_path.core}/${wasmFilename}`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write(`Migrate ${wasmFilename} contract`);
  const result = await migrateContract(juno, apTeam, contract, codeId, msg);
  console.log(chalk.green(" Done!"));
}


//----------------------------------------------------------------------------------------
// Abstract away steps to send a message to another contract via a CW3 multisig poll:
// 1. Create a proposal on a CW3 to execute some msg on a target contract
// 2. Capture the new Proposal's ID
// 3. Optional: Addtional CW3 member(s) vote on the open poll
// 4. Proposal needs to be executed
//----------------------------------------------------------------------------------------
export async function sendMessageViaCw3Proposal(
  juno: SigningCosmWasmClient,
  proposor: string,
  cw3: string,
  target_contract: string,
  msg: Record<string, unknown>,
  // members: (SigningCosmWasmClient, string)[], // only needed if more votes required than initial proposor
): Promise<void> {
  console.log(chalk.yellow("\n> Creating CW3 Proposal"));
  const info_text = `CW3 Member proposes to send msg to: ${target_contract}`;

  // 1. Create the new proposal
  const proposal = await sendTransaction(juno, proposor, cw3, {
    propose: {
      title: info_text,
      description: info_text,
      msgs: [
        {
          wasm: {
            execute: {
              contract_addr: target_contract,
              msg: toEncodedBinary(msg),
              funds: [],
            },
          },
        },
      ],
    },
  });

  // 2. Parse out the proposal ID
  const proposal_id = await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value as string;
  console.log(chalk.yellow(`> New Proposal's ID: ${proposal_id}`));

  // // 3. Additional members need to vote on proposal to get to passing threshold
  // for member in members {
  //   console.log(chalk.green(`Member votes on proposal: ${proposal_id}`));
  //   await sendTransaction(juno, proposor, cw3, {
  //     vote: {
  //       poll_id: parseInt(proposal_id),
  //       vote: VoteOption.YES,
  //     },
  //   });
  // }

  console.log(chalk.yellow("> Executing the Poll"));
  await sendTransaction(juno, proposor, cw3, {
    execute: { proposal_id: parseInt(proposal_id) }
  });
}

//----------------------------------------------------------------------------------------
// Abstract away steps to send an Application proposal message to Review Team CW3 multisig and approve:
// 1. Create Application Proposal on CW3 to execute endowment create msg on Accounts contract
// 2. Capture the new Proposal's ID
// 3. Optional: Addtional CW3 member(s) vote on the open poll
// 4. Proposal needs to be executed and new endowment ID captured
//----------------------------------------------------------------------------------------
export async function sendApplicationViaCw3Proposal(
  juno: SigningCosmWasmClient,
  proposor: string,
  cw3: string,
  target_contract: string,
  msg: Record<string, unknown>,
  // members: (SigningCosmWasmClient, string)[], // only needed if more votes required than initial proposor
): Promise<number> {
  console.log(chalk.yellow("\n> Creating CW3 Proposal"));
  const info_text = `CW3 Member proposes to send msg to: ${target_contract}`;

  // 1. Create the new proposal
  const proposal = await sendTransaction(juno, proposor, cw3, {
    propose: {
      title: info_text,
      description: info_text,
      msgs: [
        {
          wasm: {
            execute: {
              contract_addr: target_contract,
              msg: toEncodedBinary(msg),
              funds: [],
            },
          },
        },
      ],
    },
  });

  // 2. Parse out the proposal ID
  const proposal_id = await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value as string;
  console.log(chalk.yellow(`> New Proposal's ID: ${proposal_id}`));

  // // 3. Additional members need to vote on proposal to get to passing threshold
  // for member in members {
  //   console.log(chalk.green(`Member votes on proposal: ${proposal_id}`));
  //   await sendTransaction(juno, proposor, cw3, {
  //     vote: {
  //       poll_id: parseInt(proposal_id),
  //       vote: VoteOption.YES,
  //     },
  //   });
  // }

  console.log(chalk.yellow("> Executing the Poll"));
  const creation = await sendTransaction(juno, proposor, cw3, {
    execute: { proposal_id: parseInt(proposal_id) }
  });

  // capture and return the new Endowment ID
  return await parseInt(creation.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "endow_id";
    })?.value as string);
}
