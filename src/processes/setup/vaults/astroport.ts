/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LocalTerra, Wallet, LCDClient, MsgExecuteContract } from "@terra-money/terra.js";

import { sendTransaction, storeCode, instantiateContract, } from "../../../utils/terra/helpers";
import { wasm_path } from "../../../config/wasmPaths";
import { localterra } from "../../../config/localterraConstants";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let chainId: string;
let terra: LocalTerra | LCDClient;
let apTeam: Wallet;
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
        apTreasury: Wallet;
    },
    config: {
        astroport_factory: string;
        astroport_farming: string;
        astroport_malo_kalo_pair: string;
        astroport_lp_reward_token: string;
        harvest_to_liquid: string;
        accepted_tokens: any | undefined;
        astroport_router: string;
        nativeToken: any,
    }
): Promise<void> {
    chainId = _chainId;
    terra = _terra;
    apTeam = wallets.apTeam;
    apTreasury = wallets.apTreasury;

    await createAstroportVaults(
        config.astroport_factory,
        config.astroport_farming,
        config.astroport_malo_kalo_pair,
        config.astroport_lp_reward_token,
        apTeam.key.accAddress,
        apTeam.key.accAddress,
        config.harvest_to_liquid,
        config.astroport_router,
        config.nativeToken,
    );
}

async function createAstroportVaults(
    loopFactory: string,
    loopFarming: string,
    loopPair: string,
    loopStakingRewardToken: string,
    keeper: string,
    tax_collector: string,
    harvest_to_liquid: string,
    astroport_router: string,
    native_token: string,
): Promise<void> {
    process.stdout.write("Uploading Vault Wasm");
    const vaultCodeId = await storeCode(terra, apTeam, `${wasm_path.core}/astroport_vault.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

    // LOOP Vault - #1 (Locked)
    process.stdout.write("Instantiating Vault #1 (Locked) contract");
    const vaultResult1 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
        acct_type: `locked`, // Locked: 0, Liquid: 1
        sibling_vault: undefined,
        registrar_contract: apTeam.key.accAddress, // Use the `apTeam` address for mock
        keeper: keeper,
        tax_collector: tax_collector,
        swap_router: astroport_router,

        lp_factory_contract: loopFactory,
        lp_staking_contract: loopFarming,
        pair_contract: loopPair,
        lp_reward_token: loopStakingRewardToken,
        native_token: native_token,

        reward_to_native_route: [
            {
                loop: {
                    offer_asset_info: {
                        cw20: localterra.astroport.ASTRO_token_contract,
                    },
                    ask_asset_info: {
                        native: localterra.astroport.ASTRO_token_contract,
                    }
                }
            }
        ],
        native_to_lp0_route: [
            {
                loop: {
                    offer_asset_info: {
                        native: localterra.astroport.ASTRO_token_contract,
                    },
                    ask_asset_info: {
                        cw20: localterra.astroport.ASTRO_token_contract,
                    }
                }
            }
        ],
        native_to_lp1_route: [
            {
                loop: {
                    offer_asset_info: {
                        native: localterra.astroport.ASTRO_token_contract,
                    },
                    ask_asset_info: {
                        cw20: localterra.astroport.ASTRO_token_contract,
                    }
                }
            }
        ],

        name: "Vault Token for MALO-KALO pair",
        symbol: "VTMALOKALO",
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

    // Vault - #1 (Liquid)
    process.stdout.write("Instantiating Vault #1 (Liquid) contract");
    const vaultResult2 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
        acct_type: `liquid`, // Locked: 0, Liquid: 1
        sibling_vault: vault1_locked,
        registrar_contract: apTeam.key.accAddress, // Use the `apTeam` address for mock
        keeper: keeper,
        tax_collector: tax_collector,
        swap_router: astroport_router,

        lp_factory_contract: loopFactory,
        lp_staking_contract: loopFarming,
        pair_contract: loopPair,
        lp_reward_token: loopStakingRewardToken,
        native_token: native_token,

        reward_to_native_route: [
            {
                loop: {
                    offer_asset_info: {
                        cw20: localterra.astroport.ASTRO_token_contract,
                    },
                    ask_asset_info: {
                        native: localterra.astroport.ASTRO_token_contract,
                    }
                }
            }
        ],
        native_to_lp0_route: [
            {
                loop: {
                    offer_asset_info: {
                        native: localterra.astroport.ASTRO_token_contract,
                    },
                    ask_asset_info: {
                        cw20: localterra.astroport.ASTRO_token_contract,
                    }
                }
            }
        ],
        native_to_lp1_route: [
            {
                loop: {
                    offer_asset_info: {
                        native: localterra.astroport.ASTRO_token_contract,
                    },
                    ask_asset_info: {
                        cw20: localterra.astroport.ASTRO_token_contract,
                    }
                }
            }
        ],

        name: "Vault Token for MALO-KALO pair",
        symbol: "VTMALOKALO",
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
        new MsgExecuteContract(
            apTeam.key.accAddress,
            vault1_liquid,
            {
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
                }
            }
        )
    ]);

    console.log(chalk.green(" Done!"));
}
