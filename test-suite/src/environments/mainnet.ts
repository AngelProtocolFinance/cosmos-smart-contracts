// -------------------------------------------------------------------------------------
// MainNet(Columbus-5) test-suite
// -------------------------------------------------------------------------------------
import { LCDClient, MnemonicKey, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { mainnet as config } from "../config/constants";
import { migrateContracts } from "../processes/migrationContracts/migration";
import { setupContracts, Member } from "../processes/setupContracts/mainnet";
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

// -------------------------------------------------------------------------------------
// initialize variables
// -------------------------------------------------------------------------------------
function initialize() {
  terra = new LCDClient({
    URL: config.networkInfo.url,
    chainID: config.networkInfo.chainId,
    gasPrices: { uusd: 0.4 },
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
}

// -------------------------------------------------------------------------------------
// start test
// -------------------------------------------------------------------------------------
export async function start(): Promise<void> {
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

  // Test query
  await testExecute(terra,
    registrar,
    indexFund,
    anchorVault,
    endowmentContracts[0]
  );
}
