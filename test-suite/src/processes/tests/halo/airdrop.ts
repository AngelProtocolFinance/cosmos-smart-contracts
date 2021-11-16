/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
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
            stage: 4,
            amount: "1000001",
            proof: [
              "9b3bdb9e3214fefc05d52e52e722ea6a536f9af86539315cbb888b2795d2cfae",
              "aad194125af54d70868455c4658de3a5723c64a46fffef6855ec73752a9aa17e",
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
    file1 = readFileSync(path.resolve(__dirname, "./airdrop/testdata/airdrop_stakers_list.json"), 'utf-8');
    // file2 = readFileSync(path.resolve(__dirname, "./airdrop/testdata/airdrop_delegators_list.json"), 'utf-8');
  } catch (e) {
    console.error(e);
    throw e;
  }

  const arr: Array<{ address: string; amount: string }> = JSON.parse(file1);
  // const delegators: Array<{ address: string, amount: string }> = JSON.parse(file2);
  // const arr = stakers.concat(delegators);
  const airdrop = new Airdrop(arr);
  const merkleRoot = airdrop.getMerkleRoot();
  return merkleRoot;
}
