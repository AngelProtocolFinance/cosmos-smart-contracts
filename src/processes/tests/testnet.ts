import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
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
  testCharityCanHarvestWithdrawFee,
  testCharityCanHarvestAUMFee,
} from "./core/accounts";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
  testUpdateAllianceMembersList,
  testUpdatingIndexFundConfigs,
  testCreateIndexFund,
  testRemoveIndexFund,
  testQueryIndexFundActiveFundDetails,
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
  testApproveEndowments,
  testCreateEndowmentViaRegistrar,
  testAngelTeamCanTriggerVaultsHarvest,
  testClosingEndpoint,
  testMigrateAllAccounts,
  testUpdatingRegistrarConfigs,
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarApprovedVaultRateList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
  testQueryRegistrarEndowmentDetails,
  testQueryRegistrarVault,
  testQueryRegistrarVaultList,
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
import {
  testFactoryUpdateConfig,
  testFactoryCreatePair,
  testFactoryUnregister,
  testQueryFactoryConfig,
  testQueryFactoryPair,
  testQueryFactoryPairs,
} from "./lbp/factory";
import {
  testQueryPairPair,
  testQueryPairPool,
  testQueryPairReverseSimulationNativeToHalo,
  testQueryPairReverseSimulationHaloToNative,
  testQueryPairSimulationNativeToHalo,
  testQueryPairSimulationHaloToNative,
  testPairProvideLiquidity,
  testPairSwapHaloToNative,
  testPairSwapNativeToHalo,
  getPairContractLpToken,
} from "./lbp/pair";
import { testQueryRouterConfig } from "./lbp/router";
import {
  testQueryTokenBalance,
  testQueryTokenInfo,
  testQueryTokenMinter,
  testPairWithdrawLiquidity,
  testTransferTokenBalance,
} from "./lbp/token";

export async function testExecute(
  terra: LocalTerra | LCDClient,
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
  anchorVault1: string,
  anchorVault2: string,
  endowmentContract1: string,
  endowmentContract2: string,
  endowmentContract3: string,
  endowmentContract4: string,
  cw4GrpApTeam: string,
  // cw4GrpOwners: string,
  cw3ApTeam: string,
  // cw3GuardianAngels: string,
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
  lbpFactoryContract: string,
  lbpPairContract: string,
  lbpRouterContract: string,
  lbpLpTokenContract: string,
  slippageTolerance: string | undefined
): Promise<void> {
  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateAllianceMembersList(
  //   terra,
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
  // await testRemoveIndexFund(terra, apTeam, indexFund, 5);
  // await testCreateIndexFund(
  //   terra,
  //   apTeam,
  //   indexFund,
  //   "", // name
  //   "", // description
  //   false, // rotating_fund
  //   [
  //     "terra178u9lz89f54njqz6nentst3m9nye2cc7ezssmq", // testnet admin (testnet ONLY!)
  //     "terra1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf", // charity#1 (testnet ONLY!)
  //   ],
  // );
  // await testUpdateFundMembers(terra, apTeam, indexFund, 4, ["","",""], ["","",""]);
  // await testChangeManyAccountsEndowmentOwners(terra, apTeam, [{ address: endowmentContract1, owner: apTeam.key.accAddress }]);

  // await testCreateAccountCw4GroupCw3Multisig(
  //   terra,
  //   apTeam,
  //   registrar,
  //   62653, // cw4Code
  //   62654, // cw3Code
  //   {
  //     address: "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v",
  //     owner: "terra1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf",
  //   }
  // ); //UNIMPLEMENTED
  // await testCreateEndowmentViaRegistrar(terra, apTeam, registrar, {
  //   owner: charity1.key.accAddress,
  //   name: "Test Endowment #5",
  //   description: "Endowment created from the test-suite integration test",
  //   beneficiary: charity1.key.accAddress,
  //   withdraw_before_maturity: false,
  //   maturity_time: undefined,
  //   maturity_height: undefined,
  //   guardians_multisig_addr: undefined,
  //   cw4_members: [{ addr: charity1.key.accAddress, weight: 1 }],
  //   profile: {
  //     name: "Test Endowment #5",
  //     overview: "Endowment created from the test-suite integration test",
  //     un_sdg: 2,
  //     tier: 3,
  //     logo: undefined,
  //     image: undefined,
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
  // await testAddMemberToC4Group(terra, apTeam, cw3ApTeam, cw4GrpApTeam, "terra1......");
  // await testUpdateCw3Config(terra, apTeam, cw3ApTeam, 50, 25000);
  // await testAddGuardiansToEndowment(terra, apTeam3, charity1, charity2, charity3, pleb, cw3GuardianAngels, endowmentContract1);  //SHOULD_REMOVE
  // await testGuardiansChangeEndowmentOwner(terra, charity2, charity3, pleb, endowmentContract1, cw3GuardianAngels); //SHOULD_REMOVE
  // await testQueryMultisigVoters(terra, cw3ApTeam);
  // await testQueryMultisigThreshold(terra, cw3ApTeam);
  // await testQueryGroupMembersList(terra, cw4GrpApTeam);

  // Test execute
  // await testRejectUnapprovedDonations(terra, pleb, endowmentContract1, "10000000");
  // await testDonorSendsToIndexFund(terra, pleb, indexFund, 3, "0.5", "4200000");
  // await testTcaMemberSendsToIndexFund(terra, tca, indexFund); //UNKNOWN_ERROR
  // await testAngelTeamCanTriggerVaultsHarvest(
  //   terra,
  //   apTeam,
  //   charity1,
  //   registrar,
  //   haloCollector,
  //   "0.5"
  // );
  // await testCharityCanUpdateStrategies(
  //   terra,
  //   charity1,
  //   endowmentContract1,
  //   anchorVault1,
  //   anchorVault2
  // ); //RUNTIME_ERROR:UNREACHABLE
  // await testBeneficiaryCanWithdrawFromLiquid(
  //   terra,
  //   charity3,
  //   endowmentContract3,
  //   anchorVault1,
  //   pleb.key.accAddress
  // );
  // await testUpdatingRegistrarConfigs(terra, apTeam, registrar, {
  //   cw3_code: 102,
  //   cw4_code: 104,
  //   accounts_code_id: 102,
  // });
  // await testApproveEndowments(terra, apTeam, registrar, endowmentContract1, 1);
  // await testClosingEndpoint(
  //   terra,
  //   apTeam,
  //   registrar,
  //   endowmentContract3,
  //   endowmentContract4
  // ); //QUERY_YIELDVAULT_NOT_FOUND_ERR
  // await testUpdateFundMembers(
  //   terra,
  //   apTeam,
  //   indexFund,
  //   2,
  //   [endowmentContract2],
  //   [endowmentContract4]
  // );
  // await testCreateIndexFund(
  //   terra,
  //   apTeam,
  //   indexFund,
  //   "Test fund for Ukraine Portal",
  //   "Another portal test fund",
  //   false,
  //   [endowmentContract2, endowmentContract3, endowmentContract4]
  // );
  // await testRemoveIndexFund(terra, apTeam, indexFund, 1);
  // await testCharityCanHarvestWithdrawFee(terra, charity1, endowmentContract1, anchorVault1);
  // await testCharityCanHarvestAUMFee(terra, charity1, endowmentContract1);
  
  // Test query
  // await testQueryRegistrarConfig(terra, registrar);
  // await testQueryRegistrarEndowmentList(terra, registrar);
  // await testQueryRegistrarEndowmentDetails(terra, registrar, endowmentContract1);
  // await testQueryRegistrarApprovedVaultList(terra, registrar);
  // await testQueryRegistrarApprovedVaultRateList(terra, registrar);
  // await testQueryRegistrarVaultList(terra, registrar);
  // await testQueryRegistrarVault(terra, registrar, anchorVault1); //EMPTY_ANCHORVAULT_ERR
  // await testQueryVaultConfig(terra, anchorVault1);  //EMPTY_ANCHORVAULT_ERR
  // await testQueryAccountsBalance(terra, endowmentContract4); //QUERY_VAULT_ADDRESS_ERR
  // await testQueryAccountsConfig(terra, endowmentContract4);
  // await testQueryAccountsEndowment(terra, endowmentContract1);
  // await testQueryAccountsProfile(terra, endowmentContract4);
  // await testQueryAccountsState(terra, endowmentContract4);
  // await testQueryAccountsTransactions(
  //   terra,
  //   endowmentContract4,
  //   undefined,
  //   undefined,
  //   undefined
  // );
  // await testQueryIndexFundConfig(terra, indexFund);
  // await testQueryIndexFundState(terra, indexFund);
  // await testQueryIndexFundTcaList(terra, indexFund);
  // await testQueryIndexFundFundsList(terra, indexFund);
  // await testQueryIndexFundFundDetails(terra, indexFund, 4);
  // await testQueryIndexFundActiveFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDonations(terra, indexFund);
  // await testQueryIndexFundDeposit(terra, indexFund);

  // Test query for HALO airdrop
  // await testAirdropUpdateConfig(terra, apTeam, apTeam2, pleb, haloAirdrop);
  // await testAirdropRegisterNewMerkleRoot(terra, apTeam2, haloAirdrop);
  // await testAirdropClaim(terra, apTeam2, haloAirdrop); //MERKLE_VERIFICATION_FAILED
  // await testQueryAirdropConfig(terra, haloAirdrop);
  // await testQueryAirdropMerkleRoot(terra, haloAirdrop, 1);
  // await testQueryAirdropIsClaimed(terra, haloAirdrop, 1, "terra1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8");
  // await testQueryAirdropLatestStage(terra, haloAirdrop);

  // Test query for HALO collector
  // await testCollectorUpdateConfig(
  //   terra,
  //   apTeam,
  //   haloCollector,
  //   "1.0",
  //   haloGov,
  //   undefined
  // );
  // await testCollectorSweep(terra, apTeam, haloCollector); //ZERO_AMOUNT_ERR
  // await testQueryCollectorConfig(terra, haloCollector);
  // await testQueryCollectorPair(terra, haloCollector);

  // Test query for HALO community
  // await testCommunityUpdateConfig(terra, apTeam, pleb, haloGov, haloCommunity, "1000000", undefined); //UNKNOWN_ERR
  // await testCommunitySpend(terra, apTeam, haloGov, haloCommunity, "addr000", "1000000");
  // await testQueryCommunityConfig(terra, haloCommunity);

  // Test query for HALO distributor
  // await testDistributorUpdateConfig(terra, apTeam, haloDistributor, "1000000", haloGov); //UNKNOWN_ERR
  // await testDistributorSpend(terra, apTeam, haloDistributor, "addr000", "1000000"); //UNKNOWN_ERR
  // await testDistributorAdd(terra, apTeam, haloGov, haloDistributor, apTeam2.key.accAddress); //UNKNOWN_ERR
  // await testDistributorRemove(terra, apTeam, haloGov, haloDistributor, apTeam2.key.accAddress); //UNKNOWN_ERR
  // await testQueryDistributorConfig(terra, haloDistributor);

  // Tests for HALO vesting
  // await testVestingUpdateConfig(terra, apTeam, haloVesting, undefined, undefined, undefined); //UNKNOWN_ERR
  // await testVestingRegisterVestingAccounts(
  //   terra,
  //   apTeam,
  //   haloVesting,
  //   [
  //     {address: apTeam3.key.accAddress, schedules: [[100, 101, "100"], [100, 110, "100"], [100, 200, "100"]]},
  //     {address: apTeam2.key.accAddress, schedules: [[100, 110, "100"]]},
  //   ]
  // ); //UNKNOWN_ERR
  // let new_schedules = [[1000, 2000, "100"]];
  // await testVestingUpdateVestingAccount(
  //   terra,
  //   apTeam,
  //   haloVesting,
  //   apTeam3.key.accAddress,
  //   new_schedules,
  // ); //UNKNOWN_ERR
  // await testUserClaimsVestedTokens(terra, apTeam3, haloVesting); //UNKNOWN_ERR
  // await testQueryVestingConfig(terra, haloVesting); //UNKNOWN_ERR
  // await testQueryVestingAccount(terra, haloVesting, "addr0"); //UNKNOWN_ERR
  // await testQueryVestingAccounts(terra, haloVesting, undefined, undefined); //UNKNOWN_ERR

  // await testGovUpdateConfig(
  //   terra,
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
  // await testGovExecutePoll(terra, apTeam, haloGov, 1); //UNKNOWN_ERR
  // await testGovEndPoll(terra, apTeam, haloGov, 1);  //UNKNOWN_ERR
  // await testGovSnapshotPoll(terra, apTeam, haloGov, 1); //UNKNOWN_ERR
  // await testGovStakeVotingTokens(terra, apTeam, terraswapToken, haloGov, "20000000000");
  // await testGovStakeVotingTokens(terra, apTeam2, terraswapToken, haloGov, "10000000000");
  // await testGovStakeVotingTokens(terra, apTeam3, terraswapToken, haloGov, "5000000000");
  // await testGovWithdrawVotingTokens(terra, apTeam, haloGov, "1000000000");
  // await testGovWithdrawVotingTokens(terra, apTeam2, haloGov, "10000000000");
  // await testGovWithdrawVotingTokens(terra, apTeam3, haloGov, "10000000000"); //UNKNOWN_ERR
  // await testGovClaimVotingTokens(terra, apTeam, haloGov); //UNKNOWN_ERR
  // await testGovCastVote(terra, apTeam, haloGov, 1, VoteOption.YES, "1"); //UNKNOWN_ERR
  // await testGovRegisterContracts(terra, apTeam, haloGov, terraswapToken);
  // await testGovExecutePollForRegistrarSettings(
  //   terra,
  //   apTeam,
  //   haloGov,
  //   terraswapToken,
  //   "1000000",
  //   100,
  //   "0.5",
  //   "0.1"
  // ); //UNKNOWN_ERR
  // await testGovResetClaims(terra, apTeam, haloGov, [
  //   apTeam.key.accAddress,
  //   apTeam2.key.accAddress,
  //   apTeam3.key.accAddress,
  // ]); //UNKNOWN_ERR
  // await testQueryGovConfig(terra, haloGov);
  // await testQueryGovState(terra, halGov);
  // await testQueryGovClaims(terra, haloGov, apTeam.key.accAddress);
  // await testQueryGovStaker(terra, haloGov, apTeam.key.accAddress);
  // await testQueryGovStaker(terra, haloGov, apTeam2.key.accAddress);
  // await testQueryGovStaker(terra, haloGov, apTeam3.key.accAddress);
  // await testQueryGovPoll(terra, haloGov, 1); //UNKNOWN_ERR
  // await testQueryGovPolls(terra, haloGov, undefined, undefined, undefined);

  // await testQueryGovVoters(terra, haloGov, 1, undefined, undefined); //UNKNOWN_ERR

  // Test query for HALO staking
  // await testStakingUnbond(terra, apTeam, haloStaking, "100"); //UNKNOWN_ERR
  // await testStakingWithdraw(terra, apTeam, haloStaking); //UNKNOWN_ERR
  // await testQueryStakingConfig(terra, haloStaking);
  // await testQueryStakingStakerInfo(terra, haloStaking, "addr000", undefined); //UNKNOWN_ERR
  // await testQueryStakingState(terra, haloStaking);

  /** 
  * NOTE: Following tests are no more needed & maintained in v2.
  *       We leave them just for completeness.
  */

  // Test query for LBP Factory
  // await testFactoryUpdateConfig(
  //   terra,
  //   apTeam,
  //   lbpFactoryContract,
  //   undefined,
  //   undefined,
  // );
  // await testFactoryCreatePair(
  //   terra,
  //   apTeam,
  //   lbpFactoryContract,
  //   terraswapToken,
  //   "uusd",
  //   datetimeStringToUTC("12/16/2021 00:00:00Z"),
  //   datetimeStringToUTC("12/17/2021 00:00:00Z"),
  //   "96",
  //   "50",
  //   "4",
  //   "50",
  //   "HALO <-> UST Pair"
  // );
  // await getPairContractLpToken(terra, lbpPairContract);
  // await testFactoryUnregister(terra, apTeam, lbpFactoryContract, terraswapToken, "uusd");
  // await testQueryFactoryConfig(terra, lbpFactoryContract);
  // await testQueryFactoryPair(terra, lbpFactoryContract, terraswapToken);
  // await testQueryFactoryPairs(terra, lbpFactoryContract);

  // await testPairSwapNativeToHalo(terra, apTeam, lbpPairContract, "100000000");
  // await testPairSwapHaloToNative(
  //   terra,
  //   apTeam,
  //   lbpPairContract,
  //   terraswapToken,
  //   "100000000"
  // );
  // await testQueryPairPair(terra, lbpPairContract);
  // await testQueryPairPool(terra, lbpPairContract);
  // await testQueryPairSimulationNativeToHalo(terra, lbpPairContract);
  // await testQueryPairSimulationHaloToNative(terra, lbpPairContract, terraswapToken);
  // await testQueryPairReverseSimulationNativeToHalo(terra, lbpPairContract);
  // await testQueryPairReverseSimulationHaloToNative(
  //   terra,
  //   lbpPairContract,
  //   terraswapToken
  // );

  // Test query for LBP Router
  // await testQueryRouterConfig(terra, lbpRouterContract);

  // Test Loop Pair
  // await testPairProvideLiquidity(
  //   terra,
  //   apTeam,
  //   terraswapToken,
  //   // "terra1yjg0tuhc6kzwz9jl8yqgxnf2ctwlfumnvscupp", // LOOP PAIR
  //   terraswapPair, // LOOP PAIR
  //   "80000000000000", //HALO
  //   "1300000000000", //UST
  // );

  // await testPairWithdrawLiquidity(
  //   terra,
  //   apTeam,
  //   lbpPairContract,
  //   lbpLpTokenContract,
  //   "100000000"
  // );

  // Test query for LBP Token
  // await testQueryTokenBalance(terra, terraswapToken, apTeam.key.accAddress);
  // await testQueryTokenInfo(terra, terraswapToken);
  // await testQueryTokenMinter(terra, terraswapToken);

  // await testTransferTokenBalance(
  //   terra,
  //   apTeam,
  //   terraswapToken,
  //   apTeam2.key.accAddress,
  //   "420000000"
  // );
}
