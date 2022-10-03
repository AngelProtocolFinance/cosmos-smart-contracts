// -------------------------------------------------------------------------------------
// LocalJuno test-suite
// -------------------------------------------------------------------------------------
import { GasPrice } from "@cosmjs/stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import chalk from "chalk";
import { localjuno as config } from "../config/localjunoConstants";
import { datetimeStringToUTC, getWalletAddress, Endowment } from "../utils/helpers";

import { migrateCore } from "../processes/migrate/core";
// import { migrateHalo } from "../processes/migrate/halo";

import { setupCore } from "../processes/setup/core/testnet";
import { setupEndowments } from "../processes/setup/endowments/endowments";
import { setupLoopSwap } from "../processes/setup/loopswap/localjuno";
import { setupMockVaults } from "../processes/setup/vaults/mock-vault";
import { setupLoopVaults } from "../processes/setup/vaults/loop";
// import { setupHalo } from "../processes/setup/halo";

import { testExecute } from "../processes/tests/testnet";
import jsonData from "../processes/setup/endowments/endowments_list_testnet.json";

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
let cw4GrpReviewTeam: string;
let cw3ReviewTeam: string;
let indexFund: string;
let accounts: string;
let swapRouter: string;
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

let loopswapMaloTokenContract: string;
let loopswapMaloJunoPairContract: string;
let loopswapMaloJunoPairLpToken: string;
let loopswapInitialMaloSupply: string;
let loopswapMaloJunoPairMaloLiquidity: string;
let loopswapMaloJunoPairJunoLiquidity: string;

let loopswapKaloTokenContract: string;
let loopswapKaloJunoPairContract: string;
let loopswapKaloJunoPairLpToken: string;
let loopswapInitialKaloSupply: string;
let loopswapKaloJunoPairKaloLiquidity: string;
let loopswapKaloJunoPairJunoLiquidity: string;

let loopswapMaloKaloPairContract: string;
let loopswapMaloKaloPairLpToken: string;
let loopswapMaloKaloPairMaloLiquidity: string;
let loopswapMaloKaloPairKaloLiquidity: string;

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
  apTeam = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.apTeam, { prefix: config.networkInfo.walletPrefix });
  apTeam2 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.apTeam2, { prefix: config.networkInfo.walletPrefix });
  apTeam3 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.apTeam3, { prefix: config.networkInfo.walletPrefix });
  apTreasury = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.apTreasury, { prefix: config.networkInfo.walletPrefix });
  charity1 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.charity1, { prefix: config.networkInfo.walletPrefix });
  charity2 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.charity2, { prefix: config.networkInfo.walletPrefix });
  charity3 = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.charity3, { prefix: config.networkInfo.walletPrefix });
  pleb = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.pleb, { prefix: config.networkInfo.walletPrefix });
  tca = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonicKeys.tca, { prefix: config.networkInfo.walletPrefix });

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
  cw4GrpReviewTeam = config.contracts.cw4GrpReviewTeam;
  cw3ReviewTeam = config.contracts.cw3ReviewTeam;
  indexFund = config.contracts.indexFund;
  accounts = config.contracts.accounts;
  swapRouter = config.contracts.swapRouter;
  endowId1 = config.contracts.endowId1;
  endowId2 = config.contracts.endowId2;
  endowId3 = config.contracts.endowId3;
  vaultLocked1 = config.contracts.vaultLocked1;
  vaultLiquid1 = config.contracts.vaultLiquid1;
  vaultLocked2 = config.contracts.vaultLocked2;
  vaultLiquid2 = config.contracts.vaultLiquid2;

  console.log(`Using ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Using ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Using ${chalk.cyan(accounts)} as Accounts`);
  console.log(`Using ${chalk.cyan(swapRouter)} as SwapRouter`);
  console.log(`Using ${chalk.cyan(endowId1)} as Endowment ID #1`);
  console.log(`Using ${chalk.cyan(endowId2)} as Endowment ID #2`);
  console.log(`Using ${chalk.cyan(endowId3)} as Endowment ID #3`);
  console.log(`Using ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Using ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Using ${chalk.cyan(cw4GrpReviewTeam)} as CW4 Review Team Group`);
  console.log(`Using ${chalk.cyan(cw3ReviewTeam)} as CW3 Review Team MultiSig`);

  console.log(`Using ${chalk.cyan(vaultLocked1)} as vault Locked #1`);
  console.log(`Using ${chalk.cyan(vaultLiquid1)} as vault Liquid #1`);
  console.log(`Using ${chalk.cyan(vaultLocked2)} as vault Locked #2`);
  console.log(`Using ${chalk.cyan(vaultLiquid2)} as vault Liquid #2`);


  loopswapTokenCode = config.loopswap.loopswap_token_code;
  loopswapPairCode = config.loopswap.loopswap_pair_code;
  loopswapFactory = config.loopswap.loopswap_factory;
  loopswapFarming = config.loopswap.loopswap_farming;

  loopswapLoopTokenContract = config.loopswap.loop_token_contract;
  loopswapLoopJunoPairContract = config.loopswap.loop_juno_pair_contract;
  loopswapLoopJunoPairLpToken = config.loopswap.loop_juno_pair_lp_token;
  loopswapInitialLoopSupply = config.loopswap.initial_loop_supply;
  loopswapLoopLiquidity = config.loopswap.lj_pair_loop_liquidity;
  loopswapJunoLiquidity = config.loopswap.lj_pair_juno_liquidity;

  loopswapMaloTokenContract = config.loopswap.malo_token_contract;
  loopswapMaloJunoPairContract = config.loopswap.malo_juno_pair_contract;
  loopswapMaloJunoPairLpToken = config.loopswap.malo_juno_pair_lp_token;
  loopswapInitialMaloSupply = config.loopswap.initial_malo_supply;
  loopswapMaloJunoPairMaloLiquidity = config.loopswap.mj_pair_malo_liquidity;
  loopswapMaloJunoPairJunoLiquidity = config.loopswap.mj_pair_juno_liquidity;

  loopswapKaloTokenContract = config.loopswap.kalo_token_contract;
  loopswapKaloJunoPairContract = config.loopswap.kalo_juno_pair_contract;
  loopswapKaloJunoPairLpToken = config.loopswap.kalo_juno_pair_lp_token;
  loopswapInitialKaloSupply = config.loopswap.initial_kalo_supply;
  loopswapKaloJunoPairKaloLiquidity = config.loopswap.kj_pair_kalo_liquidity;
  loopswapKaloJunoPairJunoLiquidity = config.loopswap.kj_pair_juno_liquidity;

  loopswapMaloKaloPairContract = config.loopswap.malo_kalo_pair_contract;
  loopswapMaloKaloPairLpToken = config.loopswap.malo_kalo_pair_lp_token;
  loopswapMaloKaloPairMaloLiquidity = config.loopswap.mk_pair_malo_liquidity;
  loopswapMaloKaloPairKaloLiquidity = config.loopswap.mk_pair_kalo_liquidity;

  console.log(`Using ${chalk.cyan(loopswapTokenCode)} as loopSwap (cw20) Token Code`);
  console.log(`Using ${chalk.cyan(loopswapPairCode)} as loopSwap Pair Code`);
  console.log(`Using ${chalk.cyan(loopswapFactory)} as loopSwap Factory contract`);
  console.log(`Using ${chalk.cyan(loopswapFarming)} as loopSwap Farming contract`);

  console.log(`Using ${chalk.cyan(loopswapLoopTokenContract)} as loopSwap LOOP Token`);
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

  console.log(`Using ${chalk.cyan(loopswapMaloTokenContract)} as loopSwap MALO Token`);
  console.log(
    `Using ${chalk.cyan(loopswapMaloJunoPairContract)} as loopSwap MALO/JUNO Swap Pair`
  );
  console.log(
    `Using ${chalk.cyan(loopswapMaloJunoPairLpToken)} as loopSwap MALO/JUNO Swap Pair LP Token`
  );
  console.log(`Using ${chalk.cyan(loopswapInitialMaloSupply)} as loopSwap MALO Initial Supply`);
  console.log(
    `Using ${chalk.cyan(loopswapMaloJunoPairMaloLiquidity)} as loopSwap MALO/JUNO Pair MALO liquidity`
  );
  console.log(
    `Using ${chalk.cyan(loopswapMaloJunoPairJunoLiquidity)} as loopSwap MALO/JUNO Pair JUNO liquidity`
  );

  console.log(`Using ${chalk.cyan(loopswapKaloTokenContract)} as loopSwap KALO Token`);
  console.log(
    `Using ${chalk.cyan(loopswapKaloJunoPairContract)} as loopSwap KALO/JUNO Swap Pair`
  );
  console.log(
    `Using ${chalk.cyan(loopswapKaloJunoPairLpToken)} as loopSwap KALO/JUNO Swap Pair LP Token`
  );
  console.log(`Using ${chalk.cyan(loopswapInitialKaloSupply)} as loopSwap KALO Initial Supply`);
  console.log(
    `Using ${chalk.cyan(loopswapKaloJunoPairKaloLiquidity)} as loopSwap KALO/JUNO Pair KALO liquidity`
  );
  console.log(
    `Using ${chalk.cyan(loopswapKaloJunoPairJunoLiquidity)} as loopSwap KALO/JUNO Pair JUNO liquidity`
  );

  console.log(
    `Using ${chalk.cyan(loopswapMaloKaloPairContract)} as loopSwap MALO/KALO Swap Pair`
  );
  console.log(
    `Using ${chalk.cyan(loopswapMaloKaloPairLpToken)} as loopSwap MALO/KALO Swap Pair LP Token`
  );
  console.log(
    `Using ${chalk.cyan(loopswapMaloKaloPairMaloLiquidity)} as loopSwap KALO/JUNO Pair MALO liquidity`
  );
  console.log(
    `Using ${chalk.cyan(loopswapMaloKaloPairKaloLiquidity)} as loopSwap KALO/JUNO Pair KALO liquidity`
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
  juno = await SigningCosmWasmClient.connectWithSigner(config.networkInfo.url, apTeam, { gasPrice: GasPrice.fromString(config.networkInfo.gasPrice) });
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
    config.networkInfo,
    juno,
    // wallets
    {
      apTeam,
      apTeam2,
      apTeam3,
      apTreasury,
    },
    // config
    {
      tax_rate: "0.2", // tax rate
      threshold_absolute_percentage: "0.5", // threshold absolute percentage for "apteam-cw3" & "reviewteam-cw3"
      max_voting_period_height: 100000, // max voting period height for "apteam-cw3" & "reviewteam-cw3"
      fund_rotation: undefined, // index fund rotation
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      funding_goal: "500000000", // funding goal
      fund_member_limit: 10,
      charity_cw3_threshold_abs_perc: "0.5", // threshold absolute percentage for "charity-cw3"
      charity_cw3_max_voting_period: 604800, // 1 week max voting period time(unit: seconds) for "charity-cw3"
      accepted_tokens: {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', config.networkInfo.nativeToken],
        cw20: [],
      },
    }
  );
}


// -------------------------------------------------------------------------------------
// setup new charity endowments in the Accounts contract
// -------------------------------------------------------------------------------------
export async function startSetupEndowments(): Promise<void> {
  console.log(chalk.blue(`\nLocalJuno ${config.networkInfo.chainId}`));

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
    604800, // 1 week max voting period time(unit: seconds) for "charity-cw3"
  );
}

// -------------------------------------------------------------------------------------
// setup mock vault contracts
// -------------------------------------------------------------------------------------
export async function startSetupMockVaults(): Promise<void> {
  console.log(chalk.blue(`\nLocalJuno ${config.networkInfo.chainId}`));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup contracts
  console.log(chalk.yellow("\nStep 2. Mock Vault Contracts Setup"));
  await setupMockVaults(
    config.networkInfo.chainId,
    config.networkInfo.nativeToken,
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
      accepted_tokens: {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', config.networkInfo.nativeToken],
        cw20: [],
      },
    }
  );
}

// -------------------------------------------------------------------------------------
// setup LOOP vault contracts
// -------------------------------------------------------------------------------------
export async function startSetupLoopVaults(): Promise<void> {
  console.log(chalk.blue(`\nLocalJuno ${config.networkInfo.chainId}`));

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
      loopswap_malo_kalo_pair: loopswapMaloKaloPairContract, // LoopSwap MALO-KALO pair contract
      loopswap_lp_reward_token: loopswapLoopTokenContract, // LoopSwap Pair LP Staking reward token (LOOP token)
      harvest_to_liquid: "0.75", // harvest to liquid percentage
      accepted_tokens: {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', config.networkInfo.nativeToken],
        cw20: [],
      },
      swapRouter: swapRouter, // SwapRouter contract
      nativeToken: { native: config.networkInfo.nativeToken }, // { cw20: config.loopswap.halo_token_contract },
    }
  );
}

// -------------------------------------------------------------------------------------
// setup LoopSwap contracts
// -------------------------------------------------------------------------------------
export async function startSetupLoopSwap(): Promise<void> {
  console.log(chalk.blue("\nLocalJuno"));

  // Initialize environment information
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  await initialize();

  // Setup LoopSwap contracts
  console.log(chalk.yellow("\nStep 2a. LoopSwap Contracts"));
  const apTeamAccount = await getWalletAddress(apTeam);
  const apTeam2Account = await getWalletAddress(apTeam2);
  const apTeam3Account = await getWalletAddress(apTeam3);
  await setupLoopSwap(
    juno,
    apTeamAccount,
    apTeam2Account,
    apTeam3Account,
    loopswapInitialLoopSupply,
    loopswapLoopLiquidity,
    loopswapJunoLiquidity,

    loopswapInitialMaloSupply,
    loopswapMaloJunoPairMaloLiquidity,
    loopswapMaloJunoPairJunoLiquidity,

    loopswapInitialKaloSupply,
    loopswapKaloJunoPairKaloLiquidity,
    loopswapKaloJunoPairJunoLiquidity,

    loopswapMaloKaloPairMaloLiquidity,
    loopswapMaloKaloPairKaloLiquidity,
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
