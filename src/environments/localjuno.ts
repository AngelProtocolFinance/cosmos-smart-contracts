// -------------------------------------------------------------------------------------
// LocalJuno test-suite
// -------------------------------------------------------------------------------------
import { GasPrice } from "@cosmjs/stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import chalk from "chalk";
import { localjuno as config } from "../config/localjunoConstants";
import { datetimeStringToUTC, getWalletAddress } from "../utils/helpers";

import { migrateCore } from "../processes/migrate/core";
// import { migrateHalo } from "../processes/migrate/halo";

import { setupCore } from "../processes/setup/core/testnet";
import { setupJunoSwap } from "../processes/setup/junoswap/localjuno";
// import { setupHalo } from "../processes/setup/halo";

import { testExecute } from "../processes/tests/testnet";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let juno: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTeam2: DirectSecp256k1HdWallet;
let apTeam3: DirectSecp256k1HdWallet;
let apTreasury: DirectSecp256k1HdWallet;
let charity1: DirectSecp256k1HdWallet;
let charity2: DirectSecp256k1HdWallet;
let charity3: DirectSecp256k1HdWallet;
let pleb: DirectSecp256k1HdWallet;
let tca: DirectSecp256k1HdWallet;

// wallet addresses
let apTeamAccount: string;
let apTeam2Account: string;
let apTeam3Account: string;
let apTreasuryAccount: string;
let charity1Account: string;
let charity2Account: string;
let charity3Account: string;
let plebAccount: string;
let tcaAccount: string;

// Core contracts
let registrar: string;
let cw4GrpApTeam: string;
let cw3ApTeam: string;
let indexFund: string;
let accounts: string;
let endowId1: number;
let endowId2: number;
let endowId3: number;
let endowId4: number;
let vault1: string;
let vault2: string;

// JunoSwap Contracts
let junoswapTokenCode: number;
let junoswapHaloTokenContract: string;
let junoswapHaloJunoPairContract: string;
let junoswapHaloJunoPairLpToken: string;
let junoswapHaloJunoPairLpStakingContract: string;
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
  apTeam2 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.apTeam2, { prefix: "juno" });
  apTeam3 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.apTeam3, { prefix: "juno" });
  apTreasury = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.apTreasury, { prefix: "juno" });
  charity1 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.charity1, { prefix: "juno" });
  charity2 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.charity2, { prefix: "juno" });
  charity3 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.charity3, { prefix: "juno" });
  pleb = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.pleb, { prefix: "juno" });
  tca = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.tca, { prefix: "juno" });

  apTeamAccount = await getWalletAddress(apTeam);
  apTeam2Account = await getWalletAddress(apTeam2);
  apTeam3Account = await getWalletAddress(apTeam3);
  apTreasuryAccount = await getWalletAddress(apTreasury);
  charity1Account = await getWalletAddress(charity1);
  charity2Account = await getWalletAddress(charity2);
  charity3Account = await getWalletAddress(charity3);
  plebAccount = await getWalletAddress(pleb);
  tcaAccount = await getWalletAddress(tca);

  console.log(`Using ${chalk.cyan(apTeamAccount)} as Angel Team`);
  console.log(`Using ${chalk.cyan(apTeam2Account)} as Angel Team #2`);
  console.log(`Using ${chalk.cyan(apTeam3Account)} as Angel Team #3`);
  console.log(`Using ${chalk.cyan(apTreasuryAccount)} as Angel Protocol Treasury`);
  console.log(`Using ${chalk.cyan(charity1Account)} as Charity #1`);
  console.log(`Using ${chalk.cyan(charity2Account)} as Charity #2`);
  console.log(`Using ${chalk.cyan(charity3Account)} as Charity #3`);
  console.log(`Using ${chalk.cyan(plebAccount)} as Pleb`);
  console.log(`Using ${chalk.cyan(tcaAccount)} as TCA member`);

  registrar = config.contracts.registrar;
  cw4GrpApTeam = config.contracts.cw4GrpApTeam;
  cw3ApTeam = config.contracts.cw3ApTeam;
  indexFund = config.contracts.indexFund;
  accounts = config.contracts.accountsContract;
  vault1 = config.contracts.vault1;
  vault2 = config.contracts.vault2;

  console.log(`Using ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Using ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Using ${chalk.cyan(accounts)} as Accounts`);
  console.log(`Using ${chalk.cyan(endowId1)} as Endowment ID #1`);
  console.log(`Using ${chalk.cyan(endowId2)} as Endowment ID #2`);
  console.log(`Using ${chalk.cyan(endowId3)} as Endowment ID #3`);
  console.log(`Using ${chalk.cyan(endowId4)} as Endowment ID #4`);
  console.log(`Using ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Using ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Using ${chalk.cyan(vault1)} as Vault1`);
  console.log(`Using ${chalk.cyan(vault2)} as Vault2`);

  junoswapTokenCode = config.junoswap.junoswap_token_code;
  junoswapHaloTokenContract = config.junoswap.halo_token_contract;
  junoswapHaloJunoPairContract = config.junoswap.halo_juno_pool_contract;
  junoswapHaloJunoPairLpToken = config.junoswap.halo_juno_pool_lp_token;
  junoswapHaloJunoPairLpStakingContract = config.junoswap.halo_juno_pool_lp_staking_addr;
  junoswapInitialHaloSupply = config.junoswap.initial_halo_supply;
  junoswapHaloLiquidity = config.junoswap.halo_liquidity;
  junoswapNativeLiquidity = config.junoswap.native_liquidity;

  console.log(`Using ${chalk.cyan(junoswapHaloTokenContract)} as JunoSwap HALO Token`);
  console.log(
    `Using ${chalk.cyan(junoswapHaloJunoPairContract)} as JunoSwap HALO/JUNO Swap Pool(Pair)`
  );
  console.log(
    `Using ${chalk.cyan(junoswapHaloJunoPairLpToken)} as JunoSwap HALO/JUNO Swap Pool LP Token`
  );
  console.log(
    `Using ${chalk.cyan(junoswapHaloJunoPairLpStakingContract)} as JunoSwap HALO/JUNO Swap Pool LP Token Staking contract`
  );

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
  juno = await SigningCosmWasmClient.connectWithSigner(config.networkInfo.url, apTeam, { gasPrice: GasPrice.fromString("0.1ujuno") });
}

// -------------------------------------------------------------------------------------
// setup core contracts
// -------------------------------------------------------------------------------------
export async function startSetupCore(): Promise<void> {
  console.log(chalk.blue("\nLocalJuno"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupCore(
    juno,     
    // wallets
    {
      apTeam,
      apTeam2,
      apTeam3,
      apTreasury,
      charity1,
      charity2,
      charity3,
      tca,
    },
    // config
    {
      tax_rate: "0.2", // tax rate
      threshold_absolute_percentage: "0.50", // threshold absolute percentage for "ap-cw3"
      max_voting_period_height: 1000, // max voting period height for "ap-cw3"
      fund_rotation: undefined, // index fund rotation
      is_localjuno: true, // is LocalJuno
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      tax_per_block: "0.0000000259703196", // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
      funding_goal: "500000000", // funding goal
      fund_member_limit: 10,
      charity_cw3_threshold_abs_perc: "0.5", // threshold absolute percentage for "charity-cw3"
      charity_cw3_max_voting_period: 60,      // max_voting_period time(unit: seconds) for "charity-cw3"
      accepted_tokens:  {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', 'ujuno'],
        cw20: [],
      },
      junoswap_pool_addr: junoswapHaloJunoPairContract, // Junoswap pool (HALO-JUNO) contract
      junoswap_pool_staking: junoswapHaloJunoPairLpStakingContract, // Junoswap pool (HALO-JUNO) LP token staking contract
    }
  );
}

// -------------------------------------------------------------------------------------
// setup JunoSwap contracts
// -------------------------------------------------------------------------------------
export async function startSetupJunoSwap(): Promise<void> {
  console.log(chalk.blue("\nLocalJuno"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup JunoSwap contracts
  console.log(chalk.yellow("\nStep 2a. JunoSwap Contracts"));
  const apTeamAccount = await getWalletAddress(apTeam);
  const apTeam2Account = await getWalletAddress(apTeam2);
  const apTeam3Account = await getWalletAddress(apTeam3);
  await setupJunoSwap(
    juno,
    apTeamAccount,
    apTeam2Account,
    apTeam3Account,
    junoswapInitialHaloSupply,
    junoswapHaloLiquidity,
    junoswapNativeLiquidity
  );
}

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
//   const apTeamAccount = await getWalletAddress(apTeam);
//   await setupHalo(
//     juno,
//     apTeamAccount,
//     registrar,
//     junoswapHaloTokenContract, // halo junoswap token contract
//     junoswapFactory,
//     junoswapHaloJunoPairLpToken, // staking_token: HALO-JUNO pair LP Token contract
//     30, // quorum
//     50, // threshold,
//     2000, // voting_period,
//     1000, // timelock_period,
//     "10000000000", // proposal_deposit,
//     10, // snapshot_period,
//     120, // unbonding_period in seconds
//     [], // whitelist
//     "10000000000", // spend_limit
//     "0.2", // reward_factor
//     [[100, 200, "1000000"]], // distribution_schedule
//     12345 // genesis_time
//   );
// }

// -------------------------------------------------------------------------------------
// migrate Angel Protocol core contracts
// -------------------------------------------------------------------------------------
export async function startMigrateCore(): Promise<void> {
  console.log(chalk.blue("\nLocalJuno"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Migrate Contracts
  console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  const apTeamAccount = await getWalletAddress(apTeam);
  await migrateCore(
    juno,
    apTeamAccount,
    registrar,
    indexFund,
    accounts,
    cw4GrpApTeam,
    cw3ApTeam,
    [],
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
//   const apTeamAccount = await getWalletAddress(apTeam);
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
  console.log(chalk.blue("\nLocalJuno"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Test queries
  await testExecute(
    config,
    apTeam,
    apTeam2,
    apTeam3,
    charity1,
    charity2,
    charity3,
    pleb,
    tca,
    apTeamAccount,
    apTeam2Account,
    apTeam3Account,
    apTreasuryAccount,
    charity1Account,
    charity2Account,
    charity3Account,
    plebAccount,
    tcaAccount,
    registrar,
    indexFund,
    vault1,
    vault2,
    accounts,
    endowId1,
    endowId2,
    endowId3,
    endowId4,
    cw4GrpApTeam,
    cw3ApTeam,
    junoswapHaloTokenContract,
    junoswapHaloJunoPairContract,
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloStaking,
    haloVesting,
  );
}
