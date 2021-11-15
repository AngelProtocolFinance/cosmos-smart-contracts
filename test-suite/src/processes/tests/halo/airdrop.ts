/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";
import { Airdrop } from "./airdrop/airdrop";
import { readFileSync } from 'fs';

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update airdrop config
//
// SCENARIO:
// Pleb cannot update contract config, only owner can update config
//
//----------------------------------------------------------------------------------------
export async function testAirdropUpdateConfig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  apTeam2: Wallet,
  pleb: Wallet,
  airdropContract: string
): Promise<void> {
  process.stdout.write("Test - Pleb cannot update airdrop config");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        airdropContract,
        {
          update_config: {
            owner: apTeam2.key.accAddress
          },
        },
      ),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Failed!"));

  process.stdout.write("Test - Only owner can update airdrop config");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        airdropContract,
        {
          update_config: {
            owner: apTeam2.key.accAddress
          },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Register new merkle root
//
// SCENARIO:
// owner can register new merkle root
//
//----------------------------------------------------------------------------------------
export async function testAirdropRegisterNewMerkleRoot(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  airdropContract: string,
): Promise<void> {
  process.stdout.write("Test - Register new merkle root");

  const merkle_root = await generateMerkleRoot();
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        airdropContract,
        {
          register_merkle_root: { merkle_root },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Claim
//
// SCENARIO:
// 
//
//----------------------------------------------------------------------------------------
export async function testAirdropClaim(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  airdropContract: string
): Promise<void> {
  process.stdout.write("Test - Airdrop claim");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        airdropContract,
        {
          claim: {
            amount: "1000001",
            stage: 1,
            proof: [
              "b8ee25ffbee5ee215c4ad992fe582f20175868bc310ad9b2b7bdf440a224b2df",
              "98d73e0a035f23c490fef5e307f6e74652b9d3688c2aa5bff70eaa65956a24e1",
              "f328b89c766a62b8f1c768fefa1139c9562c6e05bab57a2af87f35e83f9e9dcf",
              "fe19ca2434f87cadb0431311ac9a484792525eb66a952e257f68bf02b4561950",
            ],
          },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryAirdropConfig(
  terra: LocalTerra | LCDClient,
  airdropContract: string
): Promise<void> {
  process.stdout.write("Test - Query Airdrop Config");
  const result: any = await terra.wasm.contractQuery(airdropContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAirdropMerkleRoot(
  terra: LocalTerra | LCDClient,
  airdropContract: string,
  stage: number
): Promise<void> {
  process.stdout.write("Test - Query Merkle Root");
  const result: any = await terra.wasm.contractQuery(airdropContract, {
    merkle_root: { stage },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAirdropLatestStage(
  terra: LocalTerra | LCDClient,
  airdropContract: string
): Promise<void> {
  process.stdout.write("Test - Query Airdrop Latest Stage");
  const result: any = await terra.wasm.contractQuery(airdropContract, {
    latest_stage: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAirdropIsClaimed(
  terra: LocalTerra | LCDClient,
  airdropContract: string,
  stage: number,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query Airdrop Is Claimed");
  const result: any = await terra.wasm.contractQuery(airdropContract, {
    is_claimed: { stage, address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

async function generateMerkleRoot(): Promise<string> {
  let file1;
  let file2;
  try {
    file1 = readFileSync("../airdrop/testdata/airdrop_stakers_list.json", 'utf-8');
    file2 = readFileSync("../airdrop/testdata/airdrop_delegators_list.json", 'utf-8');
  } catch (e) {
    console.error(e);
    throw e;
  }

  const stakers: Array<{ address: string; amount: string }> = JSON.parse(file1);
  const delegators: Array<{ address: string, amount: string }> = JSON.parse(file2);
  const arr = stakers.concat(delegators);
  const airdrop = new Airdrop(arr);
  const merkleRoot = airdrop.getMerkleRoot();
  return merkleRoot;
}
