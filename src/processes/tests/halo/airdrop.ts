/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient,  MsgExecuteContract, Wallet } from "@cosmjs/launchpad";
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  apTeam2: string,
  pleb: string,
  airdropContract: string
): Promise<void> {
  process.stdout.write("Test - Pleb cannot update airdrop config");

  await expect(
    sendTransaction(juno, pleb, airdropContract, {
      update_config: {
        owner: apTeam2
      },
    }),
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Failed!"));

  process.stdout.write("Test - Only owner can update airdrop config");

  await expect(
    sendTransaction(juno, apTeam, airdropContract, {
      update_config: {
        owner: apTeam2
      }
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  airdropContract: string,
): Promise<void> {
  process.stdout.write("Test - Register new merkle root");

  const merkle_root = await generateMerkleRoot();
  await expect(
    sendTransaction(juno, apTeam, airdropContract, {
      register_merkle_root: { merkle_root },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  airdropContract: string
): Promise<void> {
  process.stdout.write("Test - Airdrop claim");

  await expect(
    sendTransaction(juno, apTeam, airdropContract, {
      claim: {
        stage: 1,
        amount: "1000001",
        proof: [
          "eb0422c52c8afe5bf78f199fcbff0e87eb1a8e5713a9e0b992b575035510b3d9",
          "9d5a269ba089bafdced3d362b80c516854a1c450b45b386fa186f80af5020021",
        ],
      },
    })
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryAirdropConfig(
  juno: SigningCosmWasmClient,
  airdropContract: string
): Promise<void> {
  process.stdout.write("Test - Query Airdrop Config");
  const result: any = await juno.queryContractSmart(airdropContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAirdropMerkleRoot(
  juno: SigningCosmWasmClient,
  airdropContract: string,
  stage: number
): Promise<void> {
  process.stdout.write("Test - Query Merkle Root");
  const result: any = await juno.queryContractSmart(airdropContract, {
    merkle_root: { stage },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAirdropLatestStage(
  juno: SigningCosmWasmClient,
  airdropContract: string
): Promise<void> {
  process.stdout.write("Test - Query Airdrop Latest Stage");
  const result: any = await juno.queryContractSmart(airdropContract, {
    latest_stage: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAirdropIsClaimed(
  juno: SigningCosmWasmClient,
  airdropContract: string,
  stage: number,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query Airdrop Is Claimed");
  const result: any = await juno.queryContractSmart(airdropContract, {
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
    file2 = readFileSync(path.resolve(__dirname, "./airdrop/testdata/airdrop_delegators_list.json"), 'utf-8');
  } catch (e) {
    console.error(e);
    throw e;
  }

  const stakers: Array<{ address: string; amount: string }> = JSON.parse(file1);
  const delegators: Array<{ address: string, amount: string }> = JSON.parse(file2);
  const arr = stakers.concat(delegators);
  const airdrop = new Airdrop(arr);
  const merkleRoot = airdrop.getMerkleRoot();
  console.log(merkleRoot);
  arr.forEach(element => {
    console.log(element);
    const proof = airdrop.getMerkleProof(element);
    console.log(proof); // Replace this proof when call claim
    const verify = airdrop.verify(proof, element);
    console.log(verify);
  });
  return merkleRoot;
}
