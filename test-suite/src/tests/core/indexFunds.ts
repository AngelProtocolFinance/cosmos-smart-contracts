/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
  sendMessageViaCw3Proposal,
  sendTransaction,
  sendTransactionWithFunds,
} from "../../utils/helpers/juno";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Only owner can update owner/admin of the Index Fund.
//
// SCENARIO:
// (config)Owner updates the owner address in Index Fund.
//
//----------------------------------------------------------------------------------------
export async function testIndexFundUpdateOwner(
  juno: SigningCosmWasmClient,
  apTeam: string,
  indexFund: string,
  new_owner: string
): Promise<void> {
  process.stdout.write(
    "Test - Owner can set new_owner address in an Index Fund"
  );

  const config = await juno.queryContractSmart(indexFund, { config: {} });
  const cw3 = config.owner;

  await sendMessageViaCw3Proposal(juno, apTeam, cw3, indexFund, {
    update_owner: { new_owner },
  });
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Only owner can update registrar of the Index Fund.
//
// SCENARIO:
// (config)Owner updates the owner address in Index Fund.
//
//----------------------------------------------------------------------------------------
export async function testIndexFundUpateRegistrar(
  juno: SigningCosmWasmClient,
  apTeam: string,
  indexFund: string,
  new_registrar: string
): Promise<void> {
  process.stdout.write(
    "Test - Owner can set new_registrar address in an Index Fund"
  );

  const config = await juno.queryContractSmart(indexFund, { config: {} });
  const cw3 = config.owner;

  await sendMessageViaCw3Proposal(juno, apTeam, cw3, indexFund, {
    update_registrar: { new_registrar },
  });
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Normal Donor cannot send funds to the Index Fund
//
// SCENARIO:
// Normal user sends JUNO funds to an Index Fund SC fund to have it split
// up amonst the fund's charity members.
//
//----------------------------------------------------------------------------------------
export async function testDonorSendsToIndexFund(
  juno: SigningCosmWasmClient,
  pleb: string,
  indexFund: string,
  fund_id: number,
  split: string,
  amount: string
): Promise<void> {
  process.stdout.write(
    "Test - Donor (normal pleb) can send a JUNO donation to an Index Fund fund"
  );

  await sendTransactionWithFunds(
    juno,
    pleb,
    indexFund,
    {
      deposit: { fund_id, split },
    },
    [{ denom: "ujuno", amount }]
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: TCA Member can send donations to the Index Fund
//
// SCENARIO:
// TCA Member sends JUNO funds to an Index Fund SC fund to have it split
// up amonst the active fund's charity members.
//
//----------------------------------------------------------------------------------------
export async function testTcaMemberSendsToIndexFund(
  juno: SigningCosmWasmClient,
  tca: string,
  indexFund: string
): Promise<void> {
  process.stdout.write(
    "Test - TCA Member can send a JUNO donation to an Index Fund"
  );
  await sendTransactionWithFunds(
    juno,
    tca,
    indexFund,
    {
      deposit: { fund_id: undefined, split: undefined },
    },
    [{ denom: "ujuno", amount: "300000" }]
  );

  await sendTransactionWithFunds(
    juno,
    tca,
    indexFund,
    {
      deposit: { fund_id: 3, split: undefined },
    },
    [{ denom: "ujuno", amount: "400000" }]
  );

  await sendTransactionWithFunds(
    juno,
    tca,
    indexFund,
    {
      deposit: { fund_id: 3, split: "0.76" },
    },
    [{ denom: "ujuno", amount: "400000" }]
  );
  console.log(chalk.green(" Passed!"));
}

export async function testUpdatingIndexFundOwner(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  indexFund: string,
  new_owner: string
): Promise<void> {
  process.stdout.write("AP Team updates Index Fund Owner");
  await sendMessageViaCw3Proposal(juno, apTeam, cw3, indexFund, {
    update_owner: {
      new_owner,
    },
  });
  console.log(chalk.green(" Done!"));
}

export async function testUpdatingIndexFundRegistrar(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  indexFund: string,
  new_registrar: string
): Promise<void> {
  process.stdout.write("AP Team updates Index Fund Registrar address");
  await sendMessageViaCw3Proposal(juno, apTeam, cw3, indexFund, {
    update_registrar: {
      new_registrar,
    },
  });
  console.log(chalk.green(" Done!"));
}

export async function testUpdatingIndexFundConfigs(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  indexFund: string
): Promise<void> {
  process.stdout.write("AP Team updates Index Fund configs - funding goal");
  await sendMessageViaCw3Proposal(juno, apTeam, cw3, indexFund, {
    update_config: {
      funding_goal: "10000000000",
      fund_rotation: undefined,
    },
  });
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: SC owner can update the fund members to an Index Fund
//
// SCENARIO:
// pleb cannot update fund members, only SC owner can update fund members
//
//----------------------------------------------------------------------------------------
export async function testUpdateFundMembers(
  juno: SigningCosmWasmClient,
  apTeam: string,
  indexFund: string,
  fundId: number,
  add: number[],
  remove: number[]
): Promise<void> {
  process.stdout.write("Test - SC owner can update fund members");

  const config = await juno.queryContractSmart(indexFund, { config: {} });
  const cw3 = config.owner;

  await sendMessageViaCw3Proposal(juno, apTeam, cw3, indexFund, {
    update_members: { fund_id: fundId, add: add, remove: remove },
  });
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: SC owner can create an Index Fund
//
// SCENARIO:
// Create index fund
//----------------------------------------------------------------------------------------
export async function testCreateIndexFund(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  indexFund: string,
  name: string,
  description: string,
  rotating_fund: boolean,
  members: string[]
): Promise<void> {
  process.stdout.write("Test - SC owner can create index fund");
  await sendMessageViaCw3Proposal(juno, apTeam, cw3, indexFund, {
    create_fund: {
      name: name,
      description: description,
      members: members,
      rotating_fund: rotating_fund,
    },
  });
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: SC owner can remove an Index Fund
//
// SCENARIO:
// Remove index fund
// Check if this index fund is active fund update the active fund by calling fund_rotate
//----------------------------------------------------------------------------------------
export async function testRemoveIndexFund(
  juno: SigningCosmWasmClient,
  apTeam: string,
  indexFund: string,
  fundId: number
): Promise<void> {
  process.stdout.write("Test - SC owner can remove index fund");

  const config = await juno.queryContractSmart(indexFund, { config: {} });
  const cw3 = config.owner;

  await sendMessageViaCw3Proposal(juno, apTeam, cw3, indexFund, {
    remove_fund: { fund_id: fundId },
  });
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Accounts contract can remove an Index Fund member
//
// SCENARIO:
// Remove index fund member
//----------------------------------------------------------------------------------------
export async function testIndexFundRemoveMember(
  juno: SigningCosmWasmClient,
  apTeam: string,
  indexFund: string,
  member: number
): Promise<void> {
  process.stdout.write("Test - SC owner can remove member");

  const config = await juno.queryContractSmart(indexFund, { config: {} });
  const registrar = config.registrar_contract;

  await sendTransaction(juno, apTeam, indexFund, {
    remove_member: { member },
  });
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryIndexFundConfig(
  juno: SigningCosmWasmClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund Config");
  const result: any = await juno.queryContractSmart(indexFund, {
    config: {},
  });
  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundState(
  juno: SigningCosmWasmClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund State");
  const result: any = await juno.queryContractSmart(indexFund, {
    state: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundFundDetails(
  juno: SigningCosmWasmClient,
  indexFund: string,
  fund_id: number
): Promise<void> {
  process.stdout.write("Test - Query IndexFund FundDetails");
  const result: any = await juno.queryContractSmart(indexFund, {
    fund_details: { fund_id: fund_id },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundActiveFundDetails(
  juno: SigningCosmWasmClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund ActiveFundDetails");
  const result: any = await juno.queryContractSmart(indexFund, {
    active_fund_details: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundActiveFundDonations(
  juno: SigningCosmWasmClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund ActiveFundDonations");
  const result: any = await juno.queryContractSmart(indexFund, {
    active_fund_donations: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundDeposit(
  juno: SigningCosmWasmClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund Deposit msg builder");
  const result: any = await juno.queryContractSmart(indexFund, {
    deposit: {
      token_denom: "ujuno",
      amount: "100000000",
      fund_id: 6,
      split: undefined,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundInvolvedAddress(
  juno: SigningCosmWasmClient,
  indexFund: string,
  address: string
): Promise<void> {
  process.stdout.write(
    "Test - Query IndexFund for all funds an Address is involoved with"
  );
  const result: any = await juno.queryContractSmart(indexFund, {
    involved_funds: { address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
