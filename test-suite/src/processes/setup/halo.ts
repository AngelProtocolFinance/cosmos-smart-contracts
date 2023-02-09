/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { storeCode, instantiateContract } from "../../utils/juno/helpers";
import { wasm_path } from "../../config/wasmPaths";

// // Deploy HALO/DANO contracts
// export async function setupHalo(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   registrar_contract: string,
//   junoswapHaloToken: string,
//   junoswapFactory: string,
//   junoswapHaloLpToken: string,
//   quorum: number,
//   threshold: number,
//   voting_period: number,
//   timelock_period: number,
//   proposal_deposit: string,
//   snapshot_period: number,
//   unbonding_period: number,
//   whitelist: string[],
//   spend_limit: string,
//   reward_factor: string,
//   distribution_schedule: [number, number, string][],
//   genesis_time: number
// ): Promise<void> {
//   // Setup Governance contract
//   const govContract = await setupGov(
//     juno,
//     apTeam,
//     registrar_contract,
//     junoswapHaloToken,
//     quorum,
//     threshold,
//     voting_period,
//     timelock_period,
//     proposal_deposit,
//     snapshot_period,
//     unbonding_period,
//     "juno1vn8ycrkmm8llqcu82qe3sg5ktn6hajs6tkpnx0"
//   );

//   // Setup Gov Hodler contract
//   const govHodlerContract = await setupGovHodler(
//     juno,
//     apTeam,
//     junoswapHaloToken,
//     govContract
//   );

//   // Setup Distributor contract
//   const distributorContract = await setupDistributor(
//     juno,
//     apTeam,
//     junoswapHaloToken,
//     govContract,
//     whitelist,
//     spend_limit
//   );

//   // Setup Collector contract
//   await setupCollector(
//     juno,
//     apTeam,
//     junoswapHaloToken,
//     distributorContract,
//     junoswapFactory,
//     govContract,
//     reward_factor
//   );

//   // Setup Community contract
//   await setupCommunity(juno, apTeam, junoswapHaloToken, govContract, spend_limit);

//   // Setup Staking contract
//   await setupStaking(
//     juno,
//     apTeam,
//     junoswapHaloToken,
//     junoswapHaloLpToken,
//     distribution_schedule
//   );

//   // Setup Airdrop contract
//   await setupAirdrop(juno, apTeam, junoswapHaloToken);

//   // Setup Vesting contract
//   await setupVesting(juno, apTeam, junoswapHaloToken, genesis_time);
// }

// // gov contract
// async function setupGov(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   registrar_contract: string,
//   halo_token: string,
//   quorum: number,
//   threshold: number,
//   voting_period: number,
//   timelock_period: number,
//   proposal_deposit: string,
//   snapshot_period: number,
//   unbonding_period: number,
//   gov_hodler: string
// ): Promise<string> {
//   process.stdout.write("Uploading gov contract Wasm");
//   const govCodeId = await storeCode(
//     juno,
//     apTeam,
//     path.resolve(__dirname, `${wasm_path.core}/halo_gov.wasm`)
//   );
//   console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${govCodeId}`);

//   process.stdout.write("Instantiating gov contract");
//   const govResult = await instantiateContract(juno, apTeam, apTeam, govCodeId, {
//     quorum,
//     threshold,
//     voting_period,
//     timelock_period,
//     proposal_deposit,
//     snapshot_period,
//     registrar_contract,
//     halo_token,
//     unbonding_period,
//     gov_hodler,
//   });
//   const govContractAddr = govResult.logs[0].events
//     .find((event) => {
//       return event.type == "instantiate";
//     })
//     ?.attributes.find((attribute) => {
//       return attribute.key == "_contract_address";
//     })?.value as string;
//   console.log(
//     chalk.green(" Done!"),
//     `${chalk.blue("contractAddress")}=${govContractAddr}`
//   );

//   return govContractAddr;
// }

// // gov contract
// async function setupGovHodler(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   halo_token: string,
//   govContract: string
// ): Promise<string> {
//   process.stdout.write("Uploading gov hodler contract Wasm");
//   const govHodlerCodeId = await storeCode(
//     juno,
//     apTeam,
//     path.resolve(__dirname, `${wasm_path.core}/halo_gov_hodler.wasm`)
//   );
//   console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${govHodlerCodeId}`);

//   process.stdout.write("Instantiating gov hodler contract");
//   const govHodlerResult = await instantiateContract(
//     juno,
//     apTeam,
//     apTeam,
//     govHodlerCodeId,
//     {
//       halo_token,
//       gov_contract: govContract,
//     }
//   );
//   const govHodlerContractAddr = govHodlerResult.logs[0].events
//     .find((event) => {
//       return event.type == "instantiate";
//     })
//     ?.attributes.find((attribute) => {
//       return attribute.key == "_contract_address";
//     })?.value as string;
//   console.log(
//     chalk.green(" Done!"),
//     `${chalk.blue("contractAddress")}=${govHodlerContractAddr}`
//   );

//   return govHodlerContractAddr;
// }

// // airdrop contract
// async function setupAirdrop(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   haloTokenContract: string
// ): Promise<void> {
//   process.stdout.write("Uploading airdrop contract Wasm");
//   const airdropCodeId = await storeCode(
//     juno,
//     apTeam,
//     path.resolve(__dirname, `${wasm_path.core}/halo_airdrop.wasm`)
//   );
//   console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${airdropCodeId}`);

//   process.stdout.write("Instantiating airdrop contract");
//   const airdropResult = await instantiateContract(juno, apTeam, apTeam, airdropCodeId, {
//     owner: apTeam,
//     halo_token: haloTokenContract,
//   });
//   const airdropContractAddr = airdropResult.logs[0].events
//     .find((event) => {
//       return event.type == "instantiate";
//     })
//     ?.attributes.find((attribute) => {
//       return attribute.key == "_contract_address";
//     })?.value as string;
//   console.log(
//     chalk.green(" Done!"),
//     `${chalk.blue("contractAddress")}=${airdropContractAddr}`
//   );
// }

// // collector contract
// async function setupCollector(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   haloTokenContract: string,
//   distributorContract: string,
//   junoswapFactory: string,
//   govContract: string,
//   rewardFactor: string
// ): Promise<void> {
//   process.stdout.write("Uploading collector contract Wasm");
//   const collectorCodeId = await storeCode(
//     juno,
//     apTeam,
//     path.resolve(__dirname, `${wasm_path.core}/halo_collector.wasm`)
//   );
//   console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${collectorCodeId}`);

//   process.stdout.write("Instantiating collector contract");
//   const collectorResult = await instantiateContract(
//     juno,
//     apTeam,
//     apTeam,
//     collectorCodeId,
//     {
//       gov_contract: govContract,
//       swap_factory: junoswapFactory,
//       halo_token: haloTokenContract,
//       distributor_contract: distributorContract,
//       reward_factor: rewardFactor,
//     }
//   );
//   const collectorContractAddr = collectorResult.logs[0].events
//     .find((event) => {
//       return event.type == "instantiate";
//     })
//     ?.attributes.find((attribute) => {
//       return attribute.key == "_contract_address";
//     })?.value as string;
//   console.log(
//     chalk.green(" Done!"),
//     `${chalk.blue("contractAddress")}=${collectorContractAddr}`
//   );
// }

// // community contract
// async function setupCommunity(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   haloTokenContract: string,
//   govContract: string,
//   spendLimit: string
// ): Promise<void> {
//   process.stdout.write("Uploading community contract Wasm");
//   const communityCodeId = await storeCode(
//     juno,
//     apTeam,
//     path.resolve(__dirname, `${wasm_path.core}/halo_community.wasm`)
//   );
//   console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${communityCodeId}`);

//   process.stdout.write("Instantiating community contract");
//   const communityResult = await instantiateContract(
//     juno,
//     apTeam,
//     apTeam,
//     communityCodeId,
//     {
//       gov_contract: govContract,
//       halo_token: haloTokenContract,
//       spend_limit: spendLimit,
//     }
//   );
//   const communityContractAddr = communityResult.logs[0].events
//     .find((event) => {
//       return event.type == "instantiate";
//     })
//     ?.attributes.find((attribute) => {
//       return attribute.key == "_contract_address";
//     })?.value as string;
//   console.log(
//     chalk.green(" Done!"),
//     `${chalk.blue("contractAddress")}=${communityContractAddr}`
//   );
// }

// // distributor contract
// async function setupDistributor(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   haloTokenContract: string,
//   govContract: string,
//   whitelist: string[],
//   spendLimit: string
// ): Promise<string> {
//   process.stdout.write("Uploading distributor contract Wasm");
//   const distributorCodeId = await storeCode(
//     juno,
//     apTeam,
//     path.resolve(__dirname, `${wasm_path.core}/halo_distributor.wasm`)
//   );
//   console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${distributorCodeId}`);

//   process.stdout.write("Instantiating distributor contract");
//   const distributorResult = await instantiateContract(
//     juno,
//     apTeam,
//     apTeam,
//     distributorCodeId,
//     {
//       gov_contract: govContract,
//       halo_token: haloTokenContract,
//       whitelist,
//       spend_limit: spendLimit,
//     }
//   );
//   const distributorContractAddr = distributorResult.logs[0].events
//     .find((event) => {
//       return event.type == "instantiate";
//     })
//     ?.attributes.find((attribute) => {
//       return attribute.key == "_contract_address";
//     })?.value as string;
//   console.log(
//     chalk.green(" Done!"),
//     `${chalk.blue("contractAddress")}=${distributorContractAddr}`
//   );

//   return distributorContractAddr;
// }

// // staking contract
// async function setupStaking(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   haloTokenContract: string,
//   junoswapHaloLpToken: string,
//   distribution_schedule: [number, number, string][]
// ): Promise<void> {
//   process.stdout.write("Uploading staking contract Wasm");
//   const stakingCodeId = await storeCode(
//     juno,
//     apTeam,
//     path.resolve(__dirname, `${wasm_path.core}/halo_staking.wasm`)
//   );
//   console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${stakingCodeId}`);

//   process.stdout.write("Instantiating staking contract");
//   const stakingResult = await instantiateContract(juno, apTeam, apTeam, stakingCodeId, {
//     halo_token: haloTokenContract,
//     staking_token: junoswapHaloLpToken, // LP token of JunoSwap HALO-axlUSDC pair contract
//     distribution_schedule,
//   });
//   const stakingContractAddr = stakingResult.logs[0].events
//     .find((event) => {
//       return event.type == "instantiate";
//     })
//     ?.attributes.find((attribute) => {
//       return attribute.key == "_contract_address";
//     })?.value as string;
//   console.log(
//     chalk.green(" Done!"),
//     `${chalk.blue("contractAddress")}=${stakingContractAddr}`
//   );
// }

// // vesting contract
// async function setupVesting(
//   juno: SigningCosmWasmClient,
//   apTeam: DirectSecp256k1HdWallet,
//   haloTokenContract: string,
//   genesis_time: number
// ): Promise<void> {
//   process.stdout.write("Uploading vesting contract Wasm");
//   const vestingCodeId = await storeCode(
//     juno,
//     apTeam,
//     path.resolve(__dirname, `${wasm_path.core}/halo_vesting.wasm`)
//   );
//   console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vestingCodeId}`);

//   process.stdout.write("Instantiating vesting contract");
//   const vestingResult = await instantiateContract(juno, apTeam, apTeam, vestingCodeId, {
//     owner: apTeam,
//     halo_token: haloTokenContract,
//     genesis_time,
//   });
//   const vestingContractAddr = vestingResult.logs[0].events
//     .find((event) => {
//       return event.type == "instantiate";
//     })
//     ?.attributes.find((attribute) => {
//       return attribute.key == "_contract_address";
//     })?.value as string;
//   console.log(
//     chalk.green(" Done!"),
//     `${chalk.blue("contractAddress")}=${vestingContractAddr}`
//   );
// }
