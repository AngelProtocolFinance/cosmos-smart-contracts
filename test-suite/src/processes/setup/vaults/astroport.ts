/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LocalTerra, Wallet, LCDClient, MsgExecuteContract } from "@terra-money/terra.js";

import { sendTransaction, storeCode, instantiateContract } from "../../../utils/terra/helpers";
import { wasm_path } from "../../../config/wasmPaths";
import { localterra } from "../../../config/localterraConstants";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let chainId: string;
let terra: LocalTerra | LCDClient;
let apTeam: Wallet;
let apTeam2: Wallet;
let apTeam3: Wallet;
let apTreasury: Wallet;

// contracts
let vault1_locked: string;
let vault1_liquid: string;

// -------------------------------------------------------------------------------------
// setup all contracts for LocalTerra and TestNet
// -------------------------------------------------------------------------------------

export async function setupAstroportVaults(
    _chainId: string,
    _terra: LocalTerra | LCDClient,
    wallets: {
        apTeam: Wallet;
        apTeam2: Wallet;
        apTeam3: Wallet;
        apTreasury: Wallet;
    },
    config: {
        astroport_factory: string;
        astroport_generator: string;
        astroport_usdc_usdt_lp_pair: string;
        astroport_lp_reward_token: string;
        astroport_router: string;
        nativeToken: any;
        ap_tax_rate: string;
        interest_distribution: string;
    }
): Promise<void> {
    chainId = _chainId;
    terra = _terra;
    apTeam = wallets.apTeam;
    apTeam2 = wallets.apTeam2;
    apTeam3 = wallets.apTeam3;
    apTreasury = wallets.apTreasury;

    await createAstroportVaults(
        config.astroport_factory,
        config.astroport_generator,
        config.astroport_usdc_usdt_lp_pair,
        config.astroport_lp_reward_token,
        apTeam.key.accAddress,
        apTeam2.key.accAddress,
        config.astroport_router,
        config.nativeToken,
        config.ap_tax_rate,
        config.interest_distribution
    );
}

async function createAstroportVaults(
    astroportFactory: string,
    astroportGenerator: string,
    usdc_usdt_pair_contract: string,
    astroport_lp_reward_token: string,
    keeper: string,
    tax_collector: string,
    astroport_router: string,
    native_token: string,
    ap_tax_rate: string,
    interest_distribution: string
): Promise<void> {
    process.stdout.write("Uploading Vault Wasm");
    const vaultCodeId = await storeCode(terra, apTeam, `${wasm_path.core}/astroport_vault.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

    // Astroport Vault - #1 (Locked)
    process.stdout.write("Instantiating Vault #1 (Locked) contract");
    const vaultResult1 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
        ibc_host: apTeam.key.accAddress,
        ibc_controller: apTeam.key.accAddress,
        ap_tax_rate: ap_tax_rate,
        interest_distribution: interest_distribution,

        acct_type: `locked`, // Locked: 0, Liquid: 1
        sibling_vault: undefined,
        registrar_contract: apTeam.key.accAddress, // Use the `apTeam` address for mock
        keeper: keeper,
        tax_collector: tax_collector,
        swap_router: astroport_router,

        lp_factory_contract: astroportFactory,
        lp_staking_contract: astroportGenerator,
        pair_contract: usdc_usdt_pair_contract,
        lp_reward_token: astroport_lp_reward_token,
        native_token: native_token,

        reward_to_native_route: [
            {
                astro_swap: {
                    offer_asset_info: {
                        token: { contract_addr: astroport_lp_reward_token },
                    },
                    ask_asset_info: {
                        native_token: { denom: localterra.denoms.usdc },
                    },
                },
            },
        ],
        native_to_lp0_route: [],
        native_to_lp1_route: [
            {
                astro_swap: {
                    offer_asset_info: {
                        native_token: { denom: localterra.denoms.usdc },
                    },
                    ask_asset_info: {
                        native_token: { denom: localterra.denoms.usdt },
                    },
                },
            },
        ],

        name: "Vault Token for USDC-USDT pair",
        symbol: "VTUSDCUSDT",
        decimals: 6,
    });
    vault1_locked = vaultResult1.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("Locked contractAddress")}=${vault1_locked}`);

    // Astroport Vault - #1 (Liquid)
    process.stdout.write("Instantiating Vault #1 (Liquid) contract");
    const vaultResult2 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
        ibc_host: apTeam.key.accAddress,
        ibc_controller: apTeam.key.accAddress,
        ap_tax_rate: ap_tax_rate,
        interest_distribution: interest_distribution,

        acct_type: `liquid`, // Locked: 0, Liquid: 1
        sibling_vault: vault1_locked,
        registrar_contract: apTeam.key.accAddress, // Use the `apTeam` address for mock
        keeper: keeper,
        tax_collector: tax_collector,
        swap_router: astroport_router,

        lp_factory_contract: astroportFactory,
        lp_staking_contract: astroportGenerator,
        pair_contract: usdc_usdt_pair_contract,
        lp_reward_token: astroport_lp_reward_token,
        native_token: native_token,

        reward_to_native_route: [
            {
                astro_swap: {
                    offer_asset_info: {
                        token: { contract_addr: astroport_lp_reward_token },
                    },
                    ask_asset_info: {
                        native_token: { denom: localterra.denoms.usdc },
                    },
                },
            },
        ],
        native_to_lp0_route: [],
        native_to_lp1_route: [
            {
                astro_swap: {
                    offer_asset_info: {
                        native_token: { denom: localterra.denoms.usdc },
                    },
                    ask_asset_info: {
                        native_token: { denom: localterra.denoms.usdt },
                    },
                },
            },
        ],

        name: "Vault Token for USDC-USDT pair",
        symbol: "VTUSDCUSDT",
        decimals: 6,
    });
    vault1_liquid = vaultResult2.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("Liquid contractAddress")}=${vault1_liquid}`);

    // Update the "sibling_vault" config of "vault1_locked"
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, vault1_locked, {
            update_config: {
                sibling_vault: vault1_liquid,
                lp_staking_contract: undefined,
                lp_pair_contract: undefined,
                keeper: undefined,
                tax_collector: undefined,

                native_token: undefined,
                reward_to_native_route: undefined,
                native_to_lp0_route: undefined,
                native_to_lp1_route: undefined,
            },
        }),
    ]);

    console.log(chalk.green(" Done!"));
}
