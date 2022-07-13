/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendTransaction, sendTransactionWithFunds } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

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

  await expect(
    sendTransactionWithFunds(juno, pleb, indexFund, {
        deposit: { fund_id, split },
      },
      [{ denom: "ujuno", amount }]
    )
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
  process.stdout.write("Test - TCA Member can send a JUNO donation to an Index Fund");
  await expect(
    sendTransactionWithFunds(juno, tca, indexFund, { 
        deposit: { fund_id: undefined, split: undefined } 
      },
      [{ denom: "ujuno", amount: "300000" }]
    )
  );
  await expect(
    sendTransactionWithFunds(juno, tca, indexFund, { 
      deposit: { fund_id: 3, split: undefined } },
      [{ denom: "ujuno", amount: "400000" }]
    )
  );
  await expect(
    sendTransactionWithFunds(juno, tca, indexFund, { 
        deposit: { fund_id: 3, split: "0.76" } },
        [{ denom: "ujuno", amount: "400000" }]
    )
  );
  console.log(chalk.green(" Passed!"));
}

export async function testUpdatingIndexFundConfigs(
  juno: SigningCosmWasmClient,
  apTeam: string,
  indexFund: string
): Promise<void> {
  process.stdout.write("AP Team updates Index Fund configs - funding goal");
  await sendTransaction(juno, apTeam, indexFund, {
    update_config: {
      funding_goal: "10000000000",
      fund_rotation: undefined,
    },
  });
  console.log(chalk.green(" Done!"));
}

export async function testUpdateAllianceMembersList(
  juno: SigningCosmWasmClient,
  apTeam: string,
  indexFund: string,
  address: string,
  member: any,
  action: string
): Promise<void> {
  process.stdout.write("AP Team updates Angel Alliance members list");
  await sendTransaction(juno, apTeam, indexFund, {
    update_alliance_member_list: { address, member, action },
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
  add: string[],
  remove: string[]
): Promise<void> {
  process.stdout.write("Test - SC owner can update fund members");
  await expect(
    sendTransaction(juno, apTeam, indexFund, {
      update_members: { fund_id: fundId, add: add, remove: remove },
    })
  );
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
  indexFund: string,
  name: string,
  description: string,
  rotating_fund: boolean,
  members: string[]
): Promise<void> {
  process.stdout.write("Test - SC owner can create index fund");
  await expect(
    sendTransaction(juno, apTeam, indexFund, {
      create_fund: {
        name: name,
        description: description,
        members: members,
        rotating_fund: rotating_fund,
      },
    })
  );
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
  await expect(
    sendTransaction(juno, apTeam, indexFund, {
      remove_fund: { fund_id: fundId },
    })
  );
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

export async function testQueryIndexFundTcaList(
  juno: SigningCosmWasmClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund TcaList");
  const result: any = await juno.queryContractSmart(indexFund, {
    alliance_members: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundFundsList(
  juno: SigningCosmWasmClient,
  indexFund: string,
  start_after: number | undefined,
  limit: number | undefined
): Promise<void> {
  process.stdout.write("Test - Query IndexFund FundsList");
  const result: any = await juno.queryContractSmart(indexFund, {
    funds_list: {
      limit,
      start_after,
    },
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
      amount: "100000000",
      fund_id: 6,
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
