// -------------------------------------------------------------------------------------
// TestNet(Bombay-12) test-suite
// -------------------------------------------------------------------------------------
import { LCDClient, MnemonicKey, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { testnet as config } from "../config/constants";
import { migrateHaloContracts } from "../processes/migrateContracts/migrateHalo";
import { migrateLBPContracts } from "../processes/migrateContracts/migrateLBP";
import { migrateContracts } from "../processes/migrateContracts/migration";
import { setupContracts } from "../processes/setupContracts/testnet";
import { setupHalo } from "../processes/setup/halo";
import { setupLBP } from "../processes/setup/lbp";
import { setupToken } from "../processes/setup/token";
import { testExecute } from "../processes/tests/testnet";
import { migrateAMMContracts } from "../processes/migrateContracts/migrateAMM";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let terra: LCDClient;
let apTeam: Wallet;
let apTeam2: Wallet;
let apTeam3: Wallet;
let charity1: Wallet;
let charity2: Wallet;
let charity3: Wallet;
let pleb: Wallet;
let tca: Wallet;

let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let anchorVault1: string;
let anchorVault2: string;
let endowmentContract1: string;
let endowmentContract2: string;
let endowmentContract3: string;
let endowmentContract4: string;
let anchorMoneyMarket: string;

let tokenCodeId: number;
let pairCodeId: number;
let factoryCodeId: number;
let factoryContract: string;
let tokenContract: string;
let pairContract: string;
let routerContract: string;
let apTreasury: string;

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
  apTeam2 = terra.wallet(new MnemonicKey({mnemonic: config.mnemonicKeys.apTeam2}));
  apTeam3 = terra.wallet(new MnemonicKey({mnemonic: config.mnemonicKeys.apTeam3}));
  charity1 = terra.wallet(new MnemonicKey({mnemonic: config.mnemonicKeys.charity1}));
  charity2 = terra.wallet(new MnemonicKey({mnemonic: config.mnemonicKeys.charity2}));
  charity3 = terra.wallet(new MnemonicKey({mnemonic: config.mnemonicKeys.charity3}));
  pleb = terra.wallet(new MnemonicKey({mnemonic: config.mnemonicKeys.pleb}));
  tca = terra.wallet(new MnemonicKey({mnemonic: config.mnemonicKeys.tca}));

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
  anchorVault1 = config.contracts.anchorVault1;
  anchorVault2 = config.contracts.anchorVault2;
  endowmentContract1 = config.contracts.endowmentContract1;
  endowmentContract2 = config.contracts.endowmentContract2;
  endowmentContract3 = config.contracts.endowmentContract3;
  endowmentContract4 = config.contracts.endowmentContract4;

  anchorMoneyMarket = config.anchorMoneyMarket;
  apTreasury = config.apTreasury;

  console.log(`Use ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Use ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Use ${chalk.cyan(anchorVault1)} as Anchor Vault #1`);
  console.log(`Use ${chalk.cyan(anchorVault2)} as Anchor Vault #2`);
  console.log(`Use ${chalk.cyan(endowmentContract1)} as Endowment Contract #1`);
  console.log(`Use ${chalk.cyan(endowmentContract2)} as Endowment Contract #2`);
  console.log(`Use ${chalk.cyan(endowmentContract3)} as Endowment Contract #3`);
  console.log(`Use ${chalk.cyan(endowmentContract4)} as Endowment Contract #4`);
  console.log(`Use ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Use ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Use ${chalk.cyan(cw4GrpOwners)} as CW4 Endowment Owners Group`);
  console.log(`Use ${chalk.cyan(cw3GuardianAngels)} as CW3 Guardian Angels MultiSig`);

  tokenCodeId = config.token_code_id;
  pairCodeId = config.pair_code_id;
  factoryCodeId = config.factory_code_id;
  factoryContract = config.factory_contract;
  tokenContract = config.token_contract;
  pairContract = config.pair_contract;
  routerContract = config.router_contract;

  console.log(`Use ${chalk.cyan(factoryContract)} as LBP factory`);
  console.log(`Use ${chalk.cyan(tokenContract)} as HALO token`);
  console.log(`Use ${chalk.cyan(pairContract)} as LBP HALO/UST pair`);
  console.log(`Use ${chalk.cyan(routerContract)} as LBP router`);

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
  console.log(chalk.blue("\nTestNet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupContracts(
    terra,
    anchorMoneyMarket,
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
// setup Token contracts
// -------------------------------------------------------------------------------------
export async function startSetupTokenContract(): Promise<void> {
  console.log(chalk.blue("\nTestNet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup LBP contracts
  console.log(chalk.yellow("\nStep 2a. Token Contract"));
  await setupToken(
    terra,
    apTeam,
  );
}


// -------------------------------------------------------------------------------------
// setup HALO contracts
// -------------------------------------------------------------------------------------
export async function startSetupHalo(): Promise<void> {
  console.log(chalk.blue("\nTestnet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup HALO contracts
  console.log(chalk.yellow("\nStep2. Halo Contracts"));
  await setupHalo(
    terra,
    apTeam,
    factoryContract,  // terraswap_factory contract
    pairContract,     // staking_token: lp token of ANC-UST pair contract
    30,            // quorum
    50,            // threshold,
    2000,             // voting_period,
    1000,             // timelock_period,
    "10000000000",      // proposal_deposit,
    10,               // snapshot_period,
    [],               // whitelist
    "1000",             // spend_limit
    "0.2",            // reward_factor
    [[100, 200, "1000000"]],  // distribution_schedule
    12345             // genesis_time
  );
}


// -------------------------------------------------------------------------------------
// setup LBP contracts
// -------------------------------------------------------------------------------------
export async function startSetupLBPContracts(): Promise<void> {
  console.log(chalk.blue("\nTestNet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  const currTime = new Date().getTime() / 1000 + 100;
  const startTime = Math.round(currTime);
  const endTime = Math.round(currTime) + 3600 * 24 * 3;
  // Setup LBP contracts
  console.log(chalk.yellow("\nStep 2a. TerraSwap Contracts"));
  await setupLBP(
    terra,
    apTeam,
    tokenCodeId,
    tokenContract,
    haloCollector,
    "0.02",
    startTime,
    endTime,
  );
}


// -------------------------------------------------------------------------------------
// migrate Angel Protocol core contracts
// -------------------------------------------------------------------------------------
export async function startMigrateContracts(): Promise<void> {
  console.log(chalk.blue("\nTestNet"));

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
    [anchorVault1, anchorVault2],
    [
      endowmentContract1,
      endowmentContract2,
      endowmentContract3,
      endowmentContract4,
    ]
  );
}

// -------------------------------------------------------------------------------------
// migrate HALO contracts
// -------------------------------------------------------------------------------------
export async function startMigrateHaloContracts(): Promise<void> {
  console.log(chalk.blue("\nTestnet"));

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
  console.log(chalk.blue("\nTestnet"));

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
    routerContract
  );
}

// -------------------------------------------------------------------------------------
// migrate LBP contracts
// -------------------------------------------------------------------------------------
export async function startMigrateAMMContracts(): Promise<void> {
  console.log(chalk.blue("\nTestnet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate Contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  await migrateAMMContracts(
    terra,
    apTeam,
    factoryContract,
    pairContract,
    routerContract,
    tokenContract,
    "0.01"
  );
}

// -------------------------------------------------------------------------------------
// start test
// -------------------------------------------------------------------------------------
export async function startTest(): Promise<void> {
  console.log(chalk.blue("\nTestNet"));

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
    anchorVault1,
    anchorVault2,
    endowmentContract1,
    endowmentContract2,
    endowmentContract3,
    endowmentContract4,
    cw4GrpApTeam,
    cw4GrpOwners,
    cw3ApTeam,
    cw3GuardianAngels,
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloStaking,
    haloVesting,
    factoryContract,
    pairContract,
    tokenContract,
    routerContract
  );
}
