// -------------------------------------------------------------------------------------
// LocalTerra test-suite
// -------------------------------------------------------------------------------------
import { LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { localterra as config } from "../config/constants";
import { migrateContracts } from "../processes/migrateContracts/migration";
import { setupContracts } from "../processes/setupContracts/testnet";
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
  anchorVault1 = config.contracts.anchorVault1;
  anchorVault2 = config.contracts.anchorVault2;
  endowmentContract1 = config.contracts.endowmentContract1;
  endowmentContract2 = config.contracts.endowmentContract2;
  endowmentContract3 = config.contracts.endowmentContract3;
  endowmentContract4 = config.contracts.endowmentContract4;

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
}

// -------------------------------------------------------------------------------------
// start setup contracts
// -------------------------------------------------------------------------------------
export async function startSetupContracts(): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupContracts(
    terra,
    undefined,
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
      fund_rotation: undefined,     // index fund rotation
      turnover_to_multisig: false,   // turn over to AP Team multisig
      is_localterra: true,   // is LocalTerra
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      tax_per_block: "0.0000000259703196", // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
      funding_goal: "50000000", // funding goal
    },
  );
}

// -------------------------------------------------------------------------------------
// start setup contracts
// -------------------------------------------------------------------------------------
export async function startMigrateContracts(): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initialize();

  // Migrate Contracts
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

export async function startTest(): Promise<void> {
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
    anchorVault1,
    anchorVault2,
    endowmentContract1,
    endowmentContract2,
    endowmentContract3,
    endowmentContract4,
    cw4GrpApTeam,
    cw4GrpOwners,
    cw3ApTeam,
    cw3GuardianAngels
  );
}
