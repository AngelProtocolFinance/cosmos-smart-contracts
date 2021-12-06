// -------------------------------------------------------------------------------------
// MainNet(Columbus-5) test-suite
// -------------------------------------------------------------------------------------
import { LCDClient, MnemonicKey, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { mainnet as config } from "../config/constants";
import { migrateHaloContracts } from "../processes/migrateContracts/migrateHalo";
import { migrateContracts } from "../processes/migrateContracts/migration";
import { setupContracts, Member } from "../processes/setupContracts/mainnet";
import { setupHalo } from "../processes/setup/halo";
import { testExecute } from "../processes/tests/mainnet";
import { setupLBP } from "../processes/setup/lbp";
import { migrateLBPContracts } from "../processes/migrateContracts/migrateLBP";

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

// LBP contracts
let tokenCodeId: number;
let pairCodeId: number;
let factoryCodeId: number;
let factoryContract: string;
let tokenContract: string;
let pairContract: string;
let routerContract: string;
let lpTokenContract: string;
let tokenAmount: string;
let nativeTokenAmount: string;
let lbpCommissionRate: string;
let ammCommissionRate: string;

// Angel/HALO contracts
let haloAirdrop: string;
let haloCollector: string;
let haloCommunity: string;
let haloDistributor: string;
let haloGov: string;
let haloStaking: string;
let haloVesting: string;

// -------------------------------------------------------------------------------------
// initialize variables
// -------------------------------------------------------------------------------------
function initialize() {
  terra = new LCDClient({
    URL: config.networkInfo.url,
    chainID: config.networkInfo.chainId,
    gasPrices: { uusd: 0.15 },
    gasAdjustment: 1.2,
  });
  apTeam = terra.wallet(new MnemonicKey({mnemonic: config.mnemonicKeys.apTeam}));
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

  tokenCodeId = config.lbp.token_code_id;
  pairCodeId = config.lbp.pair_code_id;
  factoryCodeId = config.lbp.factory_code_id;
  factoryContract = config.lbp.factory_contract;
  tokenContract = config.lbp.token_contract;
  pairContract = config.lbp.pair_contract;
  routerContract = config.lbp.router_contract;
  lpTokenContract = config.lbp.lp_token_contract;
  tokenAmount = config.lbp.token_amount;
  nativeTokenAmount = config.lbp.native_token_amount;
  lbpCommissionRate = config.lbp.lbp_commission_rate;
  ammCommissionRate = config.lbp.amm_commission_rate;

  console.log(`Use ${chalk.cyan(factoryContract)} as TerraSwap factory`);
  console.log(`Use ${chalk.cyan(tokenContract)} as HALO token`);
  console.log(`Use ${chalk.cyan(pairContract)} as HALO/UST pair`);

  haloAirdrop = config.halo.airdrop_contract;
  haloCollector = config.halo.collector_contract;
  haloCommunity = config.halo.community_contract;
  haloDistributor = config.halo.distributor_contract;
  haloGov = config.halo.gov_contract;
  haloStaking = config.halo.staking_contract;
  haloVesting = config.halo.vesting_contract;

  console.log(`Use ${chalk.cyan(haloAirdrop)} as HALO airdrop`);
  console.log(`Use ${chalk.cyan(haloCollector)} as HALO collector`);
  console.log(`Use ${chalk.cyan(haloCommunity)} as HALO community`);
  console.log(`Use ${chalk.cyan(haloDistributor)} as HALO distributor`);
  console.log(`Use ${chalk.cyan(haloGov)} as HALO gov`);
  console.log(`Use ${chalk.cyan(haloStaking)} as HALO staking`);
  console.log(`Use ${chalk.cyan(haloVesting)} as HALO vesting`);
}

// -------------------------------------------------------------------------------------
// setup contracts
// -------------------------------------------------------------------------------------
export async function startSetupContracts(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupContracts(
    terra,
    apTeam,
    anchorMoneyMarket,
    apTreasury,
    members,
    tcaMembers,
    {
      tax_rate: "0.2",  // tax rate
      threshold_absolute_percentage: "0.50", // threshold absolute percentage
      max_voting_period_height: 1000,   // max voting period height
      max_voting_period_guardians_height: 100,    // max voting period guardians height
      fund_rotation: 10,     // index fund rotation
      turnover_to_multisig: false,   // turn over to AP Team multisig
      is_localterra: false,   // is LocalTerra
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      tax_per_block: "0.0000000259703196", // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
      funding_goal: "50000000", // funding goal
    },
  );
}

// -------------------------------------------------------------------------------------
// setup LBP contracts
// -------------------------------------------------------------------------------------
export async function startSetupLBPContracts(): Promise<void> {
  console.log(chalk.blue("\nMainnet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  const currTime = new Date().getTime() / 1000 + 100;
  const startTime = Math.round(currTime);
  const endTime = Math.round(currTime) + 3600 * 24 * 3;

  // Setup LBP contracts
  console.log(chalk.yellow("\nStep2. LBP Contracts"));
  await setupLBP(
    terra,
    apTeam,
    tokenAmount,
    nativeTokenAmount,
    lbpCommissionRate,
    haloCollector,
    startTime,
    endTime,
    undefined
  );
}

// -------------------------------------------------------------------------------------
// setup HALO contracts
// -------------------------------------------------------------------------------------
export async function startSetupHalo(): Promise<void> {
  console.log(chalk.blue("\nMainnet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  const currTime = new Date().getTime() / 1000 + 100;
  const endTime = Math.round(currTime) + 3600 * 24 * 3;

  // Setup HALO contracts
  console.log(chalk.yellow("\nStep2. Halo Contracts"));
  await setupHalo(
    terra,
    apTeam,
    registrar,
    tokenContract,        // halo_token contract
    factoryContract,      // terraswap_factory contract
    pairContract,         // pair contract
    lpTokenContract,      // staking_token: lp token of ANC-UST pair contract
    30,                   // quorum
    50,                   // threshold,
    2000,                 // voting_period,
    1000,                 // timelock_period,
    "10000000000",        // proposal_deposit,
    10,                   // snapshot_period,
    [],                   // whitelist
    "1000",               // spend_limit
    "0.2",                // reward_factor
    [[100, 200, "1000000"]],  // distribution_schedule
    12345,                // genesis_time
    endTime,
  );
}

// -------------------------------------------------------------------------------------
// migrate Angel Protocol core contracts
// -------------------------------------------------------------------------------------
export async function startMigrateContracts(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  await migrateContracts(
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
export async function startMigrateHaloContracts(): Promise<void> {
  console.log(chalk.blue("\nMainnet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate Contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  await migrateHaloContracts(
    terra,
    apTeam,
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloStaking,
    haloVesting
  );
}


// -------------------------------------------------------------------------------------
// migrate LBP contracts
// -------------------------------------------------------------------------------------
export async function startMigrateLBPContracts(): Promise<void> {
  console.log(chalk.blue("\nMainnet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate Contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  await migrateLBPContracts(
    terra,
    apTeam,
    factoryContract,
    pairContract,
    routerContract,
  );
}

// -------------------------------------------------------------------------------------
// start test
// -------------------------------------------------------------------------------------
export async function startTest(): Promise<void> {
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
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloStaking,
    haloVesting,
  );
}
