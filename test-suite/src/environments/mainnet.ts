// -------------------------------------------------------------------------------------
// MainNet test-suite
// -------------------------------------------------------------------------------------
import { GasPrice } from "@cosmjs/stargate";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import * as readline from "node:readline/promises";

import chalk from "chalk";
import { mainnet as config } from "../config/constants";
import {
  datetimeStringToUTC,
  getWalletAddress,
  Member,
  Endowment,
} from "../utils/juno/helpers";

import { migrateCore } from "../processes/migrate/core";
// import { migrateHalo } from "../processes/migrate/halo";

import { setupCore } from "../processes/setup/core/mainnet";
import { setupEndowments } from "../processes/setup/endowments/endowments";
import { setupGiftcards } from "../processes/setup/accessories/giftcards";
// import { setupJunoSwap } from "../processes/setup/junoswap/realnet";
// import { setupHalo } from "../processes/setup/halo";

import { testExecute } from "../processes/tests/mainnet";
import jsonData from "../processes/setup/endowments/endowments_list_mainnet.json";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let juno: SigningCosmWasmClient;

// ap team wallet - pulled from user input (NEVER set in config constants)
let apTeam: DirectSecp256k1HdWallet;
let apTeamMnemonic: string;
let apTeamAccount: string;

// wallet addresses
let apTreasuryAccount: string;
let keeperAccount: string;

let registrar: string;
let cw4GrpApTeam: string;
let cw3ApTeam: string;
let cw4GrpReviewTeam: string;
let cw3ReviewTeam: string;
let indexFund: string;
let accounts: string;
let donationMatching: string;
let swapRouter: string; // FIXME: Add the scripts to initialize this variable.
let settingsController: string; // FIXME: Add the scripts to initialize this variable.
let giftcards: string;
let apTreasury: string;

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
  // always get the mainnet AP Team seed phrase from user input
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  const apTeamMnemonic = await rl.question('Enter AP Team wallet mnemonic(Juno MAINNET): ');
  rl.close();

  // derive the wallet signing keys from the seed
  apTeam = await DirectSecp256k1HdWallet.fromMnemonic(
    apTeamMnemonic,
    {
      prefix: config.networkInfo.walletPrefix,
    }
  );
  apTeamAccount = await getWalletAddress(apTeam);
  
  // AP Treasury & Keeper are held as wallet addresses (not seed phrase)
  apTreasuryAccount = config.wallets.apTreasury;
  keeperAccount = config.wallets.keeper;

  console.log(`Using ${chalk.cyan(apTeamAccount)} as Angel Team`);
  console.log(
    `Using ${chalk.cyan(apTreasuryAccount)} as Angel Protocol Treasury`
  );
  console.log(`Using ${chalk.cyan(keeperAccount)} as AWS Keeper`);

  registrar = config.contracts.registrar;
  accounts = config.contracts.accounts;
  donationMatching = config.contracts.donationMatching;
  cw4GrpApTeam = config.contracts.cw4GrpApTeam;
  cw3ApTeam = config.contracts.cw3ApTeam;
  cw4GrpReviewTeam = config.contracts.cw4GrpReviewTeam;
  cw3ReviewTeam = config.contracts.cw3ReviewTeam;
  indexFund = config.contracts.indexFund;
  // members = [...config.members];
  // tcaMembers = [];
  giftcards = config.contracts.giftcards;

  console.log(`Using ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Using ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Using ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Using ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Using ${chalk.cyan(cw4GrpReviewTeam)} as CW4 Review Team Group`);
  console.log(`Using ${chalk.cyan(cw3ReviewTeam)} as CW3 Review Team MultiSig`);
  console.log(`Using ${chalk.cyan(giftcards)} as Gift Cards`);

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

  // setup client connection to the JUNO network
  juno = await SigningCosmWasmClient.connectWithSigner(
    config.networkInfo.url,
    apTeam,
    {
      gasPrice: GasPrice.fromString(config.networkInfo.gasPrice),
    }
  );
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
  await setupCore(juno, apTeam, apTreasuryAccount, {
    tax_rate: "0.2", // tax rate
    threshold_absolute_percentage: "0.50", // threshold absolute percentage for "ap-cw3"
    max_voting_period_height: 100000, // max voting period height for "ap-cw3"
    fund_rotation: 10, // index fund rotation
    fund_member_limit: 10, // fund member limit
    funding_goal: "50000000", // funding goal
    accepted_tokens: {
      native: [
        "ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034",
        "ujuno",
      ],
      cw20: [],
    },
  });
}

// -------------------------------------------------------------------------------------
// setup new charity endowments in the Accounts contract
// -------------------------------------------------------------------------------------
export async function startSetupEndowments(): Promise<void> {
  console.log(chalk.blue(`\nMainNet ${config.networkInfo.chainId}`));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // parse endowment JSON data
  const endowmentData: Endowment[] = [];
  jsonData.data.forEach((el) => {
    const item: Endowment = el;
    endowmentData.push(item);
  });

  // Setup endowments
  console.log(chalk.yellow("\nStep 2. Endowments Setup"));
  await setupEndowments(
    config.networkInfo,
    endowmentData,
    apTeam,
    cw3ReviewTeam,
    accounts,
    "0.5", // threshold absolute percentage for "charity-cw3"
    604800 // 1 week max voting period time(unit: seconds) for "charity-cw3"
  );
}

// -------------------------------------------------------------------------------------
// setup accessories contracts
// -------------------------------------------------------------------------------------
export async function startSetupGiftcards(): Promise<void> {
  console.log(chalk.blue(`\nTestNet ${config.networkInfo.chainId}`));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Gift Cards Contract Setup"));
  await setupGiftcards(
    config.networkInfo.chainId,
    juno,
    apTeam,
    keeperAccount,
    registrar
  );
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
//     apTeamAccount,
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
//     apTeamAccount,
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
    accounts,
    cw4GrpApTeam,
    cw3ApTeam,
    cw4GrpReviewTeam,
    cw3ReviewTeam,
    swapRouter,
    settingsController,
    donationMatching,
    giftcards,
    [],
    config.networkInfo.axelarGateway,
    config.networkInfo.axelarIbcChannel
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
//     apTeamAccount,
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
  console.log(chalk.blue(`\nMainNet ${config.networkInfo.chainId}`));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Test query
  await testExecute(
    juno,
    apTeam,
    apTeamAccount,
    registrar,
    indexFund,
    accounts,
    settingsController,
    donationMatching,
    cw4GrpApTeam,
    cw3ApTeam,
    cw4GrpReviewTeam,
    cw3ReviewTeam,
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloStaking,
    haloVesting,
    giftcards
  );
}
