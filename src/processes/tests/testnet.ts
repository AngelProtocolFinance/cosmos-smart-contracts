import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { datetimeStringToUTC, clientSetup } from "../../utils/helpers";
import {
  testEndowmentCanWithdrawLiquid,
  testCharityCanWithdrawLocked,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testApTeamChangesAccountsEndowmentOwner,
  testSendDonationToEndowment,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testQueryAccountsProfile,
  testQueryAccountsState,
  testQueryAccountsTransactions,
  testApproveInactiveEndowment,
  testUpdateEndowmentStatus,
  testCreateEndowment,
  testQueryAccountsEndowmentList,
  testEndowmentVaultsRedeem,
  testQueryAccountsTokenAmount,
  testSendRestitutionFundsToEndowments,
} from "./core/accounts";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
  testUpdateAllianceMembersList,
  testUpdatingIndexFundConfigs,
  testCreateIndexFund,
  testRemoveIndexFund,
  // testQueryIndexFundActiveFundDetails,
  testQueryIndexFundActiveFundDonations,
  testQueryIndexFundConfig,
  testQueryIndexFundDeposit,
  testQueryIndexFundFundDetails,
  testQueryIndexFundFundsList,
  testQueryIndexFundState,
  testQueryIndexFundTcaList,
  testQueryIndexFundAllianceMember,
  testUpdatingIndexFundOwner,
  testUpdatingIndexFundRegistrar,
  testIndexFundRemoveMember,
  testUpdateAllianceMember,
} from "./core/indexFunds";
import {
  testUpdateCw3Config,
  testAddMemberToC4Group,
  testProposalApprovingEndowment,
  testCw3CastVote,
  testCw3ExecutePoll,
  testQueryMultisigVoters,
  testQueryMultisigThreshold,
  testQueryGroupMembersList,
} from "./core/multisig";
import {
  testUpdatingRegistrarConfigs,
  testQueryRegistrarVaultList,
  testQueryRegistrarConfig,
  testQueryRegistrarVault,
  testQueryRegistrarNetworkConnection,
  testUpdatingRegistrarNetworkConnections,
  testUpdatingRegistrarUpdateOwner
} from "./core/registrar";
import { testQueryVaultConfig, testVaultHarvest, testVaultReinvestToLocked } from "./core/vaults";
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

export async function testExecute(
  config: any, // environment config object 
  apTeam: DirectSecp256k1HdWallet,
  apTeam2: DirectSecp256k1HdWallet,
  apTeam3: DirectSecp256k1HdWallet,
  charity1: DirectSecp256k1HdWallet,
  charity2: DirectSecp256k1HdWallet,
  charity3: DirectSecp256k1HdWallet,
  pleb: DirectSecp256k1HdWallet,
  tca: DirectSecp256k1HdWallet,
  apTeamAddr: string,
  apTeam2Addr: string,
  apTeam3Addr: string,
  apTreasuryAddr: string,
  charity1Addr: string,
  charity2Addr: string,
  charity3Addr: string,
  plebAddr: string,
  tcaAddr: string,
  registrar: string,
  indexFund: string,
  vaultLocked1: string,
  vaultLiquid1: string,
  vaultLocked2: string,
  vaultLiquid2: string,
  accounts: string,
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
): Promise<void> {
  console.log(chalk.yellow("\nStep 2. Setting up signing clients for all possible actors"));
  const networkInfo = config.networkInfo;
  const actors = {
    apTeam: { client: await clientSetup(apTeam, networkInfo), wallet: apTeam, addr: apTeamAddr },
    apTeam2: { client: await clientSetup(apTeam2, networkInfo), wallet: apTeam2, addr: apTeam2Addr },
    apTeam3: { client: await clientSetup(apTeam3, networkInfo), wallet: apTeam3, addr: apTeam3Addr },
    charity1: { client: await clientSetup(charity1, networkInfo), wallet: charity1, addr: charity1Addr },
    charity2: { client: await clientSetup(charity2, networkInfo), wallet: charity2, addr: charity2Addr },
    charity3: { client: await clientSetup(charity3, networkInfo), wallet: charity3, addr: charity3Addr },
    pleb: { client: await clientSetup(pleb, networkInfo), wallet: pleb, addr: plebAddr },
    tca: { client: await clientSetup(tca, networkInfo), wallet: tca, addr: tcaAddr },
  };
  console.log(chalk.green(" Done!"));

  console.log(chalk.yellow("\nStep 3. Running Tests"));

  // Test execute

  // Multisig test
  // await testAddMemberToC4Group(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, cw4GrpApTeam, actors.apTeam2.addr);
  // await testUpdateCw3Config(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, 50, 25000);
  // await testProposalApprovingEndowment(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, registrar, 1);
  // await testCw3CastVote(actors.apTeam2.client, actors.apTeam2.addr, cw3ApTeam, 7, `yes`);
  // await testCw3ExecutePoll(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, 3);
  // await testQueryMultisigVoters(actors.apTeam.client, cw3ApTeam);
  // await testQueryMultisigThreshold(actors.apTeam.client, cw3ApTeam);
  // await testQueryGroupMembersList(actors.apTeam.client, cw4GrpApTeam);

  /* --- REGISTRAR contract --- */
  // await testUpdatingRegistrarUpdateOwner(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, registrar, cw3ApTeam);
  // await testUpdatingRegistrarConfigs(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, registrar, {
  //   accepted_tokens_native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', config.networkInfo.nativeToken],
  //   accepted_tokens_cw20: [],
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
  //   // "juno178u9lz89f54njqz6nentst3m9nye2cc7ezssmq", // address #2
  //   // { name: "Testnet Admin", webiste: "http://angelprotocol.io", logo: "" }, // member #2
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
  // await testRemoveIndexFund(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   7,
  // );
  // await testIndexFundRemoveMember(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund, 1); // INCOMPLETE: "remove_member" entry should be called from "accounts_contract".
  // await testDonorSendsToIndexFund(actors.pleb.client, actors.pleb.addr, indexFund, 1, "0", "1000000");
  // await testTcaMemberSendsToIndexFund(actors.tca.client, actors.tca.addr, indexFund);
  // await testUpdateFundMembers(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   2,
  //   [1, 2],
  //   []
  // );

  /* --- ACCOUNTS & ENDOWMENTS --- */
  // let endowments_batch = [
  //   { "id": 6, "acct_type": "liquid", "amount": "114000" },
  //   { "id": 6, "acct_type": "locked", "amount": "128470" },
  //   { "id": 2, "acct_type": "liquid", "amount": "600000" },
  //   { "id": 2, "acct_type": "locked", "amount": "1003000" },
  // ];
  // await testSendRestitutionFundsToEndowments(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   accounts,
  //   endowments_batch,
  //   "ujunox"
  // );
  // await testCreateEndowment(networkUrl, actors.charity1.wallet, cw3ReviewTeam, accounts, {
  //   owner: actors.charity1.addr,
  //   withdraw_before_maturity: false,
  //   maturity_time: undefined,
  //   maturity_height: undefined,
  //   profile: {
  //     name: "Test-Suite Endowment",
  //     overview: "Endowment created from the test-suite integration test",
  //     categories: { sdgs: [2], general: [] },
  //     tier: 3,
  //     logo: "test logo",
  //     image: "test image",
  //     url: undefined,
  //     registration_number: undefined,
  //     country_city_origin: undefined,
  //     contact_email: undefined,
  //     social_media_urls: {
  //       facebook: undefined,
  //       twitter: undefined,
  //       linkedin: undefined,
  //     },
  //     number_of_employees: undefined,
  //     average_annual_budget: undefined,
  //     annual_revenue: undefined,
  //     charity_navigator_rating: undefined,
  //     endow_type: "Charity",
  //   },
  //   cw4_members: [{ addr: actors.charity1.addr, weight: 1 }],
  //   kyc_donors_only: false,
  //   cw3_threshold: { absolute_percentage: { percentage: "0.5" } },
  //   cw3_max_voting_period: 10000,
  // }, [actors.apTeam.wallet]);
  // await testCharityCanUpdateStrategies(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   `locked`,
  //   [{ vault: vaultLocked1, percentage: "0.5" }]
  // );
  // await testCharityCanUpdateStrategies(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   `liquid`,
  //   [{ vault: vaultLiquid1, percentage: "0.5" }]
  // );

  // await testSendDonationToEndowment(actors.apTeam.client, actors.apTeam.addr, accounts, 32, { denom: config.networkInfo.nativeToken, amount: "100000" });
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
  // );
  // await testVaultReinvestToLocked(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   "1000000",
  //   vaultLiquid1,
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
  //   networkUrl,
  //   actors.charity1.wallet,
  //   accounts,
  //   cw3ApTeam,
  //   endowId1,
  //   [{ info: { native: config.networkInfo.nativeToken }, amount: "2000" }],
  //   [],
  //   [actors.apTeam.wallet],
  // );
  // await testApproveInactiveEndowment(actors.apTeam.client, actors.apTeam.addr, accounts, endowId1);
  // await testUpdateEndowmentStatus(actors.apTeam.client, actors.apTeam.addr, accounts, { endowment_id: 10, status: 3, beneficiary: { wallet: { address: actors.apTeam.addr }}});
  // await testRejectUnapprovedDonations(actors.pleb.client, actors.pleb.addr, accounts, endowId3, "10000000"); // possible query registrar error


  // Test query
  // await testQueryRegistrarConfig(actors.apTeam.client, registrar);
  // await testQueryRegistrarVaultList(actors.apTeam.client, registrar);
  // await testQueryRegistrarVault(actors.apTeam.client, registrar, vaultLocked1);
  // await testQueryRegistrarNetworkConnection(actors.apTeam.client, registrar, "juno-1");

  // await testQueryAccountsEndowmentList(actors.apTeam.client, accounts);
  // await testQueryAccountsBalance(actors.apTeam.client, accounts, endowId1);
  // await testQueryAccountsConfig(actors.apTeam.client, accounts);
  // await testQueryAccountsEndowment(actors.apTeam.client, accounts, endowId1);
  // await testQueryAccountsProfile(actors.apTeam.client, accounts, endowId1);
  // await testQueryAccountsState(actors.apTeam.client, accounts, endowId1);
  // await testQueryAccountsTokenAmount(actors.apTeam.client, accounts, 1, { native: config.networkInfo.nativeToken }, "locked");

  // await testQueryIndexFundConfig(actors.apTeam.client, indexFund);
  // await testQueryIndexFundState(actors.apTeam.client, indexFund);
  // await testQueryIndexFundTcaList(actors.apTeam.client, indexFund);
  // await testQueryIndexFundFundsList(actors.apTeam.client, indexFund, undefined, undefined);
  // await testQueryIndexFundFundDetails(actors.apTeam.client, indexFund, 1);
  // await testQueryIndexFundActiveFundDonations(actors.apTeam.client, indexFund);
  // await testQueryIndexFundDeposit(actors.apTeam.client, indexFund);
  // await testQueryIndexFundAllianceMember(actors.apTeam.client, indexFund, actors.apTeam2.addr);

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

  // Test query for LBP Factory
  // await testFactoryUpdateConfig(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   lbpFactoryContract,
  //   undefined,
  //   undefined,
  //   undefined,
  //   undefined,
  //   haloCollector,
  //   undefined
  // );
  // await testFactoryCreatePair(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   lbpFactoryContract,
  //   junoswapToken,
  //   "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4",
  //   datetimeStringToUTC("12/16/2021 00:00:00Z"),
  //   datetimeStringToUTC("12/17/2021 00:00:00Z"),
  //   "96",
  //   "50",
  //   "4",
  //   "50",
  //   "HALO <-> axlUSDC Pair"
  // );
  // await getPairContractLpToken(actors.apTeam.client, lbpPairContract);
  // await testFactoryUnregister(actors.apTeam.client, actors.apTeam.addr, lbpFactoryContract, junoswapToken, "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4");
  // await testQueryFactoryConfig(actors.apTeam.client, lbpFactoryContract);
  // await testQueryFactoryPair(actors.apTeam.client, lbpFactoryContract, junoswapToken);
  // await testQueryFactoryPairs(actors.apTeam.client, lbpFactoryContract);

  // await testPairSwapNativeToHalo(actors.apTeam.client, actors.apTeam.addr, lbpPairContract, "100000000");
  // await testPairSwapHaloToNative(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   lbpPairContract,
  //   junoswapToken,
  //   "100000000"
  // );
  // await testQueryPairPair(actors.apTeam.client, lbpPairContract);
  // await testQueryPairPool(actors.apTeam.client, lbpPairContract);
  // await testQueryPairSimulationNativeToHalo(actors.apTeam.client, lbpPairContract);
  // await testQueryPairSimulationHaloToNative(actors.apTeam.client, lbpPairContract, junoswapToken);
  // await testQueryPairReverseSimulationNativeToHalo(actors.apTeam.client, lbpPairContract);
  // await testQueryPairReverseSimulationHaloToNative(
  //   actors.apTeam.client,
  //   lbpPairContract,
  //   junoswapToken
  // );

  // Test query for LBP Router
  // await testQueryRouterConfig(actors.apTeam.client, lbpRouterContract);

  // Test Loop Pair
  // await testPairProvideLiquidity(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   junoswapToken,
  //   "juno1yjg0tuhc6kzwz9jl8yqgxnf2ctwlfumnvscupp", // LOOP PAIR
  //   "80000000000000", //HALO
  //   "1300000000000", //axlUSDC
  // );

  // await testPairWithdrawLiquidity(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   lbpPairContract,
  //   lbpLpTokenContract,
  //   "100000000"
  // );

  // Test query for LBP Token
  // await testQueryTokenBalance(actors.apTeam.client, junoswapToken, apTeamAddr);
  // await testQueryTokenInfo(actors.apTeam.client, junoswapToken);
  // await testQueryTokenMinter(actors.apTeam.client, junoswapToken);

  // await testTransferTokenBalance(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   junoswapToken,
  //   apTeam2Addr,
  //   "420000000"
  // );
}
