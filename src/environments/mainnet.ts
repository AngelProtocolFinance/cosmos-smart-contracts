// -------------------------------------------------------------------------------------
// MainNet(Columbus-5) test-suite
// -------------------------------------------------------------------------------------
import { LCDClient, MnemonicKey, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { mainnet as config } from "../config/constants";
import { datetimeStringToUTC } from "../utils/helpers";

import { migrateHalo } from "../processes/migrate/halo";
import { migrateCore } from "../processes/migrate/core";
import { migrateLbp } from "../processes/migrate/lbp";

import { setupCore, Member } from "../processes/setup/core/mainnet";
import { setupTerraSwap } from "../processes/setup/terraswap/realnet";
import { setupHalo } from "../processes/setup/halo";
import { setupLbp } from "../processes/setup/lbp";

import { testExecute } from "../processes/tests/mainnet";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let terra: LCDClient;
let apTeam: Wallet;

let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let anchorVault: string;
let endowmentContracts: string[];
let anchorMoneyMarket: string;
let apTreasury: string;
let members: Member[];
let tcaMembers: string[];

// TerraSwap Contracts
let terraswapTokenCode: number;
let terraswapFactory: string;
let terraswapHaloTokenContract: string;
let terraswapHaloUstPairContract: string;
let terraswapHaloUstPairLpToken: string;
let terraswapInitialHaloSupply: string;
let terraswapHaloLiquidity: string;
let terraswapNativeLiquidity: string;

// LBP contracts
let lbpFactoryContract: string;
let lbpPairContract: string;
let lbpRouterContract: string;
let lbpLpTokenContract: string;
let haloTokenAmount: string;
let nativeTokenAmount: string;
let lbp_start_time: string;
let lbp_end_time: string;
let token_start_weight: string;
let token_end_weight: string;
let native_start_weight: string;
let native_end_weight: string;
let slippage_tolerance: string | undefined;

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
function initialize() {
  terra = new LCDClient({
    URL: config.networkInfo.url,
    chainID: config.networkInfo.chainId,
    gasPrices: { uluna: 0.15 },
    gasAdjustment: 1.2,
  });
  apTeam = terra.wallet(new MnemonicKey({ mnemonic: config.mnemonicKeys.apTeam }));
  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as Angel Team`);

  registrar = config.contracts.registrar;
  cw4GrpOwners = config.contracts.cw4GrpOwners;
  cw4GrpApTeam = config.contracts.cw4GrpApTeam;
  cw3GuardianAngels = config.contracts.cw3GuardianAngels;
  cw3ApTeam = config.contracts.cw3ApTeam;
  indexFund = config.contracts.indexFund;
  anchorVault = config.contracts.anchorVault;
  endowmentContracts = [...config.contracts.endowmentContracts];
  apTreasury = config.apTreasury;
  members = [...config.members];
  tcaMembers = [];

  anchorMoneyMarket = config.anchorMoneyMarket;

  console.log(`Use ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Use ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Use ${chalk.cyan(anchorVault)} as Anchor Vault`);
  console.log(`Use ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Use ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Use ${chalk.cyan(cw4GrpOwners)} as CW4 Endowment Owners Group`);
  console.log(`Use ${chalk.cyan(cw3GuardianAngels)} as CW3 Guardian Angels MultiSig`);
  console.log(`Use ${chalk.cyan(endowmentContracts)} as Endowment Contracts`);

  terraswapTokenCode = config.terraswap.terraswap_token_code;
  terraswapFactory = config.terraswap.terraswap_factory;
  terraswapHaloTokenContract = config.terraswap.halo_token_contract;
  terraswapHaloUstPairContract = config.terraswap.halo_ust_pair_contract;
  terraswapHaloUstPairLpToken = config.terraswap.halo_ust_pair_lp_token;
  terraswapInitialHaloSupply = config.terraswap.initial_halo_supply;
  terraswapHaloLiquidity = config.terraswap.halo_liquidity;
  terraswapNativeLiquidity = config.terraswap.native_liquidity;

  console.log(`Use ${chalk.cyan(terraswapFactory)} as TerraSwap Factory`);
  console.log(`Use ${chalk.cyan(terraswapHaloTokenContract)} as TerraSwap HALO Token`);
  console.log(
    `Use ${chalk.cyan(terraswapHaloUstPairContract)} as TerraSwap HALO/UST Pair`
  );

  lbpFactoryContract = config.lbp.factory_contract;
  lbpPairContract = config.lbp.pair_contract;
  lbpRouterContract = config.lbp.router_contract;
  lbpLpTokenContract = config.lbp.lp_token_contract;
  haloTokenAmount = config.lbp.halo_token_amount;
  nativeTokenAmount = config.lbp.native_token_amount;
  lbp_start_time = config.lbp.lbp_start_time;
  lbp_end_time = config.lbp.lbp_end_time;
  token_start_weight = config.lbp.token_start_weight;
  token_end_weight = config.lbp.token_end_weight;
  native_start_weight = config.lbp.native_start_weight;
  native_end_weight = config.lbp.native_end_weight;
  slippage_tolerance = config.lbp.slippage_tolerance;

  console.log(`Use ${chalk.cyan(lbpFactoryContract)} as LBP Factory`);
  console.log(`Use ${chalk.cyan(lbpPairContract)} as LBP HALO/UST Pair`);
  console.log(`Use ${chalk.cyan(lbpRouterContract)} as LBP Router`);
  console.log(`Use ${chalk.cyan(lbpLpTokenContract)} as LBP HALO/UST Pair LP Token`);

  haloAirdrop = config.halo.airdrop_contract;
  haloCollector = config.halo.collector_contract;
  haloCommunity = config.halo.community_contract;
  haloDistributor = config.halo.distributor_contract;
  haloGov = config.halo.gov_contract;
  haloGovHodler = config.halo.gov_hodler_contract;
  haloStaking = config.halo.staking_contract;
  haloVesting = config.halo.vesting_contract;

  console.log(`Use ${chalk.cyan(haloAirdrop)} as HALO airdrop`);
  console.log(`Use ${chalk.cyan(haloCollector)} as HALO collector`);
  console.log(`Use ${chalk.cyan(haloCommunity)} as HALO community`);
  console.log(`Use ${chalk.cyan(haloDistributor)} as HALO distributor`);
  console.log(`Use ${chalk.cyan(haloGov)} as HALO gov`);
  console.log(`Use ${chalk.cyan(haloGovHodler)} as HALO gov hodler`);
  console.log(`Use ${chalk.cyan(haloStaking)} as HALO staking`);
  console.log(`Use ${chalk.cyan(haloVesting)} as HALO vesting`);
}

// -------------------------------------------------------------------------------------
// setup contracts
// -------------------------------------------------------------------------------------
export async function startSetupCore(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupCore(terra, apTeam, anchorMoneyMarket, apTreasury, members, tcaMembers, {
    tax_rate: "0.2", // tax rate
    threshold_absolute_percentage: "0.50", // threshold absolute percentage
    max_voting_period_height: 1000, // max voting period height
    max_voting_period_guardians_height: 100, // max voting period guardians height
    fund_rotation: 10, // index fund rotation
    turnover_to_multisig: false, // turn over to AP Team multisig
    is_localterra: false, // is LocalTerra
    harvest_to_liquid: "0.75", // harvest to liquid percentage
    tax_per_block: "0.0000000259703196", // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
    funding_goal: "50000000", // funding goal
  });
}

// -------------------------------------------------------------------------------------
// setup TerraSwap contracts
// -------------------------------------------------------------------------------------
export async function startSetupTerraSwap(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupTerraSwap(
    terra,
    apTeam,
    terraswapTokenCode,
    terraswapFactory,
    terraswapInitialHaloSupply,
    terraswapHaloLiquidity,
    terraswapNativeLiquidity
  );
}

// -------------------------------------------------------------------------------------
// setup LBP contracts
// -------------------------------------------------------------------------------------
export async function startSetupLbp(): Promise<void> {
  console.log(chalk.blue("\nMainnet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup LBP contracts
  console.log(chalk.yellow("\nStep2. LBP Contracts"));
  await setupLbp(
    terra,
    apTeam,
    terraswapHaloTokenContract,
    haloTokenAmount,
    nativeTokenAmount,
    "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4",
    datetimeStringToUTC(lbp_start_time),
    datetimeStringToUTC(lbp_end_time),
    token_start_weight,
    token_end_weight,
    native_start_weight,
    native_end_weight,
    undefined
  );
}

// -------------------------------------------------------------------------------------
// setup HALO contracts
// -------------------------------------------------------------------------------------
export async function startSetupHalo(): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup HALO contracts
  console.log(chalk.yellow("\nStep2. Halo Contracts"));
  await setupHalo(
    terra,
    apTeam,
    registrar,
    terraswapHaloTokenContract, // halo terraswap token contract
    terraswapFactory,
    terraswapHaloUstPairContract, // staking_token: LP token of HALO-UST pair contract
    30, // quorum
    50, // threshold,
    100000000000, // voting_period (~7 days in blocks),
    1000, // timelock_period,
    "5000000000", // proposal_deposit,
    10000000000, // snapshot_period,
    7 * 24 * 60 * 60, // unbonding_period in seconds
    [], // whitelist
    "1000000000000", // spend_limit of 1M HALO
    "1.0", // reward_factor
    [], // distribution_schedule
    12345 // genesis_time
  );
}

// -------------------------------------------------------------------------------------
// migrate Angel Protocol core contracts
// -------------------------------------------------------------------------------------
export async function startMigrateCore(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  await migrateCore(
    terra,
    apTeam,
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
export async function startMigrateHalo(): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate Contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  await migrateHalo(
    terra,
    apTeam,
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloGovHodler,
    haloStaking,
    haloVesting
  );
}

// -------------------------------------------------------------------------------------
// migrate LBP contracts
// -------------------------------------------------------------------------------------
export async function startMigrateLbp(): Promise<void> {
  console.log(chalk.blue("\nMainnet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate Contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  await migrateLbp(terra, apTeam, lbpFactoryContract, lbpPairContract, lbpRouterContract);
}

// -------------------------------------------------------------------------------------
// start test
// -------------------------------------------------------------------------------------
export async function startTests(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Test query
  await testExecute(
    terra,
    apTeam,
    registrar,
    indexFund,
    anchorVault,
    endowmentContracts[0],
    cw4GrpApTeam,
    cw4GrpOwners,
    cw3ApTeam,
    cw3GuardianAngels,
    terraswapFactory,
    terraswapHaloTokenContract,
    terraswapHaloUstPairContract,
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloStaking,
    haloVesting,
    lbpFactoryContract,
    lbpPairContract,
    lbpRouterContract,
    lbpLpTokenContract
  );
}
