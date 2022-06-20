/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LcdClient, Msg, MsgExecuteContract, Wallet } from "@cosmjs/launchpad";
import { sendTransaction } from "../../../utils/helpers";
import jsonData from "./charity_list.json";
import fs from "fs";

chai.use(chaiAsPromised);

type Charity = {
  endowment_address: string;
  charity_name: string;
  charity_owner: string;
  tier: number;
  charity_overview: string;
  url: string;
  un_sdg: number;
  charity_logo: string;
  charity_image: string;
  total_lock: number;
  total_liq: number;
  overall: number;
  chain: string;
  charity_email: string;
  twitter_handle: string;
  facebook_page: string;
  linkedin_page: string;
  number_of_employees: number;
  charity_registration_number: string;
  country_of_origin: string;
  street_address: string;
  charity_navigator_rating: string;
  annual_revenue: string;
  average_annual_budget: string;
};

let terra: LcdClient;
let apTeam: Wallet;
let registrar: string;
let indexFund: string;
let charities: Charity[];
let endowmentContracts: string[];

export function initializeCharities(
  lcdClient: LcdClient,
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
  jsonData.data.forEach((el) => {
    const item: Charity = el;
    charities.push(el);
  });
}

// setup charity endowments
export async function setupEndowments(): Promise<void> {
  let prom = Promise.resolve();
  charities.forEach((item) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            await createEndowment(item);
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });

  await prom;
  saveEndowments();
}

// Create Endowment base on charity and registrar
async function createEndowment(charity: Charity): Promise<void> {
  process.stdout.write(
    `Charity Endowment ##${charity.charity_name}## created from the Registrar by the AP Team`
  );
  const charityResult = await sendTransaction(juno, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity.charity_owner,
        beneficiary: charity.charity_owner,
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
        guardians_multisig_addr: undefined,
        profile: {
          name: charity.charity_name,
          overview: charity.charity_overview,
          un_sdg: charity.un_sdg,
          tier: charity.tier,
          logo: charity.charity_logo,
          image: charity.charity_image,
          url: charity.url,
          registration_number: charity.charity_registration_number,
          country_city_origin: charity.country_of_origin,
          contact_email: charity.charity_email,
          social_media_urls: {
            facebook: charity.facebook_page,
            twitter: charity.twitter_handle,
            linkedin: charity.linkedin_page,
          },
          number_of_employees: charity.number_of_employees,
          average_annual_budget: charity.average_annual_budget,
          annual_revenue: charity.annual_revenue,
          charity_navigator_rating: charity.charity_navigator_rating,
          endow_type: "Charity",
        },
      },
    }),
  ]);
  const endowmentContract = charityResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract}`
  );
  endowmentContracts.push(endowmentContract);
}

export async function approveEndowments(): Promise<void> {
  // AP Team approves newly created endowments
  process.stdout.write("AP Team approves all verified endowments");
  const msgs: Msg[] = endowmentContracts.map((endowment) => {
    return new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowment,
        status: 1,
        beneficiary: undefined,
      },
    });
  });
  await sendTransaction(juno, apTeam, msgs);
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
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            await createIndexFundWithMembers(id++, members);
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  }
  await prom;
}

async function createIndexFundWithMembers(id: number, members: string[]): Promise<void> {
  // Create an initial "Fund" with the charities
  process.stdout.write(`Create Fund ID#${id} with ${members.length} endowments`);
  await sendTransaction(juno, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      create_fund: {
        fund: {
          id: id,
          name: `Index Fund #${id}`,
          description: "",
          members: members,
        },
      },
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
