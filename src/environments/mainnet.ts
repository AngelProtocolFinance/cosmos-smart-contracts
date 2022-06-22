// -------------------------------------------------------------------------------------
// MainNet(Columbus-5) test-suite
// -------------------------------------------------------------------------------------
import { GasPrice } from "@cosmjs/stargate";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import chalk from "chalk";
import { mainnet as config } from "../config/constants";
import { datetimeStringToUTC, getWalletAddress } from "../utils/helpers";

import { migrateHalo } from "../processes/migrate/halo";
import { migrateCore } from "../processes/migrate/core";

import { setupCore, Member } from "../processes/setup/core/mainnet";
import { setupJunoSwap } from "../processes/setup/junoswap/realnet";
import { setupHalo } from "../processes/setup/halo";

import { testExecute } from "../processes/tests/mainnet";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let juno: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;

let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let anchorVault: string;
let endowmentContracts: string[];
let apTreasury: string;
let members: Member[];
let tcaMembers: string[];

// JunoSwap Contracts
let junoswapTokenCode: number;
let junoswapFactory: string;
let junoswapHaloTokenContract: string;
let junoswapHaloUstPairContract: string;
let junoswapHaloUstPairLpToken: string;
let junoswapInitialHaloSupply: string;
let junoswapHaloLiquidity: string;
let junoswapNativeLiquidity: string;

// Angel/HALO contracts
let haloAirdrop: string;
let haloCollector: string;
let haloCommunity: string;
let haloDistributor: string;
let haloGov: string;
let haloGovHodler: string;
let haloStaking: string;
let haloVesting: string;

// -------------------------------------------------------------------------------------
// initialize variables
// -------------------------------------------------------------------------------------
async function initialize() {
  apTeam = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.apTeam, { prefix: "juno" });
  apTreasury = config.mnemonicKeys.apTeam;

  const apTeamAccount = await getWalletAddress(apTeam);
  const apTreasuryAccount = await getWalletAddress(apTreasury);
  
  console.log(`Using ${chalk.cyan(apTeamAccount)} as Angel Team`);
  console.log(`Using ${chalk.cyan(apTreasuryAccount)} as Angel Protocol Treasury`);

  registrar = config.contracts.registrar;
  cw4GrpApTeam = config.contracts.cw4GrpApTeam;
  cw3ApTeam = config.contracts.cw3ApTeam;
  indexFund = config.contracts.indexFund;
  anchorVault = config.contracts.anchorVault;
  endowmentContracts = [...config.contracts.endowmentContracts];
  members = [...config.members];
  tcaMembers = [];

  console.log(`Using ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Using ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Using ${chalk.cyan(anchorVault)} as Anchor Vault`);
  console.log(`Using ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Using ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Using ${chalk.cyan(endowmentContracts)} as Endowment Contracts`);

  junoswapTokenCode = config.junoswap.junoswap_token_code;
  junoswapFactory = config.junoswap.junoswap_factory;
  junoswapHaloTokenContract = config.junoswap.halo_token_contract;
  junoswapHaloUstPairContract = config.junoswap.halo_ust_pair_contract;
  junoswapHaloUstPairLpToken = config.junoswap.halo_ust_pair_lp_token;
  junoswapInitialHaloSupply = config.junoswap.initial_halo_supply;
  junoswapHaloLiquidity = config.junoswap.halo_liquidity;
  junoswapNativeLiquidity = config.junoswap.native_liquidity;

  console.log(`Using ${chalk.cyan(junoswapFactory)} as JunoSwap Factory`);
  console.log(`Using ${chalk.cyan(junoswapHaloTokenContract)} as JunoSwap HALO Token`);
  console.log(`Using ${chalk.cyan(junoswapHaloUstPairContract)} as JunoSwap HALO/axlUSDC Pair`);

  haloAirdrop = config.halo.airdrop_contract;
  haloCollector = config.halo.collector_contract;
  haloCommunity = config.halo.community_contract;
  haloDistributor = config.halo.distributor_contract;
  haloGov = config.halo.gov_contract;
  haloGovHodler = config.halo.gov_hodler_contract;
  haloStaking = config.halo.staking_contract;
  haloVesting = config.halo.vesting_contract;

  console.log(`Using ${chalk.cyan(haloAirdrop)} as HALO airdrop`);
  console.log(`Using ${chalk.cyan(haloCollector)} as HALO collector`);
  console.log(`Using ${chalk.cyan(haloCommunity)} as HALO community`);
  console.log(`Using ${chalk.cyan(haloDistributor)} as HALO distributor`);
  console.log(`Using ${chalk.cyan(haloGov)} as HALO gov`);
  console.log(`Using ${chalk.cyan(haloGovHodler)} as HALO gov hodler`);
  console.log(`Using ${chalk.cyan(haloStaking)} as HALO staking`);
  console.log(`Using ${chalk.cyan(haloVesting)} as HALO vesting`);
}

// -------------------------------------------------------------------------------------
// setup contracts
// -------------------------------------------------------------------------------------
export async function startSetupCore(): Promise<void> {
  console.log(chalk.blue("\nMainNet juno-1"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupCore(juno, apTeam, apTreasury, members, tcaMembers, {
    tax_rate: "0.2", // tax rate
    threshold_absolute_percentage: "0.50", // threshold absolute percentage
    max_voting_period_height: 1000, // max voting period height
    max_voting_period_guardians_height: 100, // max voting period guardians height
    fund_rotation: 10, // index fund rotation
    turnover_to_multisig: false, // turn over to AP Team multisig
    is_localjuno: false, // is LocalJuno
    harvest_to_liquid: "0.75", // harvest to liquid percentage
    tax_per_block: "0.0000000259703196", // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
    funding_goal: "50000000", // funding goal
  });
}

// -------------------------------------------------------------------------------------
// setup JunoSwap contracts
// -------------------------------------------------------------------------------------
// export async function startSetupJunoSwap(): Promise<void> {
//   console.log(chalk.blue("\nMainNet Columbus-5"));

//   // Initialize environment information
//   console.log(chalk.yellow("\nStep 1. Environment Info"));
//   await initialize();

//   // Setup contracts
//   console.log(chalk.yellow("\nStep 2. Contracts Setup"));
//   await setupJunoSwap(
//     juno,
//     apTeam,
//     junoswapTokenCode,
//     junoswapFactory,
//     junoswapInitialHaloSupply,
//     junoswapHaloLiquidity,
//     junoswapNativeLiquidity
//   );
// }

// -------------------------------------------------------------------------------------
// setup HALO contracts
// -------------------------------------------------------------------------------------
// export async function startSetupHalo(): Promise<void> {
//   console.log(chalk.blue("\nLocalJuno"));

//   // Initialize environment information
//   console.log(chalk.yellow("\nStep 1. Environment Info"));
//   await initialize();

//   // Setup HALO contracts
//   console.log(chalk.yellow("\nStep2. Halo Contracts"));
//   await setupHalo(
//     juno,
//     apTeam,
//     registrar,
//     junoswapHaloTokenContract, // halo junoswap token contract
//     junoswapFactory,
//     junoswapHaloUstPairContract, // staking_token: LP token of HALO-axlUSDC pair contract
//     30, // quorum
//     50, // threshold,
//     100000000000, // voting_period (~7 days in blocks),
//     1000, // timelock_period,
//     "5000000000", // proposal_deposit,
//     10000000000, // snapshot_period,
//     7 * 24 * 60 * 60, // unbonding_period in seconds
//     [], // whitelist
//     "1000000000000", // spend_limit of 1M HALO
//     "1.0", // reward_factor
//     [], // distribution_schedule
//     12345 // genesis_time
//   );
// }

// -------------------------------------------------------------------------------------
// migrate Angel Protocol core contracts
// -------------------------------------------------------------------------------------
export async function startMigrateCore(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Migrate contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  await migrateCore(
    juno,
    apTeamAccount,
    registrar,
    indexFund,
    cw4GrpApTeam,
    cw4GrpOwners,
    cw3ApTeam,
    cw3GuardianAngels,
    [anchorVault],
    endowmentContracts
  );
}

// -------------------------------------------------------------------------------------
// migrate HALO contracts
// -------------------------------------------------------------------------------------
// export async function startMigrateHalo(): Promise<void> {
//   console.log(chalk.blue("\nLocalJuno"));

//   // Initialize environment information
//   console.log(chalk.yellow("\nStep 1. Environment Info"));
//   await initialize();

//   // Migrate Contracts
//   console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
//   await migrateHalo(
//     juno,
//     apTeam,
//     haloAirdrop,
//     haloCollector,
//     haloCommunity,
//     haloDistributor,
//     haloGov,
//     haloGovHodler,
//     haloStaking,
//     haloVesting
//   );
// }

// -------------------------------------------------------------------------------------
// start test
// -------------------------------------------------------------------------------------
export async function startTests(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Test query
  await testExecute(
    juno,
    apTeam,
    registrar,
    indexFund,
    anchorVault,
    endowmentContracts[0],
    cw4GrpApTeam,
    cw3ApTeam,
    junoswapFactory,
    junoswapHaloTokenContract,
    junoswapHaloUstPairContract,
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloStaking,
    haloVesting,
  );
}
