/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
  instantiateContract,
  sendTransaction,
  toEncodedBinary,
} from "../../../utils/juno/helpers";

// Deploy HALO Token and HALO/JUNO pair contracts to the TestNet/MainNet
export async function setupJunoSwap(
  juno: SigningCosmWasmClient,
  apTeam: string,
  token_code_id: number,
  factory_contract: string,
  initial_halo_supply: string,
  halo_liquidity: string,
  native_liquidity: string
): Promise<void> {
  // HALO token contract
  process.stdout.write("Instantiating HALO Token contract");
  const tokenResult = await instantiateContract(juno, apTeam, apTeam, token_code_id, {
    name: "Angel Protocol",
    symbol: "HALO",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam,
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
  const pairResult = await sendTransaction(juno, apTeam, factory_contract, {
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
            denom: "ujuno", // only for testnet
          },
        },
      ],
    },
  });

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
  const result: any = await juno.queryContractSmart(pairContract, {
    pair: {},
  });
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${result.liquidity_token}`
  );

  process.stdout.write("Increase the allowance for the Pair Contract");
  const allowResult = await sendTransaction(juno, apTeam, tokenContract, {
    increase_allowance: {
      amount: halo_liquidity,
      spender: pairContract,
    },
  });
  console.log(chalk.green(" Done!"));

  process.stdout.write(
    "Provide liquidity to the new Pair contract @ ratio of 0.05 JUNO per HALO"
  );
  const liqResult = await sendTransaction(juno, apTeam, pairContract, {
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
              denom: "ujuno", // only for testnet
            },
          },
          amount: native_liquidity,
        },
      ],
    },
  },
    [{ denom: "ujuno", amount: native_liquidity }]
  );
  console.log(chalk.green(" Done!"));
}
