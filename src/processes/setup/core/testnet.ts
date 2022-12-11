/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { sendTransaction, storeCode, instantiateContract, getWalletAddress, sendMessageViaCw3Proposal, sendApplicationViaCw3Proposal } from "../../../utils/juno/helpers";
import { wasm_path } from "../../../config/wasmPaths";
import { localjuno } from "../../../config/localjunoConstants";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let networkUrl: string;

let juno: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTeam2: DirectSecp256k1HdWallet;
let apTreasury: DirectSecp256k1HdWallet;

// wallet addresses
let apTeamAddr: string;
let apTeam2Addr: string;
let apTreasuryAddr: string;

// contract addresses
let registrar: string;
let accounts: string;
let cw4GrpApTeam: string;
let cw3ApTeam: string;
let cw4GrpReviewTeam: string;
let cw3ReviewTeam: string;
let indexFund: string;
let donationMatchCharities: string;
let swapRouter: string;
let settingsController: string;

let vault1: string;
let vault2: string;

let endow_1_id: number;
let endow_2_id: number;
let endow_3_id: number;


// -------------------------------------------------------------------------------------
// setup all contracts for LocalJuno and TestNet
// -------------------------------------------------------------------------------------

export async function setupCore(
	_networkUrl: any,
	_juno: SigningCosmWasmClient,
	wallets: {
		apTeam: DirectSecp256k1HdWallet;
		apTeam2: DirectSecp256k1HdWallet;
		apTeam3: DirectSecp256k1HdWallet;
		apTreasury: DirectSecp256k1HdWallet;
	},
	config: {
		tax_rate: string;
		threshold_absolute_percentage: string;
		max_voting_period_height: number;
		fund_rotation: number | undefined;
		harvest_to_liquid: string;
		funding_goal: string | undefined;
		fund_member_limit: number | undefined;
		charity_cw3_threshold_abs_perc: string,
		charity_cw3_max_voting_period: number,
		accepted_tokens: any | undefined;
	}
): Promise<void> {
	networkUrl = _networkUrl;
	juno = _juno;
	apTeam = wallets.apTeam;
	apTeam2 = wallets.apTeam2;
	apTreasury = wallets.apTreasury;

	apTeamAddr = await getWalletAddress(apTeam);
	apTeam2Addr = await getWalletAddress(apTeam2);
	apTreasuryAddr = await getWalletAddress(apTreasury);

	await setup(
		config.tax_rate,
		apTreasuryAddr,
		config.threshold_absolute_percentage,
		config.max_voting_period_height,
		config.fund_rotation,
		config.fund_member_limit,
		config.funding_goal,
		config.accepted_tokens,
	);
	await turnOverApTeamMultisig();
	// await createIndexFunds();
}

async function setup(
	tax_rate: string,
	treasury_address: string,
	threshold_absolute_percentage: string,
	max_voting_period_height: number,
	fund_rotation: number | undefined,
	fund_member_limit: number | undefined,
	funding_goal: string | undefined,
	accepted_tokens: any | undefined,
): Promise<void> {
	// Step 1. Upload all local wasm files and capture the codes for each....
	process.stdout.write("Uploading Registrar Wasm");
	const registrarCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/registrar.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${registrarCodeId}`);

	process.stdout.write("Uploading Index Fund Wasm");
	const fundCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/index_fund.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${fundCodeId}`);

	process.stdout.write("Uploading Accounts Wasm");
	const accountsCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/accounts.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${accountsCodeId}`);

	process.stdout.write("Uploading Settings-Controller Wasm");
	const settingsControllerCodId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/settings_controller.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${settingsControllerCodId}`);

	process.stdout.write("Uploading CW4 Group Wasm");
	const cw4Group = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw4_group.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw4Group}`);

	process.stdout.write("Uploading AP Team CW3 MultiSig Wasm");
	const cw3MultiSigApTeam = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw3_apteam.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigApTeam}`);

	process.stdout.write("Uploading Review Team CW3 MultiSig Wasm");
	const cw3MultiSigApplications = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw3_applications.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigApplications}`);

	process.stdout.write("Uploading Endowment CW3 MultiSig Wasm");
	const cw3MultiSigEndowment = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw3_endowment.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigEndowment}`);

	process.stdout.write("Uploading Endowment SubDAO Wasm");
	const subdao = await storeCode(juno, apTeamAddr, `${wasm_path.core}/subdao.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdao}`);
	process.stdout.write("Uploading SwapRouter Wasm");
	const swapRouterCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/swap_router.wasm`);
	console.log(chalk.green(" Done!", `${chalk.blue("codeId")}=${swapRouterCodeId}`))

	process.stdout.write("Uploading Endowment SubDAO Bonding Token Wasm");
	const subdaoBondingToken = await storeCode(juno, apTeamAddr, `${wasm_path.core}/subdao_bonding_token.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdaoBondingToken}`);

	process.stdout.write("Uploading Endowment SubDAO CW20 Token Wasm");
	const subdaoCw20Token = await storeCode(juno, apTeamAddr, `${wasm_path.cw20}`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdaoCw20Token}`);

	process.stdout.write("Uploading Endowment SubDAO Donation Matching Wasm");
	const subdaoDonationMatch = await storeCode(juno, apTeamAddr, `${wasm_path.core}/donation_match.wasm`);
	console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdaoDonationMatch}`);

	// Step 2. Instantiate the key contracts
	// Registrar
	process.stdout.write("Instantiating Registrar contract");
	const registrarResult = await instantiateContract(
		juno,
		apTeamAddr,
		apTeamAddr,
		registrarCodeId,
		{
			tax_rate,
			treasury: treasury_address,
			default_vault: apTeamAddr, // Fake vault address from apTeam
			split_to_liquid: undefined,
			accepted_tokens: accepted_tokens,
		}
	);
	registrar = registrarResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);

	// Index Fund
	process.stdout.write("Instantiating Index Fund contract");
	const fundResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, fundCodeId, {
		registrar_contract: registrar,
		fund_rotation: fund_rotation,
		fund_member_limit: fund_member_limit,
		funding_goal: funding_goal,
		accepted_tokens: accepted_tokens,
	});
	indexFund = fundResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${indexFund}`);

	// CW4 AP Team Group
	process.stdout.write("Instantiating CW4 AP Team Group contract");
	const cw4GrpApTeamResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, cw4Group, {
		admin: apTeamAddr,
		members: [
			{ addr: apTeamAddr, weight: 1 },
			{ addr: apTeam2Addr, weight: 1 },
		],
	});
	cw4GrpApTeam = cw4GrpApTeamResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw4GrpApTeam}`);

	// CW3 AP Team MultiSig
	process.stdout.write("Instantiating CW3 AP Team MultiSig contract");
	const cw3ApTeamResult = await instantiateContract(
		juno,
		apTeamAddr,
		apTeamAddr,
		cw3MultiSigApTeam,
		{
			registrar_contract: registrar,
			group_addr: cw4GrpApTeam,
			threshold: { absolute_percentage: { percentage: threshold_absolute_percentage } },
			max_voting_period: { height: max_voting_period_height },
			// registrar_contract: registrar,
		}
	);
	cw3ApTeam = cw3ApTeamResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw3ApTeam}`);

	// Setup AP Team C3 to be the admin to it's C4 Group
	process.stdout.write(
		"AddHook & UpdateAdmin on AP Team CW4 Group to point to AP Team C3"
	);
	await sendTransaction(juno, apTeamAddr, cw4GrpApTeam, {
		add_hook: { addr: cw3ApTeam },
	});
	await sendTransaction(juno, apTeamAddr, cw4GrpApTeam, {
		update_admin: { admin: cw3ApTeam },
	});
	console.log(chalk.green(" Done!"));

	process.stdout.write("Instantiating Settings-Controller contract");
	const settingsControllerResult = await instantiateContract(
		juno,
		apTeamAddr,
		apTeamAddr,
		settingsControllerCodId,
		{
			owner_sc: apTeamAddr,
			registrar_contract: registrar,
		}
	);
	settingsController = settingsControllerResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${settingsController}`);

	process.stdout.write("Instantiating Accounts contract");
	const accountsResult = await instantiateContract(
		juno,
		apTeamAddr,
		apTeamAddr,
		accountsCodeId,
		{
			owner_sc: apTeamAddr,
			registrar_contract: registrar,
		}
	);
	accounts = accountsResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${accounts}`);

	// Charities Donation Matching
	process.stdout.write("Instantiating Charities Donation Matching contract");
	const charityDonationMatchResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, subdaoDonationMatch, {
		id: 1, // FAKE! Need to fix.
		registrar_contract: registrar,
		reserve_token: apTeamAddr, // FAKE! Need to fix.
		lp_pair: apTeamAddr, // FAKE! Need to fix.
	});
	donationMatchCharities = charityDonationMatchResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${donationMatchCharities}`);

	// Swap-Rotuer
	process.stdout.write("Instantiating the Swap-router contract")
	const swapRouterResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, swapRouterCodeId, {
		registrar_contract: registrar,
		accounts_contract: accounts,
		pairs: [
			{
				assets: [
					{
						native: localjuno.networkInfo.nativeToken,
					},
					{
						cw20: localjuno.loopswap.malo_token_contract,
					}
				],
				contract_address: localjuno.loopswap.malo_juno_pair_contract,
			},
			{
				assets: [
					{
						native: localjuno.networkInfo.nativeToken,
					},
					{
						cw20: localjuno.loopswap.kalo_token_contract,
					}
				],
				contract_address: localjuno.loopswap.kalo_juno_pair_contract,
			},
			{
				assets: [
					{
						cw20: localjuno.loopswap.malo_token_contract,
					},
					{
						cw20: localjuno.loopswap.kalo_token_contract,
					}
				],
				contract_address: localjuno.loopswap.malo_kalo_pair_contract,
			},
			{
				assets: [
					{
						native: localjuno.networkInfo.nativeToken,
					},
					{
						cw20: localjuno.loopswap.loop_token_contract,
					}
				],
				contract_address: localjuno.loopswap.loop_juno_pair_contract,
			}
		],
	});
	swapRouter = swapRouterResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${swapRouter}`);

	// CW4 Review Team Group
	process.stdout.write("Instantiating CW4 Review Team Group contract");
	const cw4GrpReviewTeamResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, cw4Group, {
		admin: apTeamAddr,
		members: [
			{ addr: apTeamAddr, weight: 1 },
		],
	});
	cw4GrpReviewTeam = cw4GrpReviewTeamResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw4GrpReviewTeam}`);

	// CW3 Review Team MultiSig
	process.stdout.write("Instantiating CW3 Review Team MultiSig contract");
	const cw3ReviewTeamResult = await instantiateContract(
		juno,
		apTeamAddr,
		apTeamAddr,
		cw3MultiSigApplications,
		{
			registrar_contract: registrar,
			group_addr: cw4GrpReviewTeam,
			threshold: { absolute_percentage: { percentage: threshold_absolute_percentage } },
			max_voting_period: { height: max_voting_period_height },
			// registrar_contract: registrar,
		}
	);
	cw3ReviewTeam = cw3ReviewTeamResult.contractAddress as string;
	console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw3ReviewTeam}`);

	// Setup AP Team C3 to be the admin to it's C4 Group
	process.stdout.write(
		"AddHook & UpdateAdmin on AP Review Team CW4 Group to point to AP Team C3"
	);
	await sendTransaction(juno, apTeamAddr, cw4GrpReviewTeam, {
		add_hook: { addr: cw3ReviewTeam },
	});
	await sendTransaction(juno, apTeamAddr, cw4GrpReviewTeam, {
		update_admin: { admin: cw3ReviewTeam },
	});
	console.log(chalk.green(" Done!"));

	process.stdout.write("Update Registrar's config with various wasm codes & contracts");
	await sendTransaction(juno, apTeamAddr, registrar, {
		update_config: {
			accounts_contract: accounts,
			applications_review: cw3ReviewTeam,
			index_fund_contract: indexFund,
			cw3_code: cw3MultiSigEndowment,
			cw4_code: cw4Group,
			halo_token: apTeamAddr, // Fake halo token addr: Need to be handled
			halo_token_lp_contract: apTeamAddr, // Fake halo token LP addr: Need to be handled
			subdao_gov_code: subdao,
			subdao_cw20_token_code: subdaoCw20Token,
			subdao_bonding_token_code: subdaoBondingToken,
			donation_match_code: subdaoDonationMatch,
			donation_match_charites_contract: donationMatchCharities,
		},
	});
	console.log(chalk.green(" Done!"));

	process.stdout.write("Update Accounts's config with various contracts");
	await sendTransaction(juno, apTeamAddr, accounts, {
		update_config: {
			new_registrar: registrar,
			max_general_category_id: 1,
			ibc_controller: undefined,
			settings_controller: settingsController,
		},
	});
	console.log(chalk.green(" Done!"));
}

// Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract
async function turnOverApTeamMultisig(): Promise<void> {
	process.stdout.write(
		"Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract\n"
	);
	process.stdout.write(chalk.yellow("\n> Turning over Registrar"));
	await sendTransaction(juno, apTeamAddr, registrar, {
		update_owner: { new_owner: cw3ApTeam }
	});
	console.log(chalk.green(" Done!"));

	process.stdout.write(chalk.yellow("- Turning over Index Fund"));
	await sendTransaction(juno, apTeamAddr, indexFund, {
		update_owner: { new_owner: cw3ApTeam }
	});
	console.log(chalk.green(" Done!"));

	process.stdout.write(chalk.yellow("- Turning over Accounts"));
	await sendTransaction(juno, apTeamAddr, accounts, {
		update_owner: { new_owner: cw3ApTeam }
	});
	console.log(chalk.green(" Done!"));
}

// // Step 5: Index Fund finals setup
// async function createIndexFunds(): Promise<void> {
//   // Create an initial "Fund" with the two charities created above
//   process.stdout.write("Create two Funds with two endowments each");
//   await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, indexFund, {
//     create_fund: {
//       name: "Test Fund",
//       description: "My first test fund",
//       members: [endow_1_id, endow_2_id],
//       rotating_fund: true,
//       split_to_liquid: undefined,
//       expiry_time: undefined,
//       expiry_height: undefined,
//     }
//   });
//   await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, indexFund, {
//     create_fund: {
//       name: "Test Fund #2",
//       description: "Another fund to test rotations",
//       members: [endow_1_id, endow_3_id],
//       rotating_fund: true,
//       split_to_liquid: undefined,
//       expiry_time: undefined,
//       expiry_height: undefined,
//     }
//   });
//   console.log(chalk.green(" Done!"));
// }
