/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, Msg, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  sendTransaction,
} from "./helpers";
import jsonData from "./charity_list.json";

chai.use(chaiAsPromised);

type Charity = {
  address: string,
  name: string,
  description: string,
}

let terra: LCDClient;
let apTeam: Wallet;
let registrar: string;
let indexFund: string;
let charities: Charity[];
let endowmentContracts: string[];

export function initializeCharities(
  lcdClient: LCDClient,
  ap_team: Wallet,
  registrarAddr: string,
  index_fund: string
): void {
  terra = lcdClient;
  apTeam = ap_team;
  registrar = registrarAddr;
  indexFund = index_fund;

  charities = [];
  endowmentContracts = [];
  jsonData.data.forEach(el => {
    const item: Charity = {
      address: el.address,
      name: el.name,
      description: el.description
    };
    charities.push(item);
  })
}

// setup charity endowments
export async function setupEndowments(): Promise<void> {
  let prom = Promise.resolve();
  charities.forEach(item => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(() => new Promise(async (resolve, reject) => {
      try {
        await createEndowment(item);
        resolve();
      } catch(e) {
        reject(e);
      }
    }));
  });

  await prom;
}

// Create Endowment base on charity and registrar
async function createEndowment(charity: Charity): Promise<void> {
  process.stdout.write("Charity Endowment #1 created from the Registrar by the AP Team");
  const charityResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity.address,
        beneficiary: charity.address,
        name: charity.name,
        description: charity.description,
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
      }
    }),
  ]);
  const endowmentContract = charityResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${endowmentContract}`);
  endowmentContracts.push(endowmentContract);
}

export async function approveEndowments(): Promise<void> {
  // AP Team approves 3 of 4 newly created endowments
  process.stdout.write("AP Team approves 3 of 4 endowments");
  const msgs: Msg[] = endowmentContracts.map(endowment => {
    return new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowment,
        status: 1,
        beneficiary: undefined,
      }
    });
  })
  await sendTransaction(terra, apTeam, msgs);
  console.log(chalk.green(" Done!"));
}

// Create an initial "Fund" with the charities created above
export async function createIndexFunds(): Promise<void> {
  // Create an initial "Fund" with the two charities created above
  process.stdout.write("Create two Funds with two endowments each");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      create_fund: {
        fund: {
          id: 1,
          name: "First Fund",
          description: "My first test fund",
          members: endowmentContracts,
        }
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));
}
