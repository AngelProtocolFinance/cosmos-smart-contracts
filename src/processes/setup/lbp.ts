/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LocalTerra, LCDClient, Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import { instantiateContract, sendTransaction, storeCode } from "../../utils/helpers";
import { wasm_path } from "../../config/wasmPaths";

// Deploy HALO/UST LBP pair and LP Token contracts to TestNet/MainNet
export async function setupLbp(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  terraswapToken: string,
  haloTokenAmount: string,
  nativeTokenAmount: string,
  start_time: number,
  end_time: number,
  token_start_weight: string,
  token_end_weight: string,
  native_start_weight: string,
  native_end_weight: string,
  description: string | undefined
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
  const pairContract = await createPair(
    terra,
    apTeam,
    factoryContract,
    terraswapToken,
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
  await provideLiquidity(
    terra,
    apTeam,
    terraswapToken,
    pairContract,
    haloTokenAmount,
    nativeTokenAmount
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

async function createPair(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  terraswapToken: string,
  start_time: number,
  end_time: number,
  token_start_weight: string,
  token_end_weight: string,
  native_start_weight: string,
  native_end_weight: string,
  description: string | undefined
): Promise<string> {
  process.stdout.write("Creating Pair contract from Factory contract");

  const pairResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, factoryContract, {
      create_pair: {
        asset_infos: [
          {
            info: {
              token: {
                contract_addr: terraswapToken,
              },
            },
            start_weight: token_start_weight,
            end_weight: token_end_weight,
          },
          {
            info: {
              native_token: {
                denom: "uusd".toString(),
              },
            },
            start_weight: native_start_weight,
            end_weight: native_end_weight,
          },
        ],
        start_time,
        end_time,
        description,
      },
    }),
  ]);

  const pairContract = pairResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);

  return pairContract;
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

async function provideLiquidity(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  terraswapToken: string,
  pairContract: string,
  haloTokenAmount: string,
  nativeTokenAmount: string
): Promise<void> {
  process.stdout.write("Provide liquidity to the New Pair contract");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, terraswapToken, {
      increase_allowance: {
        amount: haloTokenAmount,
        spender: pairContract,
      },
    }),
    new MsgExecuteContract(
      apTeam.key.accAddress,
      pairContract,
      {
        provide_liquidity: {
          assets: [
            {
              info: {
                token: {
                  contract_addr: terraswapToken,
                },
              },
              amount: haloTokenAmount,
            },
            {
              info: {
                native_token: {
                  denom: "uusd",
                },
              },
              amount: nativeTokenAmount,
            },
          ],
        },
      },
      {
        uusd: nativeTokenAmount,
      }
    ),
  ]);
  console.log(chalk.green(" Done!"));
}

async function getPairContractLpToken(
  terra: LocalTerra | LCDClient,
  pairContract: string
): Promise<void> {
  process.stdout.write("Query new Pair's LP Token contract");
  const result: any = await terra.wasm.contractQuery(pairContract, {
    pair: {},
  });
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${result.liquidity_token}`
  );
}
