/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LocalTerra, LCDClient, Wallet } from "@terra-money/terra.js";
import { instantiateContract, storeCode } from "../../utils/helpers";
import { testFactoryCreatePair } from "../tests/lbp/factory";
import { testPairProvideLiquidity, getPairContractLpToken } from "../tests/lbp/pair";
import { wasm_path } from "../../config/wasmPaths";

// Deploy HALO/UST LBP pair and LP Token contracts to TestNet/MainNet
export async function setupLbp(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  terraswapToken: string,
  haloTokenAmount: string,
  nativeTokenAmount: string,
  denom: string,
  start_time: number,
  end_time: number,
  token_start_weight: string,
  token_end_weight: string,
  native_start_weight: string,
  native_end_weight: string,
  description: string | undefined,
  slippage_tolerance: string | undefined
): Promise<void> {
  process.stdout.write("Uploading LBP factory Wasm");
  const factoryCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.lbp}/astroport_lbp_factory.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${factoryCodeId}`);

  process.stdout.write("Uploading LBP pair Wasm");
  const pairCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.lbp}/astroport_lbp_pair.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${pairCodeId}`);

  process.stdout.write("Uploading LBP token Wasm");
  const tokenCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.lbp}/astroport_lbp_token.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${tokenCodeId}`);

  process.stdout.write("Uploading LBP router Wasm");
  const routerCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.lbp}/astroport_lbp_router.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${routerCodeId}`);

  // Factory contract
  const factoryContract = await setupFactory(
    terra,
    apTeam,
    factoryCodeId,
    pairCodeId,
    tokenCodeId
  );

  // Router contract
  await setupRouter(terra, apTeam, routerCodeId, factoryContract);

  // Create Pair contract
  const pairContract = await testFactoryCreatePair(
    terra,
    apTeam,
    factoryContract,
    terraswapToken,
    denom,
    start_time,
    end_time,
    token_start_weight,
    token_end_weight,
    native_start_weight,
    native_end_weight,
    description
  );

  // Get the LP Token address of newly created pair
  await getPairContractLpToken(terra, pairContract);

  // Provide liquidity
  await testPairProvideLiquidity(
    terra,
    apTeam,
    terraswapToken,
    pairContract,
    haloTokenAmount,
    nativeTokenAmount,
    slippage_tolerance
  );
}

async function setupFactory(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryCodeId: number,
  pairCodeId: number,
  tokenCodeId: number
): Promise<string> {
  process.stdout.write("Instantiating Factory contract");
  const factoryResult = await instantiateContract(terra, apTeam, apTeam, factoryCodeId, {
    pair_code_id: pairCodeId,
    token_code_id: tokenCodeId,
    owner: apTeam.key.accAddress,
  });
  const factoryContract = factoryResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${factoryContract}`
  );

  return factoryContract;
}

async function setupRouter(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  routerCodeId: number,
  factoryContract: string
): Promise<void> {
  process.stdout.write("Instantiating LBP Router contract");
  const routerResult = await instantiateContract(terra, apTeam, apTeam, routerCodeId, {
    astroport_lbp_factory: factoryContract,
  });
  const routerContract = routerResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${routerContract}`
  );
}
