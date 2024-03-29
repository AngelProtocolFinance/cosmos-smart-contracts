import * as fs from "fs";
import chalk from "chalk";
import BN from "bn.js";
import axios from "axios";
import { Coin, coin } from "@cosmjs/amino";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { wasm_path } from "../config/wasmPaths";
import { GasPrice } from "@cosmjs/stargate";

export enum VoteOption {
  YES,
  NO,
}

export type EndowmentFee = {
  payout_address: string;
  fee_percentage: string; // "0" - "1"
  active: boolean;
};

export type SplitDetails = {
  max: string;
  min: string;
  default: string;
};

export type CW4Member = {
  addr: string;
  weight: number;
};

export type Delegate = {
  address: string;
  expires?: number;
};

export type SettingsPermissions = {
  owner_controlled: boolean;
  gov_controlled: boolean;
  modifiable: boolean;
  delegate?: Delegate;
};

export type EndowmentController = {
  endowment_controller: SettingsPermissions;
  beneficiaries_allowlist: SettingsPermissions;
  contributors_allowlist: SettingsPermissions;
  maturity_allowlist: SettingsPermissions;
  donation_match_active: SettingsPermissions;
  earnings_fee: SettingsPermissions;
  withdraw_fee: SettingsPermissions;
  deposit_fee: SettingsPermissions;
  aum_fee: SettingsPermissions;
  kyc_donors_only: SettingsPermissions;
  name: SettingsPermissions;
  image: SettingsPermissions;
  logo: SettingsPermissions;
  categories: SettingsPermissions;
  ignore_user_splits: SettingsPermissions;
  split_to_liquid: SettingsPermissions;
};

export type CreateMsgCharityEndowment = {
  ref_id: string;
  meta: string;
  name: string;
  owner: string;
  tier: number;
  categories: { sdgs: number[]; general: number[] };
  logo: string;
  image: string;
  kyc_donors_only: boolean;
  referral_id?: number;
};

export type CreateMsgNormalEndowment = {
  name: string;
  owner: string;
  tier: number;
  categories: { sdgs: number[]; general: number[] };
  logo: string;
  image: string;
  kyc_donors_only: boolean;
  cw4_members: CW4Member[];
  cw3_threshold: string;
  cw3_max_voting_period: number;
  beneficiaries_allowlist: string[];
  contributors_allowlist: string[];
  earnings_fee?: EndowmentFee;
  withdraw_fee?: EndowmentFee;
  deposit_fee?: EndowmentFee;
  aum_fee?: EndowmentFee;
  dao?: any;
  proposal_link?: number;
  endowment_controller?: EndowmentController;
  parent?: number;
  split_to_liquid: SplitDetails;
  ignore_user_splits: boolean;
  referral_id?: number;
};

export type Member = {
  addr: string;
  weight: number;
};

export type Actor = {
  addr: string;
  client: SigningCosmWasmClient;
  wallet: DirectSecp256k1HdWallet;
};

export async function clientSetup(
  wallet: DirectSecp256k1HdWallet,
  networkInfo: any
) {
  const client = await SigningCosmWasmClient.connectWithSigner(
    networkInfo.url,
    wallet,
    {
      gasPrice: GasPrice.fromString(networkInfo.gasPrice),
    }
  );
  return client;
}

export async function getWalletAddress(wallet: DirectSecp256k1HdWallet) {
  const [account] = await wallet.getAccounts();
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
    const result = await juno.execute(
      sender,
      contract,
      msg,
      "auto",
      memo,
      funds
    );
    if (verbose) {
      console.log(
        chalk.yellow("\n~~~ TX HASH: ", result.transactionHash, "~~~~")
      );
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
    const result = await juno.execute(
      sender,
      contract,
      msg,
      "auto",
      memo,
      funds
    );
    if (verbose) {
      console.log(
        chalk.yellow("\n~~~ TX HASH: ", result.transactionHash, "~~~~")
      );
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
  process.stdout.write(`Uploading Wasm: ${filepath}`);
  const code = fs.readFileSync(filepath);
  const result = await juno.upload(deployer, code, "auto");
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${result.codeId}`
  );
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
  instantiateMsg: Record<string, unknown>,
  label: string | undefined = undefined
) {
  const result = await juno.instantiate(
    deployer,
    codeId,
    instantiateMsg,
    `instantiate-${label || codeId + new Date().getMilliseconds()}`,
    "auto",
    { admin: admin }
  );
  const resultContract = result.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${resultContract}`
  );
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
  const result = await juno.migrate(
    sender,
    contract,
    new_code_id,
    migrateMsg,
    "auto"
  );
  return result;
}

/**
 * @notice Existing Admin address updates a contract to a new admin address
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function updateContractAdmin(
  juno: SigningCosmWasmClient,
  sender: string,
  contract: string,
  new_admin: string
) {
  const result = await juno.updateAdmin(sender, contract, new_admin, "auto");
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
  const codeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.core}/${wasmFilename}`
  );
  process.stdout.write(`Migrate ${wasmFilename} contract`);
  const result = await migrateContract(juno, apTeam, contract, codeId, msg);
  console.log(chalk.green(" Done!"));
}

// --------------------------------------------------
// Wrapper Function:
// Stores wasm, gets new code, and migrates a contract via CW3 proposal
//---------------------------------------------------
export async function storeAndMigrateContractViaCw3(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  contract: string,
  wasmFilename: string,
  msg = {}
): Promise<void> {
  const codeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.core}/${wasmFilename}`
  );
  process.stdout.write(`Migrate ${wasmFilename} contract`);
  // push a migration message via a CW3 proposal
  const result = await sendMessagesViaCw3Proposal(
    juno,
    apTeam,
    cw3,
    `Migrate protected contract: ${contract}`,
    [
      {
        wasm: {
          migrate: {
            contract_addr: contract,
            new_code_id: codeId,
            msg: toEncodedBinary(msg),
          },
        },
      },
    ]
  );
  console.log(chalk.green(" Done!"));
}

// --------------------------------------------------
// Wrapper Function:
// Stores wasm, gets new code, and instantiates a new contract
//---------------------------------------------------
export async function storeAndInstantiateContract(
  juno: SigningCosmWasmClient,
  deployer: string,
  admin: string, // leave this emtpy then contract is not migratable
  wasmFilename: string,
  instantiateMsg: Record<string, unknown>,
  label: string | undefined = undefined
) {
  const codeId = await storeCode(
    juno,
    deployer,
    `${wasm_path.core}/${wasmFilename}`
  );

  process.stdout.write(`Instantiating ${wasmFilename} contract`);
  const result = await juno.instantiate(
    deployer,
    codeId,
    instantiateMsg,
    `instantiate-${label || codeId + new Date().getMilliseconds()}`,
    "auto",
    { admin: admin }
  );
  const resultContract = result.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${resultContract}`
  );
  return resultContract;
}

//----------------------------------------------------------------------------------------
// Abstract away steps to send a message to another contract via a CW3 multisig poll:
// 1. Create a proposal on a CW3 to execute some msg on a target contract
// 2. Capture the Proposal ID and other important info
// 3. Optional: Addtional CW3 member(s) vote on the open poll
// 4. Proposal needs to be executed
//----------------------------------------------------------------------------------------
export async function sendMessageViaCw3Proposal(
  juno: SigningCosmWasmClient,
  proposor: string,
  cw3: string,
  target_contract: string,
  msg: Record<string, unknown>
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
  const proposal_id = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value) as string;
  const proposal_status = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "status";
    })?.value) as string;
  const proposal_auto_executed = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "auto-executed";
    })?.value) as string;
  console.log(
    chalk.yellow(
      `> Proposal ID: ${proposal_id}; Status: ${proposal_status}; Auto-Executed: ${proposal_auto_executed}`
    )
  );
}

//----------------------------------------------------------------------------------------
// Abstract away steps to send a message to another contract via a CW3 Endowment poll:
// 1. Create a proposal on a CW3 to execute some msg on a target contract
// 2. Capture the Proposal ID and other important info
// 3. Proposal needs to be executed
//----------------------------------------------------------------------------------------
export async function sendMessageViaCw3Endowment(
  juno: SigningCosmWasmClient,
  proposor: string,
  cw3: string,
  target_contract: string,
  msg: Record<string, unknown>
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
  const proposal_id = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value) as string;
  const proposal_status = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "status";
    })?.value) as string;

  // 3. Execute the proposal with ID
  const res = await sendTransaction(juno, proposor, cw3, {
    execute: {
      proposal_id: parseInt(proposal_id),
    },
  });
  console.log(
    chalk.yellow(
      `> Proposal ID: ${proposal_id}; Status: ${proposal_status}; Auto-Executed: ${true}`
    )
  );
}

export async function sendMessagesViaCw3Proposal(
  juno: SigningCosmWasmClient,
  proposor: string,
  cw3: string,
  description: string,
  msgs: any[]
  // members: (SigningCosmWasmClient, string)[], // only needed if more votes required than initial proposor
): Promise<void> {
  console.log(chalk.yellow("\n> Creating CW3 Proposal"));
  const info_text = `CW3 Member proposes to: ${description}`;

  // 1. Create the new proposal
  const proposal = await sendTransaction(juno, proposor, cw3, {
    propose: {
      title: info_text,
      description: info_text,
      msgs,
    },
  });

  // 2. Parse out the proposal ID
  const proposal_id = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value) as string;
  const proposal_status = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "status";
    })?.value) as string;
  const proposal_auto_executed = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "auto-executed";
    })?.value) as string;
  console.log(
    chalk.yellow(
      `> Proposal ID: ${proposal_id}; Status: ${proposal_status}; Auto-Executed: ${proposal_auto_executed}`
    )
  );
}

//----------------------------------------------------------------------------------------
// Abstract away steps to send an Application proposal message to Review Team CW3 multisig and approve:
// 1. Create Application Proposal on CW3 to execute endowment create msg on Accounts contract
// 2. Capture the new Proposal's ID
// 3. Optional: Addtional CW3 member(s) vote on the open poll
// 4. Proposal needs to be executed and new endowment ID captured
//----------------------------------------------------------------------------------------
export async function sendApplicationViaCw3Proposal(
  networkInfo: any,
  proposor: DirectSecp256k1HdWallet,
  cw3: string,
  target_contract: string,
  ref_id: string,
  meta: string | undefined,
  msg: Record<string, unknown>,
  members: DirectSecp256k1HdWallet[]
): Promise<number> {
  const proposor_client = await clientSetup(proposor, networkInfo);
  const proposor_wallet = await getWalletAddress(proposor);
  console.log(
    chalk.yellow(`> Charity ${proposor_wallet} submits an application proposal`)
  );
  // 1. Create the new proposal (no vote is cast here)
  const proposal = await sendTransaction(
    proposor_client,
    proposor_wallet,
    cw3,
    {
      propose_application: {
        ref_id,
        msg,
        latest: undefined,
        meta,
      },
    }
  );

  // 2. Parse out the proposal ID
  const proposal_id = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value) as string;
  const proposal_status = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "status";
    })?.value) as string;
  const proposal_auto_executed = (await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "auto-executed";
    })?.value) as string;
  console.log(
    chalk.yellow(
      `> Proposal ID: ${proposal_id}; Status: ${proposal_status}; Auto-Executed: ${proposal_auto_executed}`
    )
  );

  // 3. Additional members need to vote on proposal to get to passing threshold
  let prom = Promise.resolve();
  let endowment_id = 0;
  members.forEach((member) => {
    prom = prom.then(async () => {
      const voter_wallet = await getWalletAddress(member);
      const voter_client = await clientSetup(member, networkInfo);
      console.log(
        chalk.yellow(
          `> CW3 Review Member ${voter_wallet} votes YES on application proposal`
        )
      );
      const creation = await sendTransaction(voter_client, voter_wallet, cw3, {
        vote_application: {
          proposal_id: parseInt(proposal_id),
          vote: `yes`,
          reason: undefined,
        },
      });
      // capture the endowment ID
      endowment_id = await parseInt(
        creation.logs[0].events
          .find((event) => {
            return event.type == "wasm";
          })
          ?.attributes.find((attribute) => {
            return attribute.key == "endow_id";
          })?.value as string
      );
    });
  });
  await prom;

  //  return the new Endowment ID (if auto-executed will be a number > 0)
  return endowment_id;
}
