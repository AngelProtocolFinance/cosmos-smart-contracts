// -------------------------------------------------------------------------------------
// MainNet(Columbus-5) test-suite
// -------------------------------------------------------------------------------------
import { LCDClient, MnemonicKey, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { mainnet as config } from "../config/constants";
import { migrateContracts } from "../processes/migrateContracts/migration";
import { setupContracts, Member } from "../processes/setupContracts/mainnet";
import { setupHalo } from "../processes/setupHalo/testnet";
import { setupTerraSwap } from "../processes/setupTerraSwap/realnet";
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

let accAddress: string;
let tokenCodeId: number;
let pairCodeId: number;
let factoryCodeId: number;
let factoryContract: string;
let tokenContract: string;
let pairContract: string;

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
    gasPrices: { uusd: 0.5 },
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

  accAddress = config.accAddress;
  tokenCodeId = config.token_code_id;
  pairCodeId = config.pair_code_id;
  factoryCodeId = config.factory_code_id;
  factoryContract = config.factory_contract;
  tokenContract = config.token_contract;
  pairContract = config.pair_contract;

  console.log(`Use ${chalk.cyan(factoryContract)} as TerraSwap factory`);
  console.log(`Use ${chalk.cyan(tokenContract)} as HALO token`);
  console.log(`Use ${chalk.cyan(pairContract)} as HALO/UST pair`);

  haloAirdrop = config.contracts.haloAirdrop;
  haloCollector = config.contracts.haloCollector;
  haloCommunity = config.contracts.haloCommunity;
  haloDistributor = config.contracts.haloDistributor;
  haloGov = config.contracts.haloGov;
  haloStaking = config.contracts.haloStaking;
  haloVesting = config.contracts.haloVesting;

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
// setup TerraSwap contracts
// -------------------------------------------------------------------------------------
export async function startSetupTerraSwapContracts(): Promise<void> {
  console.log(chalk.blue("\nMainNet Columbus-5"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupTerraSwap(
    terra,
    apTeam,
    accAddress,
    tokenCodeId,
    pairCodeId,
    factoryCodeId,
    factoryContract
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
    tokenContract,    // halo_token contract
    factoryContract,  // terraswap_factory contract
    pairContract,     // staking_token: lp token of ANC-UST pair contract
    "0.3",            // quorum
    "0.5",            // threshold,
    2000,             // voting_period,
    1000,             // timelock_period,
    10000000000,      // proposal_deposit,
    10,               // snapshot_period,
    [],               // whitelist
    1000,             // spend_limit
    "0.2",            // reward_factor
    [100, 200, 1000000],  // distribution_schedule
    12345             // genesis_time
  );
}

// -------------------------------------------------------------------------------------
// migrate contracts
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
    endowmentContracts[0]
  );
}
