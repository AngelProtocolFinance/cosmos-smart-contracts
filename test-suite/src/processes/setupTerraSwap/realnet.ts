/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LCDClient, Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import { instantiateContract, sendTransaction, toEncodedBinary } from "../../utils/helpers";

// Deploy HALO Token and HALO/UST pair contracts to the TestNet/MainNet
export async function setupTerraSwap(
  terra: LCDClient,
  apTeam: Wallet,
  token_code_id: number,
  factory_contract: string,
  ): Promise<void> {

  // HALO token contract
  process.stdout.write("Instantiating HALO Token contract");
  const tokenResult = await instantiateContract(terra, apTeam, apTeam, token_code_id, {
    name: "Angel Protocol",
    symbol: "HALO",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam.key.accAddress,
        amount: "1000000000000000"
      }
    ]
  });
  const tokenContract = tokenResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${tokenContract}`);

  // Pair contract
  process.stdout.write("Creating Pair contract from Token Factory");
  const pairResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, factory_contract, {
      "create_pair": {
        "asset_infos": [
          {
            "token": {
              "contract_addr": tokenContract,
            }
          },
          {
            "native_token": {
              "denom": "uusd"
            }
          }
        ]
      }
    })
  ]);

  const pairContract = pairResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);

  process.stdout.write("Provide liquidity to the new Pair contract @ ratio of 0.05 UST per HALO");
  const liqResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, tokenContract, {
      increase_allowance: {
        amount: "2000000000",
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
              amount: "2000000000",
            },
            {
              info: {
                native_token: {
                  denom: "uusd",
                },
              },
              amount: "100000000",
            },
          ],
        },
      },
      {
        uusd: "100000000",
      }
    ),
  ]);
  console.log(chalk.green(" Done!"));

  process.stdout.write("Perform simple swap of 1 HALO for UST on Pair contract");
  const swapResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, tokenContract, {
      send: {
        amount: "1000000",
        contract: pairContract,
        msg: toEncodedBinary({
          swap: {},
        }),
      },
    })
  ]);
  console.log(chalk.green(" Done!"));
}