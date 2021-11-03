/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import {
  storeCode,
  instantiateContract,
} from "../../utils/helpers";

// Deploy HALO/DANO contracts to the Testnet
export async function setupHalo(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar_contract: string,
  halo_token: string,
  terraswap_factory: string,
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
  ): Promise<void> {
  process.stdout.write("Uploading airdrop contract Wasm");
  const airdropCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_airdrop.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${airdropCodeId}`);

  process.stdout.write("Uploading collector contract Wasm");
  const collectorCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_collector.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${collectorCodeId}`);

  process.stdout.write("Uploading community contract Wasm");
  const communityCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_community.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${communityCodeId}`);

  process.stdout.write("Uploading distributor contract Wasm");
  const distributorCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_distributor.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${distributorCodeId}`);

  process.stdout.write("Uploading gov contract Wasm");
  const govCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_gov.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${govCodeId}`);

  process.stdout.write("Uploading staking contract Wasm");
  const stakingCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_staking.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${stakingCodeId}`);

  process.stdout.write("Uploading vesting contract Wasm");
  const vestingCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/halo_vesting.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vestingCodeId}`);

  // airdrop contract
  process.stdout.write("Instantiating airdrop contract");
  const airdropResult = await instantiateContract(terra, apTeam, apTeam, airdropCodeId, {
    owner: apTeam.key.accAddress,
    halo_token: halo_token
  });
  const airdropContractAddr = airdropResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${airdropContractAddr}`);

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

  // distributor contract
  process.stdout.write("Instantiating distributor contract");
  const distributorResult = await instantiateContract(terra, apTeam, apTeam, distributorCodeId, {
    gov_contract: govContractAddr,
    halo_token,
    whitelist,
    spend_limit
  });
  const distributorContractAddr = distributorResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${distributorContractAddr}`);

  // collector contract
  process.stdout.write("Instantiating collector contract");
  const collectorResult = await instantiateContract(terra, apTeam, apTeam, collectorCodeId, {
    gov_contract: govContractAddr,
    terraswap_factory,
    halo_token,
    distributor_contract: distributorContractAddr,
    reward_factor
  });
  const collectorContractAddr = collectorResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${collectorContractAddr}`);

  // community contract
  process.stdout.write("Instantiating community contract");
  const communityResult = await instantiateContract(terra, apTeam, apTeam, communityCodeId, {
    gov_contract: govContractAddr,
    halo_token,
    spend_limit
  });
  const communityContractAddr = communityResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${communityContractAddr}`);

  // staking contract
  process.stdout.write("Instantiating staking contract");
  const stakingResult = await instantiateContract(terra, apTeam, apTeam, stakingCodeId, {
    halo_token,
    staking_token, // lp token of ANC-UST pair contract
    distribution_schedule
  });
  const stakingContractAddr = stakingResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${stakingContractAddr}`);

  // vesting contract
  process.stdout.write("Instantiating vesting contract");
  const vestingResult = await instantiateContract(terra, apTeam, apTeam, vestingCodeId, {
    owner: apTeam.key.accAddress,
    halo_token,
    genesis_time
  });
  const vestingContractAddr = vestingResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${vestingContractAddr}`);

}
