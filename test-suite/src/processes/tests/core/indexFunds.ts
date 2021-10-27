/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  sendTransaction,
} from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Normal Donor cannot send funds to the Index Fund 
//
// SCENARIO:
// Normal user sends UST funds to an Index Fund SC fund to have it split 
// up amonst the fund's charity members. 
//
//----------------------------------------------------------------------------------------
export async function testDonorSendsToIndexFund(
  terra: LocalTerra | LCDClient,
  pleb: Wallet,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Donor (normal pleb) cannot send a UST donation to an Index Fund fund");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(pleb.key.accAddress, indexFund,
        {
          deposit: {
            fund_id: 1,
            split: undefined,
          },
        },
        { uusd: "4200000", }
      ),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: TCA Member can send donations to the Index Fund 
//
// SCENARIO:
// TCA Member sends UST funds to an Index Fund SC fund to have it split 
// up amonst the active fund's charity members. 
//
//----------------------------------------------------------------------------------------
export async function testTcaMemberSendsToIndexFund(
  terra: LocalTerra | LCDClient,
  tca: Wallet,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - TCA Member can send a UST donation to an Index Fund");
  await expect(
    sendTransaction(terra, tca, [
      new MsgExecuteContract(
        tca.key.accAddress,
        indexFund,
        { deposit: { fund_id: undefined, split: undefined, }, },
        { uusd: "30000000", }
      ),
      new MsgExecuteContract(
        tca.key.accAddress,
        indexFund,
        { deposit: { fund_id: 1, split: undefined, }, },
        { uusd: "40000000", }
      ),
      new MsgExecuteContract(
        tca.key.accAddress,
        indexFund,
        { deposit: { fund_id: 1, split: "0.76", }, },
        { uusd: "40000000", }
      ),
    ])
  )
  console.log(chalk.green(" Passed!"));
}

export async function testUpdatingIndexFundConfigs(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  indexFund: string
): Promise<void> {
  process.stdout.write("AP Team updates Index Fund configs - funding goal");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      update_config: {
        funding_goal: "10000000000",
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

export async function testUpdateAllianceMembersList(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  indexFund: string,
  new_members_list: string[]
): Promise<void> {
  process.stdout.write("AP Team updates the Index Fund's Angel Alliance Members List");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      update_tca_list: {
        new_list: new_members_list,
      }
    }),
  ]);
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
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pleb: Wallet,
  indexFund: string,
  fundId: number,
  add: string[],
  remove: string[],
): Promise<void> {
  process.stdout.write("Test - pleb cannot update fund members");
  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress, 
        indexFund,
        {
          update_members: {
            fund_id: fundId,
            add: add,
            remove: remove,
          }
        }
      )
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Passed!"));

  process.stdout.write("Test - SC owner can update fund members");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress, 
        indexFund,
        {
          update_members: { fund_id: fundId, add: add, remove: remove }
        }
      )
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryIndexFundConfig(
  terra: LocalTerra | LCDClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund Config");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    config: {},
  });
  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundState(
  terra: LocalTerra | LCDClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund State");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    state: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundTcaList(
  terra: LocalTerra | LCDClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund TcaList");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    tca_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundFundsList(
  terra: LocalTerra | LCDClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund FundsList");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    funds_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundFundDetails(
  terra: LocalTerra | LCDClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund FundDetails");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    fund_details: { fund_id: 1 },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundActiveFundDetails(
  terra: LocalTerra | LCDClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund ActiveFundDetails");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    active_fund_details: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundActiveFundDonations(
  terra: LocalTerra | LCDClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund ActiveFundDonations");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    active_fund_donations: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundDeposit(
  terra: LocalTerra | LCDClient,
  indexFund: string
): Promise<void> {
  process.stdout.write("Test - Query IndexFund Deposit msg builder");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    deposit: {
      amount: "100000000",
      fund_id: undefined
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
