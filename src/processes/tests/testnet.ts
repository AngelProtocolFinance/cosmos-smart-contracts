import { LcdClient,  Wallet } from "@cosmjs/launchpad";
import chalk from "chalk";
import { localjuno } from "../../config/localjunoConstants";
import { datetimeStringToUTC } from "../../utils/helpers";
import {
  testBeneficiaryCanWithdrawFromLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testApTeamChangesAccountsEndowmentOwner,
  testChangeManyAccountsEndowmentOwners,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testQueryAccountsProfile,
  testQueryAccountsState,
  testQueryAccountsTransactions,
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
} from "./core/indexFunds";
import {
  testUpdateCw3Config,
  testAddMemberToC4Group,
  testAddGuardiansToEndowment,
  testGuardiansChangeEndowmentOwner,
  testQueryMultisigVoters,
  testQueryMultisigThreshold,
  testQueryGroupMembersList,
} from "./core/multisig";
import {
  testUpdateEndowmentsStatus,
  testCreateEndowmentViaRegistrar,
  testAngelTeamCanTriggerVaultsHarvest,
  testMigrateAllAccounts,
  testUpdatingRegistrarConfigs,
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarApprovedVaultRateList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
  testQueryRegistrarEndowmentDetails,
  testQueryRegistrarVault,
  testQueryRegistrarVaultList,
  testUpdateEndowmentsEntry,
} from "./core/registrar";
import { testQueryVaultConfig } from "./core/vaults";
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
  juno: LcdClient,
  apTeam: Wallet,
  apTeam2: Wallet,
  apTeam3: Wallet,
  charity1: Wallet,
  charity2: Wallet,
  charity3: Wallet,
  pleb: Wallet,
  tca: Wallet,
  registrar: string,
  indexFund: string,
  Vault1: string,
  Vault2: string,
  endowmentContract1: string,
  endowmentContract2: string,
  endowmentContract3: string,
  endowmentContract4: string,
  cw4GrpApTeam: string,
  cw4GrpOwners: string,
  cw3ApTeam: string,
  cw3GuardianAngels: string,
  terraswapFactory: string,
  terraswapToken: string,
  terraswapPair: string,
  haloAirdrop: string,
  haloCollector: string,
  haloCommunity: string,
  haloDistributor: string,
  haloGov: string,
  haloStaking: string,
  haloVesting: string,
): Promise<void> {
  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testUpdatingIndexFundConfigs(juno, apTeam, indexFund);
  // await testUpdateAllianceMembersList(
  //   juno,
  //   apTeam,
  //   indexFund,
  //   "terra1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf", // address #1
  //   {
  //     name: "Testnet Charity #2",
  //     website:
  //       "http://angelprotocol.io/app/charity/terra1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf",
  //     logo: "https://angelprotocol.io/favicon.ico",
  //   }, // member #1`
  //   // "terra178u9lz89f54njqz6nentst3m9nye2cc7ezssmq", // address #2
  //   // { name: "Testnet Admin", webiste: "http://angelprotocol.io", logo: "" }, // member #2
  //   "add" // action
  // );
  // await testRemoveIndexFund(juno, apTeam, indexFund, 5);
  // await testCreateIndexFund(
  //   juno,
  //   apTeam,
  //   indexFund,
  //   "Test Index Fund Name",
  //   "Test Index Fund desc",
  //   false,
  //   []
  // );
  // await testUpdateFundMembers(juno, apTeam, indexFund, 2, [], []);
  // await testChangeManyAccountsEndowmentOwners(juno, apTeam, [
  //   {
  //     "address": "terra16zj5dw97sk7q3rvakzu76uyfv6zrxkvsln0yjz2wa5s58mq67vhs5wdv7l", // Current one is localjuno endow1.
  //     "owner": apTeam, 
  //     "kyc_donors_only": false,
  //   }
  // ]);

  // await testCreateEndowmentViaRegistrar(juno, apTeam, registrar, {
  //   owner: charity1.key.accAddress,
  //   beneficiary: charity1.key.accAddress,
  //   withdraw_before_maturity: false,
  //   maturity_time: undefined,
  //   maturity_height: undefined,
  //   guardians_multisig_addr: undefined,
  //   cw4_members: [{ addr: charity1.key.accAddress, weight: 1 }],
  //   kyc_donors_only: false,
  //   profile: {
  //     name: "Test-Suite Endowment",
  //     overview: "Endowment created from the test-suite integration test",
  //     un_sdg: 2,
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
  // });
  // Multisig test
  // await testAddMemberToC4Group(juno, apTeam, cw3ApTeam, cw4GrpApTeam, apTeam.key.accAddress);
  // await testUpdateCw3Config(juno, apTeam, cw3ApTeam, 50, 25000);

  // await testQueryMultisigVoters(juno, cw3ApTeam);
  // await testQueryMultisigThreshold(juno, cw3ApTeam);
  // await testQueryGroupMembersList(juno, cw4GrpApTeam);

  // Test execute
  // await testRejectUnapprovedDonations(juno, pleb, endowmentContract1, "10000000"); // possible query registrar error
  // await testDonorSendsToIndexFund(juno, pleb, indexFund, 1, "0.5", "4200000"); // possible query registrar error
  // await testTcaMemberSendsToIndexFund(juno, tca, indexFund); // possible query registrar error
  // await testAngelTeamCanTriggerVaultsHarvest(
  //   juno,
  //   apTeam,
  //   charity1,
  //   registrar,
  //   haloCollector,
  //   "0.5"
  // );  // vault-related
  // await testCharityCanUpdateStrategies(
  //   juno,
  //   charity1,
  //   endowmentContract1,
  //   Vault1,
  //   Vault2
  // );  // vault-related
  // await testBeneficiaryCanWithdrawFromLiquid(
  //   juno,
  //   charity3,
  //   endowmentContract3,
  //   Vault1,
  //   pleb.key.accAddress
  // );  // vault-related
  // await testUpdatingRegistrarConfigs(juno, apTeam, registrar, {
  //   cw3_code: 102,
  //   cw4_code: 104,
  //   accounts_code_id: 102,
  // });
  // await testApproveEndowments(juno, apTeam, registrar, endowmentContract1, 1);
  // await testClosingEndpoint(
  //   juno,
  //   apTeam,
  //   registrar,
  //   endowmentContract3,
  //   endowmentContract4
  // );
  // await testUpdateFundMembers(
  //   juno,
  //   apTeam,
  //   indexFund,
  //   2,
  //   [endowmentContract2],
  //   [endowmentContract4]
  // );
  // await testCreateIndexFund(
  //   juno,
  //   apTeam,
  //   indexFund,
  //   "Test fund for Ukraine Portal",
  //   "Another portal test fund",
  //   false,
  //   [endowmentContract2, endowmentContract3, endowmentContract4]
  // );
  // await testRemoveIndexFund(juno, apTeam, indexFund, 1);
  // Test query
  // await testQueryRegistrarConfig(juno, registrar);
  // await testQueryRegistrarEndowmentList(juno, registrar);
  // await testQueryRegistrarEndowmentDetails(juno, registrar, endowmentContract3);
  // await testQueryRegistrarApprovedVaultList(juno, registrar);
  // await testQueryRegistrarApprovedVaultRateList(juno, registrar);
  // await testQueryRegistrarVaultList(juno, registrar);
  // await testQueryRegistrarVault(juno, registrar, Vault1);
  // await testQueryVaultConfig(juno, Vault1);
  // await testQueryAccountsBalance(juno, endowmentContract4);
  // await testQueryAccountsConfig(juno, endowmentContract4);
  // await testQueryAccountsEndowment(juno, endowmentContract4);
  // await testQueryAccountsProfile(juno, endowmentContract4);
  // await testQueryAccountsState(juno, endowmentContract4);
  // await testQueryAccountsTransactions(
  //   juno,
  //   endowmentContract4,
  //   undefined,
  //   undefined,
  //   undefined
  // );
  // await testQueryIndexFundConfig(juno, indexFund);
  // await testQueryIndexFundState(juno, indexFund);
  // await testQueryIndexFundTcaList(juno, indexFund);
  // await testQueryIndexFundFundsList(juno, indexFund);
  // await testQueryIndexFundFundDetails(juno, indexFund, 4);
  // await testQueryIndexFundActiveFundDetails(juno, indexFund);
  // await testQueryIndexFundActiveFundDonations(juno, indexFund);
  // await testQueryIndexFundDeposit(juno, indexFund);

  // Test query for HALO airdrop
  // await testAirdropUpdateConfig(juno, apTeam, apTeam2, pleb, haloAirdrop);
  // await testAirdropRegisterNewMerkleRoot(juno, apTeam, haloAirdrop);
  // await testAirdropClaim(juno, apTeam, haloAirdrop);
  // await testQueryAirdropConfig(juno, haloAirdrop);
  // await testQueryAirdropMerkleRoot(juno, haloAirdrop, 1);
  // await testQueryAirdropIsClaimed(juno, haloAirdrop, 1, "terra1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8");
  // await testQueryAirdropLatestStage(juno, haloAirdrop);

  // Test query for HALO collector
  // await testCollectorUpdateConfig(
  //   juno,
  //   apTeam,
  //   haloCollector,
  //   "1.0",
  //   haloGov,
  //   undefined
  // );
  // await testCollectorSweep(juno, apTeam, haloCollector);
  // await testQueryCollectorConfig(juno, haloCollector);
  // await testQueryCollectorPair(juno, haloCollector);

  // Test query for HALO community
  // await testCommunityUpdateConfig(juno, apTeam, pleb, haloGov, haloCommunity, "1000000", undefined);
  // await testCommunitySpend(juno, apTeam, haloGov, haloCommunity, "addr000", "1000000");
  // await testQueryCommunityConfig(juno, haloCommunity);

  // Test query for HALO distributor
  // await testDistributorUpdateConfig(juno, apTeam, haloDistributor, "1000000", haloGov);
  // await testDistributorSpend(juno, apTeam, haloDistributor, "addr000", "1000000");
  // await testDistributorAdd(juno, apTeam, haloGov, haloDistributor, apTeam2.key.accAddress);
  // await testDistributorRemove(juno, apTeam, haloGov, haloDistributor, apTeam2.key.accAddress);
  // await testQueryDistributorConfig(juno, haloDistributor);

  // Tests for HALO vesting
  // await testVestingUpdateConfig(juno, apTeam, haloVesting, undefined, undefined, undefined);
  // await testVestingRegisterVestingAccounts(
  //   juno,
  //   apTeam,
  //   haloVesting,
  //   [
  //     {address: apTeam3.key.accAddress, schedules: [[100, 101, "100"], [100, 110, "100"], [100, 200, "100"]]},
  //     {address: apTeam2.key.accAddress, schedules: [[100, 110, "100"]]},
  //   ]
  // );
  // let new_schedules = [[1000, 2000, "100"]];
  // await testVestingUpdateVestingAccount(
  //   juno,
  //   apTeam,
  //   haloVesting,
  //   apTeam3.key.accAddress,
  //   new_schedules,
  // );
  // await testUserClaimsVestedTokens(juno, apTeam3, haloVesting);
  // await testQueryVestingConfig(juno, haloVesting);
  // await testQueryVestingAccount(juno, haloVesting, "addr0");
  // await testQueryVestingAccounts(juno, haloVesting, undefined, undefined);

  // await testGovUpdateConfig(
  //   juno,
  //   apTeam,
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
  // await testGovExecutePoll(juno, apTeam, haloGov, 1);
  // await testGovEndPoll(juno, apTeam, haloGov, 1);
  // await testGovSnapshotPoll(juno, apTeam, haloGov, 1);
  // await testGovStakeVotingTokens(juno, apTeam, terraswapToken, haloGov, "20000000000");
  // await testGovStakeVotingTokens(juno, apTeam2, terraswapToken, haloGov, "10000000000");
  // await testGovStakeVotingTokens(juno, apTeam3, terraswapToken, haloGov, "5000000000");
  // await testGovWithdrawVotingTokens(juno, apTeam, haloGov, "1000000000");
  // await testGovWithdrawVotingTokens(juno, apTeam2, haloGov, "10000000000");
  // await testGovWithdrawVotingTokens(juno, apTeam3, haloGov, "10000000000");
  // await testGovClaimVotingTokens(juno, apTeam, haloGov);
  // await testGovCastVote(juno, apTeam, haloGov, 1, VoteOption.YES, "1");
  // await testGovRegisterContracts(juno, apTeam, haloGov, terraswapToken);
  // await testGovExecutePollForRegistrarSettings(
  //   juno,
  //   apTeam,
  //   haloGov,
  //   terraswapToken,
  //   "1000000",
  //   100,
  //   "0.5",
  //   "0.1"
  // );
  // await testGovResetClaims(juno, apTeam, haloGov, [
  //   apTeam.key.accAddress,
  //   apTeam2.key.accAddress,
  //   apTeam3.key.accAddress,
  // ]);
  // await testQueryGovConfig(juno, haloGov);
  // await testQueryGovState(juno, haloGov);
  // await testQueryGovClaims(juno, haloGov, apTeam.key.accAddress);
  // await testQueryGovStaker(juno, haloGov, apTeam.key.accAddress);
  // await testQueryGovStaker(juno, haloGov, apTeam2.key.accAddress);
  // await testQueryGovStaker(juno, haloGov, apTeam3.key.accAddress);
  // await testQueryGovPoll(juno, haloGov, 1);
  // await testQueryGovPolls(juno, haloGov, undefined, undefined, undefined);

  // await testQueryGovVoters(juno, haloGov, 1, undefined, undefined);

  // Test query for HALO staking
  // await testStakingUnbond(juno, apTeam, haloStaking, "100");
  // await testStakingWithdraw(juno, apTeam, haloStaking);
  // await testQueryStakingConfig(juno, haloStaking);
  // await testQueryStakingStakerInfo(juno, haloStaking, "addr000", undefined);
  // await testQueryStakingState(juno, haloStaking);

  // Test query for LBP Factory
  // await testFactoryUpdateConfig(
  //   juno,
  //   apTeam,
  //   lbpFactoryContract,
  // undefined,
  //   undefined,
  //   undefined,
  //   undefined,
  //   haloCollector,
  //   undefined
  // );
  // await testFactoryCreatePair(
  //   juno,
  //   apTeam,
  //   lbpFactoryContract,
  //   terraswapToken,
  //   "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4",
  //   datetimeStringToUTC("12/16/2021 00:00:00Z"),
  //   datetimeStringToUTC("12/17/2021 00:00:00Z"),
  //   "96",
  //   "50",
  //   "4",
  //   "50",
  //   "HALO <-> UST Pair"
  // );
  // await getPairContractLpToken(juno, lbpPairContract);
  // await testFactoryUnregister(juno, apTeam, lbpFactoryContract, terraswapToken, "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4");
  // await testQueryFactoryConfig(juno, lbpFactoryContract);
  // await testQueryFactoryPair(juno, lbpFactoryContract, terraswapToken);
  // await testQueryFactoryPairs(juno, lbpFactoryContract);

  // await testPairSwapNativeToHalo(juno, apTeam, lbpPairContract, "100000000");
  // await testPairSwapHaloToNative(
  //   juno,
  //   apTeam,
  //   lbpPairContract,
  //   terraswapToken,
  //   "100000000"
  // );
  // await testQueryPairPair(juno, lbpPairContract);
  // await testQueryPairPool(juno, lbpPairContract);
  // await testQueryPairSimulationNativeToHalo(juno, lbpPairContract);
  // await testQueryPairSimulationHaloToNative(juno, lbpPairContract, terraswapToken);
  // await testQueryPairReverseSimulationNativeToHalo(juno, lbpPairContract);
  // await testQueryPairReverseSimulationHaloToNative(
  //   juno,
  //   lbpPairContract,
  //   terraswapToken
  // );

  // Test query for LBP Router
  // await testQueryRouterConfig(juno, lbpRouterContract);

  // Test Loop Pair
  // await testPairProvideLiquidity(
  //   juno,
  //   apTeam,
  //   terraswapToken,
  //   "terra1yjg0tuhc6kzwz9jl8yqgxnf2ctwlfumnvscupp", // LOOP PAIR
  //   "80000000000000", //HALO
  //   "1300000000000", //UST
  // );

  // await testPairWithdrawLiquidity(
  //   juno,
  //   apTeam,
  //   lbpPairContract,
  //   lbpLpTokenContract,
  //   "100000000"
  // );

  // Test query for LBP Token
  // await testQueryTokenBalance(juno, terraswapToken, apTeam.key.accAddress);
  // await testQueryTokenInfo(juno, terraswapToken);
  // await testQueryTokenMinter(juno, terraswapToken);

  // await testTransferTokenBalance(
  //   juno,
  //   apTeam,
  //   terraswapToken,
  //   apTeam2.key.accAddress,
  //   "420000000"
  // );
}
