/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);

import { Coins, LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";

import { sendTransaction } from "../../utils/terra/helpers";

export async function testExecuteAstroport(
    terra: LocalTerra | LCDClient, // environment config object 
    apTeam: Wallet,
    apTeam2: Wallet,
    apTeam3: Wallet,
    apTreasury: Wallet,
    vaultLocked1: string,
    vaultLiquid1: string,
    vaultLocked2: string,
    vaultLiquid2: string,

    astroportFactory: string,
    astroportGenerator: string,
    astroportRouter: string,
    astroTokenContract: string,
    astroTokenInitialSupply: string,
    usdcUsdtPair: string,
    usdcUsdtPairLpToken: string,
    usdcUsdtPairUsdcLiquidity: string,
    usdcUsdtPairUsdtLiquidity: string,
): Promise<void> {
    console.log(chalk.yellow("\nStep 2. Running Tests"));

    /* - EXECUTE tests - */
    // await testVaultDeposit(terra, apTeam, vaultLocked1, 1, { uluna: 20000 });
    // await testVaultRedeem(terra, apTeam, vaultLocked1, 1, "5000");
    // await testVaultHarvest(terra, apTreasury, vaultLocked1);
    // await testVaultReinvestToLocked(terra, apTeam, vaultLiquid1, 1, "5000");

    /* -  QUERY tests - */
    await testQueryVaultConfig(terra, vaultLocked1);
    await testQueryVaultEndowmentBalance(terra, vaultLocked1, 1);
    await testQueryVaultTokenInfo(terra, vaultLocked1);
    await testQueryVaultTotalBalance(terra, vaultLocked1);
}

//----------------------------------------------------------------------------------------
// Execution tests
//----------------------------------------------------------------------------------------
export async function testVaultDeposit(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault: string,
    endowment_id: number,
    coins: any,
): Promise<void> {
    process.stdout.write("Test - Ibc_relayer deposits to the vault");
    await sendTransaction(terra, sender, [
        new MsgExecuteContract(
            sender.key.accAddress,
            vault,
            {
                deposit: {
                    endowment_id,
                },
            },
            coins,
        ),

    ]);
    console.log(chalk.green(" Passed!"));
}

export async function testVaultRedeem(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault: string,
    endowment_id: number,
    amount: string,
): Promise<void> {
    process.stdout.write("Test - Ibc_relayer deposits to the vault");
    await sendTransaction(terra, sender, [
        new MsgExecuteContract(
            sender.key.accAddress,
            vault,
            {
                redeem: {
                    endowment_id,
                    amount,
                },
            },
        ),

    ]);
    console.log(chalk.green(" Passed!"));
}

export async function testVaultHarvest(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault: string,
): Promise<void> {
    process.stdout.write("Test - Keeper harvests the vault");
    await sendTransaction(terra, sender, [
        new MsgExecuteContract(
            sender.key.accAddress,
            vault,
            {
                harvest: {},
            }
        )
    ]);
    console.log(chalk.green(" Passed!"));
}

export async function testVaultReinvestToLocked(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault: string,
    endowmentId: number,
    amount: string,
): Promise<void> {
    process.stdout.write("Test - Liquid vault reinvests the LP to locked vault");

    await sendTransaction(terra, sender, [
        new MsgExecuteContract(
            sender.key.accAddress,
            vault,
            {
                reinvest_to_locked: {
                    id: endowmentId,
                    amount: amount,
                }
            }
        )
    ]);
    console.log(chalk.green(" Passed!"));
}

export async function testVaultUpdateConfig(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault_addr: string,
    new_config: any | undefined,
): Promise<void> {
    process.stdout.write("Test - Vault owner updates the vault config")
    await sendTransaction(terra, sender, [
        new MsgExecuteContract(
            sender.key.accAddress,
            vault_addr,
            {
                update_config: new_config
            }
        )
    ]);
    console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryVaultConfig(
    terra: LocalTerra | LCDClient,
    vault: string
): Promise<void> {
    process.stdout.write("Test - Query Vault Config\n");
    const result: any = await terra.wasm.contractQuery(vault, {
        config: {},
    });

    console.log(result);
    console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultEndowmentBalance(
    terra: LocalTerra | LCDClient,
    vault: string,
    endowmentId: number,
): Promise<void> {
    process.stdout.write("Test - Query Vault Endowment Balance\n");
    const result: any = await terra.wasm.contractQuery(vault, {
        balance: { endowment_id: endowmentId },
    });

    console.log(`Endow ID #${endowmentId} balance: ${result}`);
    console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultTotalBalance(
    terra: LocalTerra | LCDClient,
    vault: string
): Promise<void> {
    process.stdout.write("Test - Query Vault Total Balance\n");
    const result: any = await terra.wasm.contractQuery(vault, {
        total_balance: {},
    });

    console.log(result);
    console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultTokenInfo(
    terra: LocalTerra | LCDClient,
    vault: string
): Promise<void> {
    process.stdout.write("Test - Query Vault Token Info\n");
    const result: any = await terra.wasm.contractQuery(vault, {
        token_info: {},
    });

    console.log(result);
    console.log(chalk.green(" Passed!"));
}
