/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  storeCode,
  sendTransaction,
  instantiateContract,
} from "../../utils/helpers";

// Deploy HALO/DANO contracts to the Testnet
export async function setupHalo(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar_contract: string,
  tokenContract: string,
  factoryContract: string,
  pairContract: string,
  staking_token: string,
  quorum: number,
  threshold: number,
  voting_period: number,
  timelock_period: number,
  proposal_deposit: string,
  snapshot_period: number,
  whitelist: string[],
  spend_limit: string,
  reward_factor: string,
  distribution_schedule: [number, number, string][],
  genesis_time: number,
  end_time: number,
  ): Promise<void> {
  
  await setupAirdrop(terra, apTeam, tokenContract);
  const govContract = await setupGov(
    terra,
    apTeam,
    registrar_contract,
    quorum,
    threshold,
    voting_period,
    timelock_period,
    proposal_deposit,
    snapshot_period,
  );
  const distributorContract = await setupDistributor(
    terra,
    apTeam,
    tokenContract,
    govContract,
    whitelist,
    spend_limit,
  );
  const collectorContract = await setupCollector(
    terra,
    apTeam,
    factoryContract,
    tokenContract,
    distributorContract,
    govContract,
    reward_factor
  );
  await setupCommunity(
    terra,
    apTeam,
    tokenContract,
    govContract,
    spend_limit
  );
  await setupStaking(
    terra,
    apTeam,
    tokenContract,
    staking_token,
    distribution_schedule,
  );
  await setupVesting(
    terra,
    apTeam,
    tokenContract,
    genesis_time,
  );

  // TODO: update the collector contract in the LBP pair contract
  await updatePairInfo(
    terra,
    apTeam,
    factoryContract,
    undefined,
    undefined,
    undefined,
    pairContract,
    collectorContract,
    undefined,
    end_time,
  );
}

async function setupAirdrop(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenContract: string,
  ): Promise<void> {
  process.stdout.write("Uploading airdrop contract Wasm");
  const airdropCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_airdrop.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${airdropCodeId}`);

  // airdrop contract
  process.stdout.write("Instantiating airdrop contract");
  const airdropResult = await instantiateContract(terra, apTeam, apTeam, airdropCodeId, {
    owner: apTeam.key.accAddress,
    halo_token: tokenContract
  });
  const airdropContractAddr = airdropResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${airdropContractAddr}`);
}

async function setupCollector(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  tokenContract: string,
  distributorContract: string,
  govContract: string,
  rewardFactor: string,
): Promise<string> {
  process.stdout.write("Uploading collector contract Wasm");
  const collectorCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_collector.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${collectorCodeId}`);

  // collector contract
  process.stdout.write("Instantiating collector contract");
  const collectorResult = await instantiateContract(terra, apTeam, apTeam, collectorCodeId, {
    gov_contract: govContract,
    terraswap_factory: factoryContract,
    halo_token: tokenContract,
    distributor_contract: distributorContract,
    reward_factor: rewardFactor
  });
  const collectorContractAddr = collectorResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${collectorContractAddr}`);

  return collectorContractAddr;
}

async function setupCommunity(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenContract: string,
  govContract: string,
  spendLimit: string,
): Promise<void> {
  process.stdout.write("Uploading community contract Wasm");
  const communityCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_community.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${communityCodeId}`);

  // community contract
  process.stdout.write("Instantiating community contract");
  const communityResult = await instantiateContract(terra, apTeam, apTeam, communityCodeId, {
    gov_contract: govContract,
    halo_token: tokenContract,
    spend_limit: spendLimit
  });
  const communityContractAddr = communityResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${communityContractAddr}`);
}

async function setupDistributor(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenContract: string,
  govContract: string,
  whitelist: string[],
  spendLimit: string,
): Promise<string> {
  process.stdout.write("Uploading distributor contract Wasm");
  const distributorCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_distributor.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${distributorCodeId}`);

  // distributor contract
  process.stdout.write("Instantiating distributor contract");
  const distributorResult = await instantiateContract(terra, apTeam, apTeam, distributorCodeId, {
    gov_contract: govContract,
    halo_token: tokenContract,
    whitelist,
    spend_limit: spendLimit
  });
  const distributorContractAddr = distributorResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${distributorContractAddr}`);

  return distributorContractAddr;
}

async function setupGov(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar_contract: string,
  quorum: number,
  threshold: number,
  voting_period: number,
  timelock_period: number,
  proposal_deposit: string,
  snapshot_period: number,
): Promise<string> {
  process.stdout.write("Uploading gov contract Wasm");
  const govCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_gov.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${govCodeId}`);

  // gov contract
  process.stdout.write("Instantiating gov contract");
  const govResult = await instantiateContract(terra, apTeam, apTeam, govCodeId, {
    quorum,
    threshold,
    voting_period,
    timelock_period,
    proposal_deposit,
    snapshot_period,
    registrar_contract,
  });
  const govContractAddr = govResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${govContractAddr}`);

  return govContractAddr;
}

async function setupStaking(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenContract: string,
  stakingToken: string,
  distribution_schedule: [number, number, string][],
): Promise<void> {
  process.stdout.write("Uploading staking contract Wasm");
  const stakingCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_staking.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${stakingCodeId}`);

  // staking contract
  process.stdout.write("Instantiating staking contract");
  const stakingResult = await instantiateContract(terra, apTeam, apTeam, stakingCodeId, {
    halo_token: tokenContract,
    staking_token: stakingToken, // lp token of HALO-UST pair contract
    distribution_schedule
  });
  const stakingContractAddr = stakingResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${stakingContractAddr}`);
}

async function setupVesting(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenContract: string,
  genesis_time: number,
): Promise<void> {
  process.stdout.write("Uploading vesting contract Wasm");
  const vestingCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_vesting.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vestingCodeId}`);

  // vesting contract
  process.stdout.write("Instantiating vesting contract");
  const vestingResult = await instantiateContract(terra, apTeam, apTeam, vestingCodeId, {
    owner: apTeam.key.accAddress,
    halo_token: tokenContract,
    genesis_time
  });
  const vestingContractAddr = vestingResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${vestingContractAddr}`);
}

async function updatePairInfo(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factory_contract: string,
  owner: string | undefined,
  token_code_id: number | undefined,
  pair_code_id: number | undefined,
  pair_contract: string,
  collector_addr: string,
  commission_rate: string | undefined,
  end_time: number | undefined,
): Promise<void> {
  process.stdout.write("Test - update factory config -> pair_info in pair contract");
  
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(
      apTeam.key.accAddress,
      factory_contract,
      {
        update_config: { owner, token_code_id, pair_code_id, pair_contract, collector_addr, commission_rate, end_time },
      },
    ),
  ]);
  console.log(chalk.green(" Passed!"));
}