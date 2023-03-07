/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
  sendTransaction,
  toEncodedBinary,
  VoteOption,
} from "../../../utils/juno/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

export async function testQueryMultisigConfig(
  juno: SigningCosmWasmClient,
  cw3: string
): Promise<void> {
  process.stdout.write("Test - Query multisig config");
  const result: any = await juno.queryContractSmart(cw3, { config: {} });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryProposal(
  juno: SigningCosmWasmClient,
  multisig: string,
  proposal_id: number
): Promise<void> {
  process.stdout.write("Test - Query a proposal by ID");
  const result: any = await juno.queryContractSmart(multisig, {
    proposal: { proposal_id },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryProposalList(
  juno: SigningCosmWasmClient,
  cw3: string
): Promise<void> {
  process.stdout.write("Test - Query proposal list");
  const result: any = await juno.queryContractSmart(cw3, {
    proposal_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Add a new AP Team Member to the C4 AP Team Group
//
// SCENARIO:
// New AP Team Wallet needs to be added to the C4 Group. Done via a new proposal
// by an existing group member, approved with YES votes, and executed by any wallet.
//
//----------------------------------------------------------------------------------------

export async function testAddMemberToC4Group(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  cw4Grp: string,
  new_member: string
): Promise<void> {
  process.stdout.write(
    "Test - Propose adding a new member to AP Team C4 Group"
  );

  // proposal to add new member
  const proposal = await sendTransaction(juno, apTeam, cw3, {
    propose: {
      title: "New CW4 member",
      description: "New member for the CW4 AP Team Group.",
      msgs: [
        {
          wasm: {
            execute: {
              contract_addr: cw4Grp,
              funds: [],
              msg: toEncodedBinary({
                update_members: {
                  add: [{ addr: new_member, weight: 1 }],
                  remove: [],
                },
              }),
            },
          },
        },
      ],
    },
  });
  const proposal_id = proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value as string;

  console.log(chalk.green(" Passed!"));
}

export async function testProposalApprovingEndowment(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  accounts: string,
  endowment: number
): Promise<void> {
  process.stdout.write("Test - CW3 Member Proposes to Approve an Endowment");

  const proposal = await sendTransaction(juno, apTeam, cw3, {
    propose: {
      title: "Approve an Endowment",
      description: "Proposal to change the status of an endowment to APPROVED",
      msgs: [
        {
          wasm: {
            execute: {
              contract_addr: accounts,
              funds: [],
              msg: toEncodedBinary({
                update_endowment_status: {
                  endowment_id: endowment,
                  status: 1,
                  beneficiary: undefined,
                },
              }),
            },
          },
        },
      ],
    },
  });
  const proposal_id = proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value as string;

  console.log(chalk.green(` Proposal ID: ${proposal_id}`));
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: Cast vote on poll
//
// SCENARIO:
// AP Team CW3 member can vote on the open poll
//
//----------------------------------------------------------------------------------------
export async function testCw3CastVote(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  proposal_id: number,
  // vote: VoteOption,
  vote: string
): Promise<void> {
  process.stdout.write("Test - Cast vote");

  await expect(
    sendTransaction(juno, apTeam, cw3, {
      // vote: { proposal_id, vote: true },
      vote: { proposal_id, vote },
    })
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Cast vote on Charity Application poll
//
// SCENARIO:
// AP Review Team CW3 member can vote on the open Charity Application poll
//
//----------------------------------------------------------------------------------------
export async function testCw3CastApplicationVote(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  proposal_id: number,
  vote: string
): Promise<void> {
  process.stdout.write("Test - Cast vote");

  await expect(
    sendTransaction(juno, apTeam, cw3, {
      vote_application: { proposal_id, vote },
    })
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Execute a poll
//
// SCENARIO:
// AP Team CW3 member can execute a passed poll
//
//----------------------------------------------------------------------------------------
export async function testCw3ExecutePoll(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Execute Poll");

  await expect(
    sendTransaction(juno, apTeam, cw3, {
      execute: { proposal_id: poll_id },
    })
  );
  console.log(chalk.green(" Passed!"));
}

export async function testUpdateCw3Config(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  configMsg: any
): Promise<void> {
  process.stdout.write(
    "Test - Endowment Member Proposes changing the CW3 configs"
  );

  const proposal = await sendTransaction(juno, apTeam, cw3, {
    propose: {
      title: "Update CW3 Configurations",
      description: "Changing the max voting period to 48 hours",
      msgs: [
        {
          wasm: {
            execute: {
              contract_addr: cw3,
              funds: [],
              msg: toEncodedBinary({ update_config: configMsg }),
            },
          },
        },
      ],
    },
  });
  console.log(chalk.green(" Passed!"));
}

export async function testUpdateCw3ApplicationsConfig(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3: string,
  threshold: string,
  max_voting_period: number,
  seed_amount: string,
  seed_split_to_liquid: string,
  dust_amount: string
): Promise<void> {
  process.stdout.write(
    "Test - Endowment Member Proposes changing the CW3 configs"
  );

  const proposal = await sendTransaction(juno, apTeam, cw3, {
    propose: {
      title: "Update CW3 Configurations",
      description: "Changing the max voting period to 48 hours",
      msgs: [
        {
          wasm: {
            execute: {
              contract_addr: cw3,
              funds: [],
              msg: toEncodedBinary({
                update_config: {
                  threshold: { absolute_percentage: { percentage: threshold } },
                  max_voting_period: { height: max_voting_period },
                  require_execution: false,
                  seed_asset: {
                    info: {
                      native:
                        "ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034",
                    },
                    amount: seed_amount,
                  },
                  seed_split_to_liquid,
                  new_endow_gas_money: { denom: "ujuno", amount: dust_amount },
                },
              }),
            },
          },
        },
      ],
    },
  });
  console.log(chalk.green(" Passed!"));
}

export async function testQueryMultisigGroupWeight(
  juno: SigningCosmWasmClient,
  multisig_group: string
): Promise<void> {
  process.stdout.write("Test - Query a multisig group's total weight");
  const result: any = await juno.queryContractSmart(multisig_group, {
    total_weight: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryMultisigVoters(
  juno: SigningCosmWasmClient,
  multisig: string
): Promise<void> {
  process.stdout.write("Test - Query a multisig voters list");
  const result: any = await juno.queryContractSmart(multisig, {
    list_voters: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryListProposals(
  juno: SigningCosmWasmClient,
  multisig: string
): Promise<void> {
  process.stdout.write("Test - Query a list of all proposals");
  const result: any = await juno.queryContractSmart(multisig, {
    list_proposals: { start_after: undefined, limit: undefined },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryMultisigThreshold(
  juno: SigningCosmWasmClient,
  multisig: string
): Promise<void> {
  process.stdout.write("Test - Query a multisig threshold");
  const result: any = await juno.queryContractSmart(multisig, {
    threshold: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGroupMembersList(
  juno: SigningCosmWasmClient,
  multisig: string
): Promise<void> {
  process.stdout.write("Test - Query a multisig group members list");
  const result: any = await juno.queryContractSmart(multisig, {
    list_members: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
