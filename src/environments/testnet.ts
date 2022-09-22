// -------------------------------------------------------------------------------------
// TestNet test-suite
// -------------------------------------------------------------------------------------
import { GasPrice } from "@cosmjs/stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import chalk from "chalk";
import { testnet as config } from "../config/constants";
import { datetimeStringToUTC, getWalletAddress } from "../utils/helpers";

import { migrateCore } from "../processes/migrate/core";
// import { migrateHalo } from "../processes/migrate/halo";

import { setupCore } from "../processes/setup/core/testnet";
import { setupLoopSwap } from "../processes/setup/loopswap/localjuno";
import { setupMockVaults } from "../processes/setup/vaults/mock-vault";
import { setupLoopVaults } from "../processes/setup/vaults/loop";
// import { setupJunoSwap } from "../processes/setup/junoswap/localjuno";
// import { setupHalo } from "../processes/setup/halo";

import { testExecute } from "../processes/tests/testnet";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let networkUrl: string;

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
let cw4GrpReviewTeam: string;
let cw3ReviewTeam: string;
let indexFund: string;
let accounts: string;
let vaultLocked1: string;
let vaultLiquid1: string;
let vaultLocked2: string;
let vaultLiquid2: string;
let endowId1: number;
let endowId2: number;
let endowId3: number;

// LoopSwap Contracts
let loopswapTokenCode: number;
let loopswapPairCode: number;
let loopswapFactory: string;
let loopswapFarming: string;

let loopswapLoopTokenContract: string;
let loopswapLoopJunoPairContract: string;
let loopswapLoopJunoPairLpToken: string;
let loopswapInitialLoopSupply: string;
let loopswapLoopLiquidity: string;
let loopswapJunoLiquidity: string;

let loopswapHaloTokenContract: string;
let loopswapHaloJunoPairContract: string;
let loopswapHaloJunoPairLpToken: string;
let loopswapInitialHaloSupply: string;
let loopswapHaloLiquidity: string;
let loopswapNativeLiquidity: string;

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

  networkUrl = config.networkInfo.url;

  registrar = config.contracts.registrar;
  accounts = config.contracts.accounts;
  cw4GrpApTeam = config.contracts.cw4GrpApTeam;
  cw3ApTeam = config.contracts.cw3ApTeam;
  cw4GrpReviewTeam = config.contracts.cw4GrpReviewTeam;
  cw3ReviewTeam = config.contracts.cw3ReviewTeam;
  indexFund = config.contracts.indexFund;
  accounts = config.contracts.accounts;
  vaultLocked1 = config.contracts.vaultLocked1;
  vaultLiquid1 = config.contracts.vaultLiquid1;
  vaultLocked2 = config.contracts.vaultLocked2;
  vaultLiquid2 = config.contracts.vaultLiquid2;

  endowId1 = config.contracts.endowId1;
  endowId2 = config.contracts.endowId2;
  endowId3 = config.contracts.endowId3;

  console.log(`Using ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Using ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Using ${chalk.cyan(accounts)} as Accounts`);
  console.log(`Using ${chalk.cyan(endowId1)} as Endowment ID #1`);
  console.log(`Using ${chalk.cyan(endowId2)} as Endowment ID #2`);
  console.log(`Using ${chalk.cyan(endowId3)} as Endowment ID #3`);
  console.log(`Using ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Using ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Using ${chalk.cyan(cw4GrpReviewTeam)} as CW4 Review Team Group`);
  console.log(`Using ${chalk.cyan(cw3ReviewTeam)} as CW3 Review Team MultiSig`);
  console.log(`Using ${chalk.cyan(vaultLocked1)} as Vault_Locked_1`);
  console.log(`Using ${chalk.cyan(vaultLiquid1)} as Vault_Liquid_1`);
  console.log(`Using ${chalk.cyan(vaultLocked2)} as Vault_Locked_2`);
  console.log(`Using ${chalk.cyan(vaultLiquid2)} as Vault_Liquid_2`);

  loopswapTokenCode = config.loopswap.loopswap_token_code;
  loopswapPairCode = config.loopswap.loopswap_pair_code;
  loopswapFactory = config.loopswap.loopswap_factory;
  loopswapFarming = config.loopswap.loopswap_farming;

  loopswapLoopTokenContract = config.loopswap.loop_token_contract;
  loopswapLoopJunoPairContract = config.loopswap.loop_juno_pair_contract;
  loopswapLoopJunoPairLpToken = config.loopswap.loop_juno_pair_lp_token;
  loopswapInitialLoopSupply = config.loopswap.initial_loop_supply;
  loopswapLoopLiquidity = config.loopswap.loop_liquidity;
  loopswapJunoLiquidity = config.loopswap.juno_liquidity;

  loopswapHaloTokenContract = config.loopswap.halo_token_contract;
  loopswapHaloJunoPairContract = config.loopswap.halo_juno_pair_contract;
  loopswapHaloJunoPairLpToken = config.loopswap.halo_juno_pair_lp_token;
  loopswapInitialHaloSupply = config.loopswap.initial_halo_supply;
  loopswapHaloLiquidity = config.loopswap.halo_liquidity;
  loopswapNativeLiquidity = config.loopswap.native_liquidity;

  console.log(`Using ${chalk.cyan(loopswapTokenCode)} as loopSwap (cw20) Token Code`);
  console.log(`Using ${chalk.cyan(loopswapPairCode)} as loopSwap Pair Code`);
  console.log(`Using ${chalk.cyan(loopswapFactory)} as loopSwap Factory contract`);
  console.log(`Using ${chalk.cyan(loopswapFarming)} as loopSwap Farming contract`);

  console.log(`Using ${chalk.cyan(loopswapHaloTokenContract)} as loopSwap LOOP Token`);
  console.log(
    `Using ${chalk.cyan(loopswapLoopJunoPairContract)} as loopSwap LOOP/JUNO Swap Pair`
  );
  console.log(
    `Using ${chalk.cyan(loopswapLoopJunoPairLpToken)} as loopSwap LOOP/JUNO Swap Pair LP Token`
  );
  console.log(`Using ${chalk.cyan(loopswapInitialLoopSupply)} as loopSwap Loop Initial Supply`);
  console.log(
    `Using ${chalk.cyan(loopswapLoopLiquidity)} as loopSwap LOOP/JUNO Pair LOOP liquidity`
  );
  console.log(
    `Using ${chalk.cyan(loopswapJunoLiquidity)} as loopSwap LOOP/JUNO Pair JUNO liquidity`
  );

  console.log(`Using ${chalk.cyan(loopswapHaloTokenContract)} as loopSwap HALO Token`);
  console.log(
    `Using ${chalk.cyan(loopswapHaloJunoPairContract)} as loopSwap HALO/JUNO Swap Pair`
  );
  console.log(
    `Using ${chalk.cyan(loopswapHaloJunoPairLpToken)} as loopSwap HALO/JUNO Swap Pair LP Token`
  );
  console.log(`Using ${chalk.cyan(loopswapInitialHaloSupply)} as loopSwap HALO Initial Supply`);
  console.log(
    `Using ${chalk.cyan(loopswapHaloLiquidity)} as loopSwap HALO/JUNO Pair HALO liquidity`
  );
  console.log(
    `Using ${chalk.cyan(loopswapNativeLiquidity)} as loopSwap HALO/JUNO Pair JUNO liquidity`
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
  juno = await SigningCosmWasmClient.connectWithSigner(config.networkInfo.url, apTeam, { gasPrice: GasPrice.fromString("0.025ujunox") });
}


// -------------------------------------------------------------------------------------
// setup contracts
// -------------------------------------------------------------------------------------
export async function startSetupCore(): Promise<void> {
  console.log(chalk.blue(`\nTestNet ${config.networkInfo.chainId}`));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupCore(
    networkUrl,
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
      threshold_absolute_percentage: "0.5", // threshold absolute percentage for "ap-cw3"
      max_voting_period_height: 1000, // max voting period height for "ap-cw3"
      fund_rotation: 10, // index fund rotation
      is_localjuno: false, // is LocalJuno
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      tax_per_block: "0.0000000259703196", // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
      funding_goal: "50000000", // funding goal
      fund_member_limit: 10,
      charity_cw3_threshold_abs_perc: "0.50", // threshold absolute percentage for "charity-cw3"
      charity_cw3_max_voting_period: 60,      // max_voting_period time(unit: seconds) for "charity-cw3"
      accepted_tokens: {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', 'ujunox'],
        cw20: [],
      },
    }
  );
}

// -------------------------------------------------------------------------------------
// setup mock vault contracts
// -------------------------------------------------------------------------------------
export async function startSetupMockVaults(): Promise<void> {
  console.log(chalk.blue(`\nTestNet ${config.networkInfo.chainId}`));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Mock Vault Contracts Setup"));
  await setupMockVaults(
    config.networkInfo.chainId,
    juno,
    // wallets
    {
      apTeam,
      apTreasury,
    },    
    // contracts
    {
      registrar,
      cw3ApTeam,
    },
    // config
    {
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      tax_per_block: "0.0000000259703196", // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
      accepted_tokens:  {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', 'ujunox'],
        cw20: [],
      },
    }
  );
}

// -------------------------------------------------------------------------------------
// setup LOOP vault contracts
// -------------------------------------------------------------------------------------
export async function startSetupLoopVaults(): Promise<void> {
  console.log(chalk.blue(`\nTestNet ${config.networkInfo.chainId}`));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. LOOP LP Vault Contracts Setup"));
  await setupLoopVaults(
    config.networkInfo.chainId,
    juno,
    // wallets
    {
      apTeam,
      apTreasury,
    },    
    // contracts
    {
      registrar,
      cw3ApTeam,
    },
    // config
    {
      loopswap_factory: loopswapFactory, // LoopSwap Factory contract
      loopswap_farming: loopswapFarming, // LoopSwap Farming contract
      loopswap_loop_juno_pair: loopswapLoopJunoPairContract, // LoopSwap LOOP-JUNO pair contract
      loopswap_lp_reward_token: loopswapLoopTokenContract, // LoopSwap Pair LP Staking reward token (LOOP token)
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      accepted_tokens:  {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', 'ujunox'],
        cw20: [],
      },
    }
  );
}

// -------------------------------------------------------------------------------------
// setup JunoSwap contracts
// -------------------------------------------------------------------------------------
// export async function startSetupJunoSwap(): Promise<void> {
//   console.log(chalk.blue("\nTestNet"));

//   // Initialize environment information
//   console.log(chalk.yellow("\nStep 1. Environment Info"));
//   await initialize();

//   // Setup JunoSwap contracts
//   console.log(chalk.yellow("\nStep 2a. JunoSwap Contracts"));
//   const apTeamAccount = await getWalletAddress(apTeam);
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
//   console.log(chalk.blue("\nTestnet"));

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
//     junoswapHaloJunoPairLpToken, // staking_token: LP token of HALO-JUNO pair contract
//     30, // quorum
//     50, // threshold,
//     2000, // voting_period,
//     1000, // timelock_period,
//     "10000000000", // proposal_deposit,
//     10, // snapshot_period,
//     360, // unbonding_period in seconds
//     [], // whitelist
//     "10000000000", // spend_limit
//     "1.0", // reward_factor
//     [[100, 200, "1000000"]], // distribution_schedule
//     12345 // genesis_time
//   );
// }

// -------------------------------------------------------------------------------------
// migrate Angel Protocol core contracts
// -------------------------------------------------------------------------------------
export async function startMigrateCore(): Promise<void> {
  console.log(chalk.blue("\nTestNet"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Migrate contracts
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
    cw3ReviewTeam,
    [vaultLocked1, vaultLiquid1, vaultLocked2, vaultLiquid2],
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
  console.log(chalk.blue("\nTestNet"));

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
    vaultLocked1,
    vaultLiquid1,
    vaultLocked2,
    vaultLiquid2,
    accounts,
    endowId1,
    endowId2,
    endowId3,
    cw4GrpApTeam,
    cw3ApTeam,
    cw4GrpReviewTeam,
    cw3ReviewTeam,
    loopswapFactory,
    loopswapFarming,
    loopswapLoopJunoPairContract,
    loopswapLoopTokenContract,
    haloAirdrop,
    haloCollector,
    haloCommunity,
    haloDistributor,
    haloGov,
    haloStaking,
    haloVesting,
  );
}
