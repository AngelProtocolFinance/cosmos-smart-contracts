// -------------------------------------------------------------------------------------
// LocalTerra test-suite
// -------------------------------------------------------------------------------------
import chalk from "chalk";
import { LCDClient, LocalTerra, MnemonicKey, Wallet } from "@terra-money/terra.js";

import { localterra } from "../config/localterraConstants";
import { setupAstroportPlatform } from "../processes/setup/astroport/localterra";
import { setupAstroportVaults } from "../processes/setup/vaults/astroport";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------

let terra: LocalTerra | LCDClient;
let apTeam: Wallet;
let apTeam2: Wallet;
let apTeam3: Wallet;
let apTreasury: Wallet;

let vaultLocked1: string;
let vaultLiquid1: string;
let vaultLocked2: string;
let vaultLiquid2: string;

let astroportTokenCode: number;
let astroportPairCode: number;
let astroportFactory: string;
let astroportGenerator: string;
let astroportRouter: string;
let astroTokenContract: string;
let astroTokenInitialSupply: string;
let usdcUsdtPair: string;
let usdcUsdtPairLpToken: string;
let usdcUsdtPairUsdcLiquidity: string;
let usdcUsdtPairUsdtLiquidity: string;


// -------------------------------------------------------------------------------------
// initialize variables
// -------------------------------------------------------------------------------------
async function initialize() {
    terra = new LCDClient({
        URL: localterra.networkInfo.url,
        chainID: localterra.networkInfo.chainId,
    });

    apTeam = new Wallet(terra, new MnemonicKey({ mnemonic: localterra.mnemonicKeys.test1 }));
    apTeam2 = new Wallet(terra, new MnemonicKey({ mnemonic: localterra.mnemonicKeys.test2 }));
    apTeam3 = new Wallet(terra, new MnemonicKey({ mnemonic: localterra.mnemonicKeys.test3 }));
    apTreasury = new Wallet(terra, new MnemonicKey({ mnemonic: localterra.mnemonicKeys.test4 }));

    console.log(`Using ${chalk.cyan(apTeam.key.accAddress)} as Angel Team`);
    console.log(`Using ${chalk.cyan(apTeam2.key.accAddress)} as Angel Team #2`);
    console.log(`Using ${chalk.cyan(apTeam3.key.accAddress)} as Angel Team #3`);
    console.log(`Using ${chalk.cyan(apTreasury.key.accAddress)} as Angel Protocol Treasury`);

    vaultLocked1 = localterra.contracts.vaultLocked1;
    vaultLiquid1 = localterra.contracts.vaultLiquid1;
    vaultLocked2 = localterra.contracts.vaultLocked2;
    vaultLiquid2 = localterra.contracts.vaultLiquid2;

    console.log(`Using ${chalk.cyan(vaultLocked1)} as vault Locked #1`);
    console.log(`Using ${chalk.cyan(vaultLiquid1)} as vault Liquid #1`);
    console.log(`Using ${chalk.cyan(vaultLocked2)} as vault Locked #2`);
    console.log(`Using ${chalk.cyan(vaultLiquid2)} as vault Liquid #2`);


    astroportTokenCode = localterra.astroport.astroport_token_code;
    astroportPairCode = localterra.astroport.astroport_pair_code;
    astroportFactory = localterra.astroport.astroport_factory;
    astroportGenerator = localterra.astroport.astroport_generator;
    astroportRouter = localterra.astroport.astroport_router;
    astroTokenContract = localterra.astroport.ASTRO_token_contract;
    astroTokenInitialSupply = localterra.astroport.initial_ASTRO_supply;
    usdcUsdtPair = localterra.astroport.usdc_usdt_pair_contract;
    usdcUsdtPairLpToken = localterra.astroport.usdc_usdt_pair_lp_token;
    usdcUsdtPairUsdcLiquidity = localterra.astroport.usdc_usdt_pair_usdc_liquidity;
    usdcUsdtPairUsdtLiquidity = localterra.astroport.usdc_usdt_pair_usdt_liquidity;

    console.log(`Using ${chalk.cyan(astroportTokenCode)} as Astroport (cw20) Token Code`);
    console.log(`Using ${chalk.cyan(astroportPairCode)} as Astroport Pair Code`);
    console.log(`Using ${chalk.cyan(astroportFactory)} as Astroport Factory contract`);
    console.log(`Using ${chalk.cyan(astroportGenerator)} as Astroport Generator contract`);
    console.log(`Using ${chalk.cyan(astroportRouter)} as Astroport Router contract`);
    console.log(`Using ${chalk.cyan(astroTokenContract)} as Astroport ASTRO Token`);
    console.log(
        `Using ${chalk.cyan(usdcUsdtPair)} as Astroport USDC/USDT Swap Pair`
    );
    console.log(
        `Using ${chalk.cyan(usdcUsdtPairLpToken)} as Astroport USDC/USDT Swap Pair LP Token`
    );
    console.log(`Using ${chalk.cyan(usdcUsdtPairUsdcLiquidity)} as Astroport USDC/USDT Pair USDC liquidity`);
    console.log(
        `Using ${chalk.cyan(usdcUsdtPairUsdtLiquidity)} as Astroport USDC/USDT Pair USDT liquidity`
    );
}

// -------------------------------------------------------------------------------------
// setup Astroport contracts
// -------------------------------------------------------------------------------------
export async function startSetupAstroport(): Promise<void> {
    console.log(chalk.blue("\nLocalTerra"));

    // Initialize environment information
    console.log(chalk.yellow("\nStep 1. Environment Info"));
    await initialize();

    // Setup contracts
    console.log(chalk.yellow("\nStep 2. Astroport Contracts Setup"));
    await setupAstroportPlatform(terra, { apTeam, apTeam2, apTeam3 });
}

// -------------------------------------------------------------------------------------
// setup Astroport vault contracts
// -------------------------------------------------------------------------------------
export async function startSetupAstroportVaults(): Promise<void> {
    console.log(chalk.blue(`\nLocalTerra ${localterra.networkInfo.chainId}`));

    // Initialize environment information
    console.log(chalk.yellow("\nStep 1. Environment Info"));
    await initialize();

    // // Setup contracts
    // console.log(chalk.yellow("\nStep 2. LOOP LP Vault Contracts Setup"));
    // await setupAstroportVaults(
    //     localterra.networkInfo.chainId,
    //     terra,
    //     // wallets
    //     {
    //         apTeam,
    //         apTreasury,
    //     },
    //     // config
    //     {
    //         loopswap_factory: loopswapFactory, // LoopSwap Factory contract
    //         loopswap_farming: loopswapFarming, // LoopSwap Farming contract
    //         loopswap_malo_kalo_pair: loopswapMaloKaloPairContract, // LoopSwap MALO-KALO pair contract
    //         loopswap_lp_reward_token: loopswapLoopTokenContract, // LoopSwap Pair LP Staking reward token (LOOP token)
    //         harvest_to_liquid: "0.75", // harvest to liquid percentage
    //         accepted_tokens: {
    //             native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', config.networkInfo.nativeToken],
    //             cw20: [],
    //         },
    //         swapRouter: swapRouter, // SwapRouter contract
    //         nativeToken: { native: config.networkInfo.nativeToken }, // { cw20: config.loopswap.halo_token_contract },
    //     }
    // );
}


// -------------------------------------------------------------------------------------
// Test Astroport-vault contract
// -------------------------------------------------------------------------------------
export async function startTestsAstroportVault(): Promise<void> {
    console.log(chalk.blue("\nLocalTerra"));

    // Initialize environment information
    console.log(chalk.yellow("\nStep 1. Environment Info"));
    await initialize();

    // // Test queries
    // await testExecuteStation(
    //     terra,
    //     wallet1,
    //     wallet2,
    //     wallet3,
    //     auto,
    //     registryStake,
    //     fundsRouter,
    //     timeConditions,
    //     counter,
    // );
}
