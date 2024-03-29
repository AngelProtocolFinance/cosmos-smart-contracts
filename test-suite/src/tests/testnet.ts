import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { datetimeStringToUTC, clientSetup } from "../utils/helpers/juno";
import {
  testEndowmentCanWithdrawLiquid,
  testCharityCanWithdrawLocked,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testApTeamChangesEndowmentSettings,
  testSendDonationToEndowment,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testQueryAccountsState,
  testCharityCanHarvestWithdrawFee,
  testCharityCanHarvestAUMFee,
  testApproveInactiveEndowment,
  testUpdateEndowmentStatus,
  testCreateEndowment,
  testCreateNormalEndowment,
  testQueryAccountsEndowmentByProposalLink,
  testEndowmentVaultsRedeem,
  testSendRestitutionFundsToEndowments,
} from "./core/accounts";
import {
  testSetupDao,
  testSetupDonationMatch,
  testUpdateDelegate,
  testUpdateSettingsControllerConfig,
  testQuerySettingsControllerConfig,
  testQuerySettingsControllerEndowPermissions,
  testQuerySettingsControllerEndowSettings,
  testQuerySettingsControllerEndowController,
} from "./core/settingsController";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdatingIndexFundConfigs,
  testCreateIndexFund,
  testRemoveIndexFund,
  testQueryIndexFundActiveFundDetails,
  testQueryIndexFundActiveFundDonations,
  testQueryIndexFundConfig,
  testQueryIndexFundDeposit,
  testQueryIndexFundFundDetails,
  testQueryIndexFundState,
  testIndexFundUpdateOwner,
  testIndexFundRemoveMember,
  testIndexFundUpateRegistrar,
  testUpdatingIndexFundOwner,
  testUpdatingIndexFundRegistrar,
} from "./core/indexFunds";
import {
  testUpdateCw3Config,
  testUpdateCw3ApplicationsConfig,
  testAddMemberToC4Group,
  testProposalApprovingEndowment,
  testCw3CastVote,
  testCw3CastApplicationVote,
  testCw3ExecutePoll,
  testQueryMultisigGroupWeight,
  testQueryMultisigVoters,
  testQueryMultisigThreshold,
  testQueryListProposals,
  testQueryGroupMembersList,
  testQueryProposal,
  testQueryProposalList,
  testQueryMultisigConfig,
  testQueryApplicationsCw3Balances,
} from "./core/multisig";
import {
  testUpdatingRegistrarConfigs,
  testUpdateFees,
  testUpdatingRegistrarNetworkConnections,
  testUpdatingRegistrarUpdateOwner,
  testQueryRegistrarConfig,
  testQueryRegistrarNetworkConnection,
  testQueryRegistrarStrategy,
} from "./core/registrar";
import {
  testQueryVaultConfig,
  testVaultHarvest,
  testVaultReinvestToLocked,
  testQueryVaultTotalBalance,
  testQueryVaultEndowmentBalance,
  testQueryVaultTokenInfo,
  testVaultUpdateConfig,
} from "./core/vaults";
import {
  testAirdropClaim,
  testAirdropRegisterNewMerkleRoot,
  testAirdropUpdateConfig,
  testQueryAirdropMerkleRoot,
  testQueryAirdropConfig,
  testQueryAirdropIsClaimed,
  testQueryAirdropLatestStage,
} from "./halo/airdrop";
import {
  testCollectorUpdateConfig,
  testCollectorSweep,
  testQueryCollectorConfig,
  testQueryCollectorPair,
} from "./halo/collector";
import {
  testCommunityUpdateConfig,
  testCommunitySpend,
  testQueryCommunityConfig,
} from "./halo/community";
import {
  testDistributorUpdateConfig,
  testDistributorAdd,
  testDistributorRemove,
  testDistributorSpend,
  testQueryDistributorConfig,
} from "./halo/distributor";
import {
  testGovCastVote,
  testGovEndPoll,
  testGovExecutePoll,
  testGovRegisterContracts,
  testGovSnapshotPoll,
  testGovUpdateConfig,
  testGovStakeVotingTokens,
  testGovWithdrawVotingTokens,
  testGovClaimVotingTokens,
  testGovExecutePollForRegistrarSettings,
  testQueryGovClaims,
  testQueryGovConfig,
  testQueryGovPoll,
  testQueryGovPolls,
  testQueryGovStaker,
  testQueryGovState,
  testQueryGovVoters,
  VoteOption,
  testGovHodlerUpdateConfig,
  testGovResetClaims,
  testTransferStake,
} from "./halo/gov";
import {
  testStakingUnbond,
  testStakingWithdraw,
  testQueryStakingConfig,
  testQueryStakingStakerInfo,
  testQueryStakingState,
} from "./halo/staking";
import {
  testVestingUpdateConfig,
  testVestingRegisterVestingAccounts,
  testAddSchedulesToVestingAccount,
  testUserClaimsVestedTokens,
  testQueryVestingConfig,
  testQueryVestingAccount,
  testQueryVestingAccounts,
} from "./halo/vesting";
import {
  testInstantiateSubDao,
  testInstantiateSubDaoToken,
  testInstantiateDonationMatchContract,
} from "./core/subdao";
import {
  testSendDepositToGiftcards,
  testClaimGiftcardsDeposit,
  testSpendGiftcardsBalance,
  testQueryGiftcardsBalance,
  testQueryGiftcardsConfig,
  testQueryGiftcardsDeposit,
} from "./core/accessories";
import { localjuno } from "../utils/config/localjunoConstants";
import { localibc } from "../utils/config/localIbcConstants";

export async function testExecute(
  config: any, // environment config object
  apTeam: DirectSecp256k1HdWallet,
  apTeam2: DirectSecp256k1HdWallet,
  apTeam3: DirectSecp256k1HdWallet,
  charity1: DirectSecp256k1HdWallet,
  charity2: DirectSecp256k1HdWallet,
  ast1: DirectSecp256k1HdWallet,
  ast2: DirectSecp256k1HdWallet,
  pleb: DirectSecp256k1HdWallet,
  tca: DirectSecp256k1HdWallet,
  apTeamAddr: string,
  apTeam2Addr: string,
  apTeam3Addr: string,
  apTreasuryAddr: string,
  charity1Addr: string,
  charity2Addr: string,
  ast1Addr: string,
  ast2Addr: string,
  plebAddr: string,
  tcaAddr: string,
  registrar: string,
  indexFund: string,
  vaultLocked1: string,
  vaultLiquid1: string,
  vaultLocked2: string,
  vaultLiquid2: string,
  accounts: string,
  settingsController: string,
  donationMatching: string,
  endowId1: number,
  endowId2: number,
  endowId3: number,
  cw4GrpApTeam: string,
  cw3ApTeam: string,
  cw4GrpReviewTeam: string,
  cw3ReviewTeam: string,
  loopswapFactory: string,
  loopswapFarming: string,
  loopswapLoopJunoPair: string,
  loopswapLoopToken: string,
  haloAirdrop: string,
  haloCollector: string,
  haloCommunity: string,
  haloDistributor: string,
  haloGov: string,
  haloStaking: string,
  haloVesting: string,
  giftcards: string
): Promise<void> {
  console.log(
    chalk.yellow("\nStep 2. Setting up signing clients for all possible actors")
  );
  const networkInfo = config.networkInfo;
  const actors = {
    apTeam: {
      client: await clientSetup(apTeam, networkInfo),
      wallet: apTeam,
      addr: apTeamAddr,
    },
    apTeam2: {
      client: await clientSetup(apTeam2, networkInfo),
      wallet: apTeam2,
      addr: apTeam2Addr,
    },
    apTeam3: {
      client: await clientSetup(apTeam3, networkInfo),
      wallet: apTeam3,
      addr: apTeam3Addr,
    },
    charity1: {
      client: await clientSetup(charity1, networkInfo),
      wallet: charity1,
      addr: charity1Addr,
    },
    charity2: {
      client: await clientSetup(charity2, networkInfo),
      wallet: charity2,
      addr: charity2Addr,
    },
    ast1: {
      client: await clientSetup(ast1, networkInfo),
      wallet: ast1,
      addr: ast1Addr,
    },
    ast2: {
      client: await clientSetup(ast2, networkInfo),
      wallet: ast2,
      addr: ast2Addr,
    },
    pleb: {
      client: await clientSetup(pleb, networkInfo),
      wallet: pleb,
      addr: plebAddr,
    },
    tca: {
      client: await clientSetup(tca, networkInfo),
      wallet: tca,
      addr: tcaAddr,
    },
  };
  console.log(chalk.green(" Done!"));

  console.log(chalk.yellow("\nStep 3. Running Tests"));

  // Test execute

  /* --- SUBDAO TESTS --- */
  // await testInstantiateSubDao(actors.apTeam.client, actors.apTeam.addr, 165, registrar);
  // await testInstantiateSubDaoToken(actors.apTeam.client, actors.apTeam.addr, 166, registrar);
  // await testInstantiateDonationMatchContract(actors.apTeam.client, actors.apTeam.addr, 167, registrar);

  /* --- MULTISIG contracts --- */
  // await testAddMemberToC4Group(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, cw4GrpApTeam, actors.apTeam2.addr);
  // await testUpdateCw3Config(actors.apTeam.client, actors.apTeam.addr, cw3ReviewTeam, {
  //   threshold: { absolute_percentage: { percentage: "0.3" } },
  //   max_voting_period: { height: 25000 },
  //   require_execution: false,
  //   seed_split_to_liquid: "0",
  //   seed_asset: {
  //     info: { native: "ujunox" },
  //     amount: "100000",
  //   },
  //   new_endow_gas_money: { denom: "ujunox", amount: "100000" },
  // });
  // await testUpdateCw3Config(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, {
  //   threshold: { absolute_percentage: { percentage: "0.3" } },
  //   max_voting_period: { height: 25000 },
  //   require_execution: false,
  // });
  // await testProposalApprovingEndowment(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, accounts, 1);
  // await testQueryMultisigVoters(actors.apTeam.client, cw3ReviewTeam);
  // await testQueryProposal(actors.apTeam.client, cw3ReviewTeam, 1);
  // await testCw3CastVote(actors.apTeam.client, actors.apTeam.addr, cw3ReviewTeam, 2, `yes`);
  // await testCw3CastApplicationVote(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ReviewTeam,
  //   1,
  //   `yes`
  // );
  // await testCw3ExecutePoll(actors.apTeam.client, actors.apTeam.addr, cw3ReviewTeam, 1);
  // await testQueryListProposals(actors.apTeam.client, cw3ApTeam);
  // await testQueryMultisigThreshold(actors.apTeam.client, cw3ReviewTeam);
  // await testQueryGroupMembersList(actors.apTeam.client, cw4GrpApTeam);
  // await testQueryMultisigConfig(actors.apTeam.client, cw3ReviewTeam);
  // await testQueryApplicationsCw3Balances(actors.apTeam.client, cw3ReviewTeam);

  /* --- GIFTCARD contract --- */
  // await testSendDepositToGiftcards(actors.apTeam.client, actors.apTeam.addr, giftcards, { denom: "ujunox", amount: "4206900" });
  // await testClaimGiftcardsDeposit(actors.apTeam.client, actors.apTeam.addr, giftcards, 1, actors.apTeam2.addr);
  // await testSpendGiftcardsBalance(actors.apTeam2.client, actors.apTeam2.addr, giftcards, "ujunox", "100000", 22, "0", "1");
  // await testQueryGiftcardsConfig(actors.apTeam.client, giftcards);
  // await testQueryGiftcardsBalance(actors.apTeam.client, giftcards, actors.apTeam2.addr);
  // await testQueryGiftcardsDeposit(actors.apTeam.client, giftcards, 1);

  /* --- INDEXFUND contract --- */
  // await testUpdatingIndexFundOwner(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund, cw3ApTeam);
  // await testUpdatingIndexFundRegistrar(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund, registrar);
  // await testUpdatingIndexFundConfigs(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund);
  // await testUpdateAllianceMembersList(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   actors.apTeam2.addr, // address #1
  //   {
  //     name: "Testnet Charity #2",
  //     website:
  //       "http://angelprotocol.io/app/charity/juno1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf",
  //     logo: "https://angelprotocol.io/favicon.ico",
  //   }, // member #1`
  //   "add" // action
  // );
  // await testUpdateAllianceMember(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   actors.apTeam2.addr, // member address
  //   {
  //     name: "Testnet Charity #2",
  //     website:
  //       "http://angelprotocol.io/app/charity/juno1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf",
  //     logo: "https://angelprotocol.io/favicon.ico",
  //   },
  // );
  // await testCreateIndexFund(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   "Test Index Fund Name",
  //   "Test Index Fund desc",
  //   false,
  //   []
  // );
  // await testRemoveIndexFund(actors.apTeam.client, actors.apTeam.addr, indexFund, 1);
  // await testIndexFundRemoveMember(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund, 1); // INCOMPLETE: "remove_member" entry should be called from "accounts_contract".
  // await testDonorSendsToIndexFund(actors.pleb.client, actors.pleb.addr, indexFund, 1, "0", "1000000");
  // await testTcaMemberSendsToIndexFund(actors.tca.client, actors.tca.addr, indexFund);
  // await testIndexFundUpdateAllianceMember(actors.apTeam.client, actors.apTeam.addr, indexFund, ast1Addr, { name: "Charity3", logo: undefined, website: undefined });
  // await testUpdateFundMembers(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   indexFund,
  //   2,
  //   [1, 2],
  //   []
  // );

  /* --- REGISTRAR contract --- */
  // await testUpdatingRegistrarUpdateOwner(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, registrar, cw3ApTeam);
  // await testUpdatingRegistrarConfigs(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, registrar, {
  //   accepted_tokens_native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', localjuno.denoms.usdc, localjuno.denoms.usdt, config.networkInfo.nativeToken],
  //   accepted_tokens_cw20: [],
  //   // cw3_code: 102,
  //   // cw4_code: 104,
  //   // accounts_code_id: 102,
  // });

  // await testUpdatingRegistrarNetworkConnections(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   registrar,
  //   {
  //     name: "Juno mainnet",
  //     chain_id: "juno-1",
  //     ibc_channel: undefined,
  //     ica_address: undefined,
  //     gas_limit: undefined,
  //   },
  //   "post", // action: "post" or "delete"
  // );

  // await testUpdateFees(actors.apTeam.client, actors.apTeam.addr, registrar, {
  //   fees: [
  //     ["endowtype_charity", "0.05"],
  //     ["endowtype_normal", "0.05"]
  //   ],
  // });

  /* --- ACCOUNTS & ENDOWMENTS --- */
  // let endowments_batch = [
  //   { "id": 1, "tier": 2 },
  //   { "id": 2, "tier": 2 },
  // ];
  // await testCreateEndowmentCw3s(actors.apTeam.client, actors.apTeam.addr, registrar, accounts, endowments_batch);
  // await testApTeamChangesEndowmentSettings(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, accounts, endowments_batch);
  // await testCreateEndowment(networkInfo, actors.apTeam.wallet, cw3ReviewTeam, accounts, {
  //   owner: actors.apTeam.addr,
  //   maturity_time: undefined,
  //   name: "Test Normal Endowment",
  //   categories: { sdgs: [2,11], general: [] },
  //   tier: 2,
  //   endow_type: "charity",
  //   logo: "logo 1",
  //   image: "logo 1",
  //   cw4_members: [{ addr: actors.apTeam.addr, weight: 1 }],
  //   kyc_donors_only: true,
  //   cw3_threshold: { absolute_percentage: { percentage: "0.5" } },
  //   cw3_max_voting_period: 604800,
  //   beneficiaries_allowlist: [],
  //   contributors_allowlist: [],
  //   split_max: "1",
  //   split_min: "0",
  //   split_default: "0.5",
  //   ignore_user_splits: false,
  // },
  // [actors.apTeam.wallet]);
  // await testCreateNormalEndowment(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   {
  //     owner: actors.ast1.addr,
  //     name: "Test-Suite Normal Endowment",
  //     categories: { sdgs: [], general: [] },
  //     tier: 0,
  //     endow_type: "normal",
  //     logo: "test logo",
  //     image: "test image",
  //     kyc_donors_only: false,
  //     cw4_members: [
  //       { addr: actors.ast1.addr, weight: 1 },
  //       { addr: actors.ast2.addr, weight: 1 },
  //     ],
  //     cw3_threshold: { absolute_percentage: { percentage: "0.5" } },
  //     cw3_max_voting_period: 10000,
  //     beneficiaries_allowlist: [charity1Addr],
  //     contributors_allowlist: [charity2Addr],
  //     split_max: "0.8",
  //     split_min: "0.0",
  //     split_default: "0.5",
  //     earnings_fee: undefined,
  //     deposit_fee: undefined,
  //     withdraw_fee: undefined,
  //     aum_fee: undefined,
  //     maturity_time: undefined,
  //     dao: undefined,
  //     proposal_link: undefined,
  //     endowment_controller: undefined,
  //     parent: undefined,
  //     split_to_liquid: undefined,
  //     ignore_user_splits: false,
  //   }
  // );
  // await actors.apTeam.client.sendTokens(actors.apTeam.addr, cw3ReviewTeam, [{ denom: "ujunox", amount: "10000000" }], 10000, "initial dust & seed funds");
  // await testSendDonationToEndowment(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   { denom: config.networkInfo.nativeToken, amount: "100000" }
  //   // { denom: localjuno.denoms.usdc, amount: "1000000" }
  // );
  // await testEndowmentVaultsRedeem(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   `locked`,
  //   [[vaultLocked1, "500000"]],  // Vec<(vault, amount)>
  // );
  // await testVaultHarvest(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   vaultLocked1,
  // );
  // await testEndowmentCanWithdrawLiquid(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   actors.charity1.addr,
  //   [{ info: { native: config.networkInfo.nativeToken }, amount: "1000" }],
  // );
  // await testCharityCanWithdrawLocked(
  //   networkInfo,
  //   actors.charity1.wallet,
  //   accounts,
  //   cw3ApTeam,
  //   endowId1,
  //   [{ info: { native: config.networkInfo.nativeToken }, amount: "2000" }],
  //   [],
  //   [actors.apTeam.wallet],
  // );
  // await testApproveInactiveEndowment(actors.apTeam.client, actors.apTeam.addr, accounts, endowId1);
  // await testUpdateEndowmentStatus(actors.apTeam.client, actors.apTeam.addr, accounts, { endowment_id: 3, status: 3, beneficiary: { wallet: { address: actors.apTeam.addr } } });
  // await testUpdateEndowmentStatus(actors.apTeam.client, actors.apTeam.addr, accounts, { endowment_id: 4, status: 3, beneficiary: { endowment: { id: 1 } } });
  // await testRejectUnapprovedDonations(actors.pleb.client, actors.pleb.addr, accounts, endowId2, "10000000"); // possible query registrar error

  /* --- Settings-Controller --- */
  // await testUpdateSettingsControllerConfig(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   settingsController,
  //   {
  //     owner: actors.apTeam.addr,
  //     registrar_contract: registrar,
  //   }
  // );

  // await testSetupDao(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   settingsController,
  //   1,  // endowment ID
  //   {
  //     quorum: "0.5",
  //     threshold: "0.50",
  //     voting_period: 3600,
  //     timelock_period: 300,
  //     expiration_period: 300,
  //     proposal_deposit: "1000", // Uint128
  //     snapshot_period: 300,
  //     token: {
  //       bonding_curve: {
  //         curve_type: {
  //           constant: {
  //             value: "1000000",
  //             scale: 1
  //           }
  //         },
  //         name: "DaoToken",
  //         symbol: "DAOTOKEN",
  //         decimals: 6,
  //         reserve_denom: "ujuno",
  //         reserve_decimals: 6,
  //         unbonding_period: 1000000,
  //       }
  //     },
  //   }
  // );

  // await testSetupDonationMatch(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   settingsController,
  //   1,  // endowment ID
  //   {
  //     halo_token_reserve: {},
  //   }
  // );

  // await testUpdateDelegate(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   settingsController,
  //   {
  //     id: 1, // endowment ID
  //     setting: "maturity_time",
  //     action: "set", // "set" or "revoke"
  //     delegate_address: actors.apTeam.addr,
  //     delegate_expiry: undefined,
  //   }
  // );

  /* --- LOOP VAULT(s) --- */
  // await testVaultUpdateConfig(actors.apTeam.client, apTeamAddr, vaultLocked1, {
  //   sibling_vault: undefined,
  //   lp_staking_contract: undefined,
  //   lp_pair_contract: undefined,
  //   keeper: undefined,
  //   tax_collector: undefined,

  //   native_token: undefined,
  //   reward_to_native_route: [
  //     {
  //       loop: {
  //         offer_asset_info: {
  //           cw20: localjuno.loopswap.loop_token_contract,
  //         },
  //         ask_asset_info: {
  //           native: localjuno.networkInfo.nativeToken,
  //         }
  //       }
  //     }
  //   ],
  //   native_to_lp0_route: undefined,
  //   native_to_lp1_route: undefined,
  // });

  // Test query
  // await testQueryRegistrarConfig(actors.apTeam.client, registrar);
  // await testQueryRegistrarStrategy(actors.apTeam.client, registrar, vaultLocked1);
  // await testQueryRegistrarNetworkConnection(
  //   actors.apTeam.client,
  //   registrar,
  //   networkInfo.chainId
  // );

  // await testQueryAccountsConfig(actors.apTeam.client, accounts);
  // await testQueryAccountsEndowment(actors.apTeam.client, accounts, endowId1);
  // await testQueryAccountsState(actors.apTeam.client, accounts, endowId1);
  // await testQueryAccountsEndowmentByProposalLink(actors.apTeam.client, accounts, 4); // proposal_link: number

  // await testQuerySettingsControllerConfig(
  //   actors.apTeam.client,
  //   settingsController
  // );
  // await testQuerySettingsControllerEndowSettings(
  //   actors.apTeam.client,
  //   settingsController,
  //   endowId1
  // );
  // await testQuerySettingsControllerEndowController(
  //   actors.apTeam.client,
  //   settingsController,
  //   endowId1
  // );
  // await testQuerySettingsControllerEndowPermissions(actors.apTeam.client, settingsController, endowId1, actors.apTeam.addr);

  // await testQueryIndexFundConfig(actors.apTeam.client, indexFund);
  // await testQueryIndexFundState(actors.apTeam.client, indexFund);
  // await testQueryIndexFundFundDetails(actors.apTeam.client, indexFund, 1);
  // await testQueryIndexFundActiveFundDonations(actors.apTeam.client, indexFund);
  // await testQueryIndexFundDeposit(actors.apTeam.client, indexFund);

  // await testQueryVaultConfig(actors.apTeam.client, vaultLocked1);
  // await testQueryVaultTotalBalance(actors.apTeam.client, vaultLocked1);
  // await testQueryVaultTokenInfo(actors.apTeam.client, vaultLocked1);

  // Test query for HALO airdrop
  // await testAirdropUpdateConfig(actors.apTeam.client, actors.apTeam.addr, apTeam2, pleb, haloAirdrop);
  // await testAirdropRegisterNewMerkleRoot(actors.apTeam.client, actors.apTeam.addr, haloAirdrop);
  // await testAirdropClaim(actors.apTeam.client, actors.apTeam.addr, haloAirdrop);
  // await testQueryAirdropConfig(actors.apTeam.client, haloAirdrop);
  // await testQueryAirdropMerkleRoot(actors.apTeam.client, haloAirdrop, 1);
  // await testQueryAirdropIsClaimed(actors.apTeam.client, haloAirdrop, 1, "juno1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8");
  // await testQueryAirdropLatestStage(actors.apTeam.client, haloAirdrop);

  // Test query for HALO collector
  // await testCollectorUpdateConfig(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloCollector,
  //   "1.0",
  //   haloGov,
  //   undefined
  // );
  // await testCollectorSweep(actors.apTeam.client, actors.apTeam.addr, haloCollector);
  // await testQueryCollectorConfig(actors.apTeam.client, haloCollector);
  // await testQueryCollectorPair(actors.apTeam.client, haloCollector);

  // Test query for HALO community
  // await testCommunityUpdateConfig(actors.apTeam.client, actors.apTeam.addr, pleb, haloGov, haloCommunity, "1000000", undefined);
  // await testCommunitySpend(actors.apTeam.client, actors.apTeam.addr, haloGov, haloCommunity, "addr000", "1000000");
  // await testQueryCommunityConfig(actors.apTeam.client, haloCommunity);

  // Test query for HALO distributor
  // await testDistributorUpdateConfig(actors.apTeam.client, actors.apTeam.addr, haloDistributor, "1000000", haloGov);
  // await testDistributorSpend(actors.apTeam.client, actors.apTeam.addr, haloDistributor, "addr000", "1000000");
  // await testDistributorAdd(actors.apTeam.client, actors.apTeam.addr, haloGov, haloDistributor, apTeam2Addr);
  // await testDistributorRemove(actors.apTeam.client, actors.apTeam.addr, haloGov, haloDistributor, apTeam2Addr);
  // await testQueryDistributorConfig(actors.apTeam.client, haloDistributor);

  // Tests for HALO vesting
  // await testVestingUpdateConfig(actors.apTeam.client, actors.apTeam.addr, haloVesting, undefined, undefined, undefined);
  // await testVestingRegisterVestingAccounts(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloVesting,
  //   [
  //     {address: apTeam3Addr, schedules: [[100, 101, "100"], [100, 110, "100"], [100, 200, "100"]]},
  //     {address: apTeam2Addr, schedules: [[100, 110, "100"]]},
  //   ]
  // );
  // let new_schedules = [[1000, 2000, "100"]];
  // await testVestingUpdateVestingAccount(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloVesting,
  //   apTeam3Addr,
  //   new_schedules,
  // );
  // await testUserClaimsVestedTokens(actors.apTeam3.client, actors.apTeam3.addr, haloVesting);
  // await testQueryVestingConfig(actors.apTeam.client, haloVesting);
  // await testQueryVestingAccount(actors.apTeam.client, haloVesting, "addr0");
  // await testQueryVestingAccounts(actors.apTeam.client, haloVesting, undefined, undefined);

  // await testGovUpdateConfig(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloGov,
  //   undefined,
  //   15, // quorum
  //   50, // threshold
  //   100800, // voting_period
  //   undefined,
  //   "5000000000", // deposit
  //   100800, // snapshot period
  //   undefined, // unbonding period
  //   undefined // gov_hodler
  // );
  // await testGovExecutePoll(actors.apTeam.client, actors.apTeam.addr, haloGov, 1);
  // await testGovEndPoll(actors.apTeam.client, actors.apTeam.addr, haloGov, 1);
  // await testGovSnapshotPoll(actors.apTeam.client, actors.apTeam.addr, haloGov, 1);
  // await testGovStakeVotingTokens(actors.apTeam.client, actors.apTeam.addr, junoswapToken, haloGov, "20000000000");
  // await testGovStakeVotingTokens(actors.apTeam2.client, actors.apTeam2.addr, junoswapToken, haloGov, "10000000000");
  // await testGovStakeVotingTokens(actors.apTeam3.client, actors.apTeam3.addr, junoswapToken, haloGov, "5000000000");
  // await testGovWithdrawVotingTokens(actors.apTeam.client, actors.apTeam.addr, haloGov, "1000000000");
  // await testGovWithdrawVotingTokens(actors.apTeam2.client, actors.apTeam2.addr, haloGov, "10000000000");
  // await testGovWithdrawVotingTokens(actors.apTeam3.client, actors.apTeam3.addr, haloGov, "10000000000");
  // await testGovClaimVotingTokens(actors.apTeam.client, actors.apTeam.addr, haloGov);
  // await testGovCastVote(actors.apTeam.client, actors.apTeam.addr, haloGov, 1, VoteOption.YES, "1");
  // await testGovRegisterContracts(actors.apTeam.client, actors.apTeam.addr, haloGov, junoswapToken);
  // await testGovExecutePollForRegistrarSettings(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloGov,
  //   junoswapToken,
  //   "1000000",
  //   100,
  //   "0.5",
  //   "0.1"
  // );
  // await testGovResetClaims(actors.apTeam.client, actors.apTeam.addr, haloGov, [
  //   apTeamAddr,
  //   apTeam2Addr,
  //   apTeam3Addr,
  // ]);
  // await testQueryGovConfig(actors.apTeam.client, haloGov);
  // await testQueryGovState(actors.apTeam.client, haloGov);
  // await testQueryGovClaims(actors.apTeam.client, haloGov, apTeamAddr);
  // await testQueryGovStaker(actors.apTeam.client, haloGov, apTeamAddr);
  // await testQueryGovStaker(actors.apTeam.client, haloGov, apTeam2Addr);
  // await testQueryGovStaker(actors.apTeam.client, haloGov, apTeam3Addr);
  // await testQueryGovPoll(actors.apTeam.client, haloGov, 1);
  // await testQueryGovPolls(actors.apTeam.client, haloGov, undefined, undefined, undefined);

  // await testQueryGovVoters(actors.apTeam.client, haloGov, 1, undefined, undefined);

  // Test query for HALO staking
  // await testStakingUnbond(actors.apTeam.client, actors.apTeam.addr, haloStaking, "100");
  // await testStakingWithdraw(actors.apTeam.client, actors.apTeam.addr, haloStaking);
  // await testQueryStakingConfig(actors.apTeam.client, haloStaking);
  // await testQueryStakingStakerInfo(actors.apTeam.client, haloStaking, "addr000", undefined);
  // await testQueryStakingState(actors.apTeam.client, haloStaking);
}
