// -------------------------------------------------------------------------------------
// LocalTerra test-suite
// -------------------------------------------------------------------------------------
import { LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { localterra as config } from "../config/localterraConstants";
import { datetimeStringToUTC } from "../utils/helpers";

import { migrateCore } from "../processes/migrate/core";
import { migrateHalo } from "../processes/migrate/halo";
import { migrateLbp } from "../processes/migrate/lbp";

import { setupCore } from "../processes/setup/core/testnet";
import { setupTerraSwap } from "../processes/setup/terraswap/localterra";
import { setupHalo } from "../processes/setup/halo";
import { setupLbp } from "../processes/setup/lbp";

import { testExecute } from "../processes/tests/testnet";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let terra: LocalTerra;
let apTeam: Wallet;
let apTeam2: Wallet;
let apTeam3: Wallet;
let charity1: Wallet;
let charity2: Wallet;
let charity3: Wallet;
let pleb: Wallet;
let tca: Wallet;

// Core contracts
let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let endowmentContract1: string;
let endowmentContract2: string;
let endowmentContract3: string;
let endowmentContract4: string;
let apTreasury: string;

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
  terra = new LocalTerra();
  apTeam = terra.wallets.test1;
  apTeam2 = terra.wallets.test2;
  apTeam3 = terra.wallets.test3;
  charity1 = terra.wallets.test4;
  charity2 = terra.wallets.test5;
  charity3 = terra.wallets.test6;
  pleb = terra.wallets.test7;
  tca = terra.wallets.test8;

  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as Angel Team`);
  console.log(`Use ${chalk.cyan(apTeam2.key.accAddress)} as Angel Team #2`);
  console.log(`Use ${chalk.cyan(apTeam3.key.accAddress)} as Angel Team #3`);
  console.log(`Use ${chalk.cyan(charity1.key.accAddress)} as Charity #1`);
  console.log(`Use ${chalk.cyan(charity2.key.accAddress)} as Charity #2`);
  console.log(`Use ${chalk.cyan(charity3.key.accAddress)} as Charity #3`);
  console.log(`Use ${chalk.cyan(pleb.key.accAddress)} as Pleb`);
  console.log(`Use ${chalk.cyan(tca.key.accAddress)} as TCA member`);

  registrar = config.contracts.registrar;
  cw4GrpOwners = config.contracts.cw4GrpOwners;
  cw4GrpApTeam = config.contracts.cw4GrpApTeam;
  cw3GuardianAngels = config.contracts.cw3GuardianAngels;
  cw3ApTeam = config.contracts.cw3ApTeam;
  indexFund = config.contracts.indexFund;
  endowmentContract1 = config.contracts.endowmentContract1;
  endowmentContract2 = config.contracts.endowmentContract2;
  endowmentContract3 = config.contracts.endowmentContract3;
  endowmentContract4 = config.contracts.endowmentContract4;

  apTreasury = config.apTreasury;

  console.log(`Use ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Use ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Use ${chalk.cyan(endowmentContract1)} as Endowment Contract #1`);
  console.log(`Use ${chalk.cyan(endowmentContract2)} as Endowment Contract #2`);
  console.log(`Use ${chalk.cyan(endowmentContract3)} as Endowment Contract #3`);
  console.log(`Use ${chalk.cyan(endowmentContract4)} as Endowment Contract #4`);
  console.log(`Use ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Use ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Use ${chalk.cyan(cw4GrpOwners)} as CW4 Endowment Owners Group`);
  console.log(`Use ${chalk.cyan(cw3GuardianAngels)} as CW3 Guardian Angels MultiSig`);

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
// setup core contracts
// -------------------------------------------------------------------------------------
export async function startSetupCore(): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupCore(
    terra,
    undefined,
    apTreasury,
    // wallets
    {
      apTeam,
      apTeam2,
      apTeam3,
      charity1,
      charity2,
      charity3,
      tca,
    },
    // config
    {
      tax_rate: "0.2", // tax rate
      threshold_absolute_percentage: "0.50", // threshold absolute percentage
      max_voting_period_height: 1000, // max voting period height
      max_voting_period_guardians_height: 100, // max voting period guardians height
      fund_rotation: undefined, // index fund rotation
      turnover_to_multisig: false, // turn over to AP Team multisig
      is_localterra: true, // is LocalTerra
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      tax_per_block: "0.0000000259703196", // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
      funding_goal: "500000000", // funding goal
    }
  );
}

// -------------------------------------------------------------------------------------
// setup TerraSwap contracts
// -------------------------------------------------------------------------------------
export async function startSetupTerraSwap(): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup TerraSwap contracts
  console.log(chalk.yellow("\nStep 2a. TerraSwap Contracts"));
  await setupTerraSwap(
    terra,
    apTeam,
    apTeam2,
    apTeam3,
    terraswapInitialHaloSupply,
    terraswapHaloLiquidity,
    terraswapNativeLiquidity
  );
}

// -------------------------------------------------------------------------------------
// setup LBP contracts
// -------------------------------------------------------------------------------------
export async function startSetupLbp(): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));

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
    "uusd",
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
    terraswapHaloUstPairLpToken, // staking_token: HALO-UST pair LP Token contract
    30, // quorum
    50, // threshold,
    2000, // voting_period,
    1000, // timelock_period,
    "10000000000", // proposal_deposit,
    10, // snapshot_period,
    120, // unbonding_period in seconds
    [], // whitelist
    "10000000000", // spend_limit
    "0.2", // reward_factor
    [[100, 200, "1000000"]], // distribution_schedule
    12345 // genesis_time
  );
}

// -------------------------------------------------------------------------------------
// migrate Angel Protocol core contracts
// -------------------------------------------------------------------------------------
export async function startMigrateCore(): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate Contracts
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
    [],
    [endowmentContract1, endowmentContract2, endowmentContract3, endowmentContract4]
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
  console.log(chalk.blue("\nLocalTerra"));

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
  console.log(chalk.blue("\nLocalTerra"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Test queries
  await testExecute(
    terra,
    apTeam,
    apTeam2,
    apTeam3,
    charity1,
    charity2,
    charity3,
    pleb,
    tca,
    registrar,
    indexFund,
    "undefined",
    "undefined",
    endowmentContract1,
    endowmentContract2,
    endowmentContract3,
    endowmentContract4,
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
    lbpLpTokenContract,
    slippage_tolerance
  );
}
