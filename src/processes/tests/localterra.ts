import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";

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

}