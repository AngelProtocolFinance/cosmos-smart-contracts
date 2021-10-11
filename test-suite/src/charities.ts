/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, Msg, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  sendTransaction,
} from "./helpers";
import jsonData from "./charity_list.json";
import fs from "fs";

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
  saveEndowments();
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
  process.stdout.write("AP Team approves all verified endowments");
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
  const fund_member_limit = 10;
  let prom = Promise.resolve();
  let id = 1;
  // Split the endowments list into N funds of fund_number_limit
  for (let i = 0; i < endowmentContracts.length; i += fund_member_limit) {
    const members = endowmentContracts.slice(i, i + fund_member_limit);
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(() => new Promise(async (resolve, reject) => {
      try {
        await createIndexFundWithMembers(id ++, members);
        resolve();
      } catch(e) {
        reject(e);
      }
    }));
  }
  await prom;
}


async function createIndexFundWithMembers(id: number, members: string[]): Promise<void> {
  // Create an initial "Fund" with the charities
  process.stdout.write("Create two Funds with two endowments each");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      create_fund: {
        fund: {
          id: id,
          name: `Index Fund #${id}`,
          description: "",
          members: members,
        }
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

const file = "endowment_list.txt";
function saveEndowments(): void {
  const data = endowmentContracts.join(",\n");
  fs.writeFile(file, data, (err) => {
    if (err) {
      return console.error(err);
    }
    console.log("File created!");
  });
}

function readEndowments(): void {
  fs.readFile(file, (err, data) => {
    if (err) {
      return console.error(err);
    }
    console.log(data.toString());
  });
}
