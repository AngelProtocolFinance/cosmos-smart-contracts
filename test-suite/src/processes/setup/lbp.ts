/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LocalTerra, LCDClient, Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import { instantiateContract, sendTransaction, storeCode } from "../../utils/helpers";

// Deploy HALO Token and HALO/UST pair contracts to the TestNet/MainNet
export async function setupLBP(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenAmount: string,
  nativeTokenAmount: string,
  commission_rate: string,
  collector_addr: string,
  start_time: number,
  end_time: number,
  description: string | undefined,
  ): Promise<void> {
  process.stdout.write("Uploading LBP factory Wasm");
  const factoryCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/lbp_factory.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${factoryCodeId}`);

  process.stdout.write("Uploading LBP pair Wasm");
  const pairCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/lbp_pair.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${pairCodeId}`);

  process.stdout.write("Uploading LBP token Wasm");
  const tokenCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/lbp_token.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${tokenCodeId}`);

  process.stdout.write("Uploading LBP router Wasm");
  const routerCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/lbp_router.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${routerCodeId}`);

  // HALO token contract
  const tokenContract = await setupToken(
    terra,
    apTeam,
    tokenCodeId,
    tokenAmount,
  );

  // Factory contract
  const factoryContract = await setupFactory(
    terra,
    apTeam,
    factoryCodeId,
    pairCodeId,
    tokenCodeId,
    commission_rate,
    collector_addr,
  );

  // Create Pair contract
  const pairContract = await createPair(
    terra,
    apTeam,
    factoryContract,
    tokenContract,
    start_time,
    end_time,
    description,
  );

  // Router contract
  await setupRouter(
    terra,
    apTeam,
    routerCodeId,
    factoryContract
  );

  // Provide liquidity
  await provideLiquidity(
    terra,
    apTeam,
    tokenContract,
    pairContract,
    tokenAmount,
    nativeTokenAmount,
  );
}

async function setupToken(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenCodeId: number,
  tokenAmount: string,
  ): Promise<string> {
  // HALO token contract
  process.stdout.write("Instantiating HALO Token contract");
  const tokenResult = await instantiateContract(terra, apTeam, apTeam, tokenCodeId, {
    name: "Angel Protocol",
    symbol: "HALO",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam.key.accAddress,
        amount: tokenAmount
      }
    ]
  });
  const tokenContract = tokenResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${tokenContract}`);

  return tokenContract;
}

async function setupFactory(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryCodeId: number,
  pairCodeId: number,
  tokenCodeId: number,
  commission_rate: string,
  collector_addr: string,
): Promise<string> {
  process.stdout.write("Instantiating Factory contract");
  const factoryResult = await instantiateContract(terra, apTeam, apTeam, factoryCodeId, {
    pair_code_id: pairCodeId,
    token_code_id: tokenCodeId,
    owner: apTeam.key.accAddress,
    commission_rate,
    collector_addr,
  });
  const factoryContract = factoryResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${factoryContract}`);

  return factoryContract;
}

async function createPair(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  tokenContract: string,
  start_time: number,
  end_time: number,
  description: string | undefined,
): Promise<string> {
  process.stdout.write("Creating Pair contract from Factory contract");

  const pairResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, factoryContract, {
      create_pair: {
        asset_infos: [
          {
            info:{
              token: {
                contract_addr: tokenContract,
              }
            },
            start_weight: "1",
            end_weight: "1"
          },
          {
            info:{
              native_token: {
                denom: "uusd".toString()
              }
            },
            start_weight: "1",
            end_weight: "1"
          }
        ],
        start_time,
        end_time,
        description,
      }
    })
  ]);

  const pairContract = pairResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);

  return pairContract;
}

async function setupRouter(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  routerCodeId: number,
  factoryContract: string,
): Promise<void> {
  process.stdout.write("Instantiating LBP Router contract");
  const routerResult = await instantiateContract(terra, apTeam, apTeam, routerCodeId, {
    halo_factory: factoryContract
  });
  const routerContract = routerResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${routerContract}`);
}

async function provideLiquidity(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenContract: string,
  pairContract: string,
  tokenAmount: string,
  nativeTokenAmount: string,
): Promise<void> {
  process.stdout.write("Provide liquidity to the New Pair contract");
  const liqAddResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, tokenContract, {
      increase_allowance: {
        amount: tokenAmount,
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
                  contract_addr: tokenContract,
                },
              },
              amount: tokenAmount,
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