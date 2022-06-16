/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LCDClient, Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import {
  instantiateContract,
  sendTransaction,
  toEncodedBinary,
} from "../../../utils/helpers";

// Deploy HALO Token and HALO/LUNA pair contracts to the TestNet/MainNet
export async function setupTerraSwap(
  terra: LCDClient,
  apTeam: Wallet,
  token_code_id: number,
  factory_contract: string,
  initial_halo_supply: string,
  halo_liquidity: string,
  native_liquidity: string
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
        amount: initial_halo_supply,
      },
    ],
  });
  const tokenContract = tokenResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${tokenContract}`);

  // Pair contract
  process.stdout.write("Creating Pair contract from Token Factory");
  const pairResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, factory_contract, {
      create_pair: {
        asset_infos: [
          {
            token: {
              contract_addr: tokenContract,
            },
          },
          {
            native_token: {
              // denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4",
              denom: "uluna", // only for testnet
            },
          },
        ],
      },
    }),
  ]);

  const pairContract = pairResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);

  // Get the LP Token address of newly created pair
  process.stdout.write("Query new Pair's LP Token contract");
  const result: any = await terra.wasm.contractQuery(pairContract, {
    pair: {},
  });
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${result.liquidity_token}`
  );

  process.stdout.write(
    "Provide liquidity to the new Pair contract @ ratio of 0.05 LUNA per HALO"
  );
  const liqResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, tokenContract, {
      increase_allowance: {
        amount: halo_liquidity,
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
              amount: halo_liquidity,
            },
            {
              info: {
                native_token: {
                  // denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4",
                  denom: "uluna", // only for testnet
                },
              },
              amount: native_liquidity,
            },
          ],
        },
      },
      {
        uluna: native_liquidity,
      }
    ),
  ]);
  console.log(chalk.green(" Done!"));

  // process.stdout.write("Perform simple swap of 1 HALO for LUNA on Pair contract");
  // const swapResult = await sendTransaction(terra, apTeam, [
  //   new MsgExecuteContract(apTeam.key.accAddress, tokenContract, {
  //     send: {
  //       amount: "1000000",
  //       contract: pairContract,
  //       msg: toEncodedBinary({
  //         swap: {},
  //       }),
  //     },
  //   })
  // ]);
  // console.log(chalk.green(" Done!"));
}
