import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { datetimeStringToUTC } from "../../utils/helpers";
import {
  testBeneficiaryCanWithdrawFromLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
} from "./core/accounts";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
  testUpdateAngelAllianceMembers,
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
  testAddApTeamMemberToC4Group,
  testAddGuardiansToEndowment,
  testGuardiansChangeEndowmentOwner,
} from "./core/multisig";
import {
  testAngelTeamCanTriggerVaultsHarvest,
  testClosingEndpoint,
  testMigrateAllAccounts,
  testUpdatingRegistrarConfigs,
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarApprovedVaultRateList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
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
  testGovWithdrawVotingTokens,
  testGovExecutePollForRegistrarSettings,
  testQueryGovConfig,
  testQueryGovPoll,
  testQueryGovPolls,
  testQueryGovStaker,
  testQueryGovState,
  testQueryGovVoters,
  VoteOption,
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
  testVestingUpdateVestingAccount,
  testQueryVestingConfig,
  testQueryVestingAccount,
  testQueryVestingAccounts,
} from "./halo/vesting";
import {
  testFactoryUpdateConfig,
  testFactoryCreatePair,
  testFactoryUnregister,
  testFactoryUpdatePair,
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
import {
  testRouterSwapOperations,
  testQueryRouterConfig,
  testQueryRouterSimulateSwapOperations,
} from "./lbp/router";
import {
  testQueryTokenBalance,
  testQueryTokenInfo,
  testQueryTokenMarketingInfo,
  testQueryTokenMinter,
  testPairWithdrawLiquidity,
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
  lbpFactoryContract: string,
  lbpPairContract: string,
  lbpRouterContract: string,
  lbpLpTokenContract: string,
): Promise<void> {
  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateAngelAllianceMembers(terra, apTeam, indexFund, [
  //   "terra178u9lz89f54njqz6nentst3m9nye2cc7ezssmq", // testnet admin (testnet ONLY!)
  //   "terra1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf", // charity#1 (testnet ONLY!)
  //   "terra1zxtczmxtw8mk8xncvr8lcq2qmvk4dz88ek6f79", // community
  //   "terra1janh9rs6pme3tdwhyag2lmsr2xv6wzhcrjz0xx", // community
  //   "terra18n2pc9x6q9str9dz8sqpt7ulz5telutclkzaec", // lunapes
  //   "terra17me29hk8cdd6mm6uf7cf0amsxmzxnszkfe5ph4", // lunabulls
  //   "terra1r59snugfm3gxjas565jf5ehw54junlfpmspjan", // lunabulls
  //   "terra1tz9jtxemq5e9sw048adz32tj62vkzp6f63e26f", // Astronorcs
  //   "terra1pl2cus25j79ukff04fxn9wwemerm2463gnztl6", // west coast
  //   "terra1etwq0q8wwnmq7322kz4v6ff2dcvwfm634vdkqn", // loop finance
  //   "terra157vv7nqra4zpfa58cglen5ekqmekxqw5ss3edq", // loop finance
  //   "terra1feqtlvaru4lszqnpjesgfw8splrg7u27wwwqac", // loop finance
  //   "terra1rzjxj4c6ykemk8csvtjchcqas7mul8s4w6rk8x", // tales of terra
  //   "terra1kf4k0l7hj5tlkuzf67ly43q8d2gcxay3hwa7fr", // hero
  //   "terra1yvg94g6ydgme2kdhy5t4gasgvem2kpk56g4h5e", // NFT Luna
  //   "terra14amh70rm5a3wjgkf7trvten3jfqum2svppky3e", // Terra Terrapins
  //   "terra1amrl8f5fqen2m478nuh2z7mz5ce096x4xqae9p", // Woof of Luna
  //   "terra1hxrd8pnqytqpelape3aemprw3a023wryw7p0xn", // ApolloDAO
  // ]);
  // testRemoveIndexFund(terra, apTeam, indexFund, 5);
  // await testCreateIndexFund(terra, apTeam, indexFund, 11, "MVP Rotation #5", "Fund collection for MVP", true, [
  // "", // New funds go here!!
  // ]);
  // await testUpdateFundMembers(terra, apTeam, apTeam, indexFund, 6, ["","",""], ["","",""]);
  // Guardian angels multisig test
  // await testAddApTeamMemberToC4Group(terra, apTeam, apTeam3, cw3ApTeam, cw4GrpApTeam);
  // await testAddGuardiansToEndowment(terra, apTeam3, charity1, charity2, charity3, pleb, cw3GuardianAngels, endowmentContract1);
  // await testGuardiansChangeEndowmentOwner(terra, charity2, charity3, pleb, endowmentContract1, cw3GuardianAngels);
  // Test execute
  // await testRejectUnapprovedDonations(terra, pleb, endowmentContract3);
  // await testDonorSendsToIndexFund(terra, pleb, indexFund);
  // await testTcaMemberSendsToIndexFund(terra, tca, indexFund);
  // await testAngelTeamCanTriggerVaultsHarvest(terra, apTeam, charity1, registrar);
  // await testCharityCanUpdateStrategies(terra, charity1, endowmentContract1, anchorVault1, anchorVault2);
  // await testBeneficiaryCanWithdrawFromLiquid(terra, charity1, endowmentContract1, anchorVault1, anchorVault2);
  // await testUpdatingRegistrarConfigs(terra, apTeam, registrar, haloCollector);
  // await testClosingEndpoint(terra, apTeam, registrar, endowmentContract3, endowmentContract4);
  // await testMigrateAllAccounts(terra, apTeam, registrar);
  // await testUpdateFundMembers(terra, apTeam, pleb, indexFund, 2, [endowmentContract2], [endowmentContract4]);
  // testCreateIndexFund(terra, apTeam, indexFund, 5, "Test fund", "My 5th test fund", [endowmentContract2, endowmentContract3, endowmentContract4]);
  // testRemoveIndexFund(terra, apTeam, indexFund, 1);
  // Test query
  // await testQueryRegistrarConfig(terra, registrar);
  // await testQueryRegistrarEndowmentList(terra, registrar);
  // await testQueryRegistrarApprovedVaultList(terra, registrar);
  // await testQueryRegistrarApprovedVaultRateList(terra, registrar);
  // await testQueryRegistrarVaultList(terra, registrar);
  // await testQueryRegistrarVault(terra, registrar, anchorVault1);
  // await testQueryAccountsBalance(terra, endowmentContract1);
  // await testQueryVaultConfig(terra, anchorVault1);
  // await testQueryAccountsConfig(terra, endowmentContract1);
  // await testQueryAccountsEndowment(terra, endowmentContract1);
  // await testQueryIndexFundConfig(terra, indexFund);
  // await testQueryIndexFundState(terra, indexFund);
  // await testQueryIndexFundTcaList(terra, indexFund);
  // await testQueryIndexFundFundsList(terra, indexFund);
  // await testQueryIndexFundFundDetails(terra, indexFund, 3);
  // await testQueryIndexFundActiveFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDonations(terra, indexFund);
  // await testQueryIndexFundDeposit(terra, indexFund);

  // Test query for HALO airdrop
  // await testAirdropUpdateConfig(terra, apTeam, apTeam2, pleb, haloAirdrop);
  // await testAirdropRegisterNewMerkleRoot(terra, apTeam, haloAirdrop);
  // await testAirdropClaim(terra, apTeam, haloAirdrop);
  // await testQueryAirdropConfig(terra, haloAirdrop);
  // await testQueryAirdropMerkleRoot(terra, haloAirdrop, 1);
  // await testQueryAirdropIsClaimed(terra, haloAirdrop, 1, "terra1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8");
  // await testQueryAirdropLatestStage(terra, haloAirdrop);

  // Test query for HALO collector
  // await testCollectorUpdateConfig(terra, apTeam, pleb, haloGov, haloCollector, "0.5", undefined);
  // await testCollectorSweep(terra, apTeam, haloCollector);
  // await testQueryCollectorConfig(terra, haloCollector);
  // await testQueryCollectorPair(terra, haloCollector);

  // Test query for HALO community
  // await testCommunityUpdateConfig(terra, apTeam, pleb, haloGov, haloCommunity, "1000000", undefined);
  // await testCommunitySpend(terra, apTeam, haloGov, haloCommunity, "addr000", "1000000");
  // await testQueryCommunityConfig(terra, haloCommunity);

  // Test query for HALO distributor
  // await testDistributorUpdateConfig(terra, apTeam, pleb, haloDistributor, "1000000", undefined);
  // await testDistributorSpend(terra, apTeam, haloDistributor, "addr000", "1000000");
  // await testDistributorAdd(terra, apTeam, haloGov, haloDistributor, apTeam2.key.accAddress);
  // await testDistributorRemove(terra, apTeam, haloGov, haloDistributor, apTeam2.key.accAddress);
  // await testQueryDistributorConfig(terra, haloDistributor);

  // Test query for HALO vesting
  // await testVestingUpdateConfig(terra, apTeam, haloVesting, apTeam2.key.accAddress, undefined, undefined);
  // await testVestingRegisterVestingAccounts(
  //   terra,
  //   apTeam,
  //   haloVesting,
  //   [
  //     {address: "addr0", schedules: [[100, 101, "100"], [100, 110, "100"], [100, 200, "100"]]},
  //     {address: "addr1", schedules: [[100, 110, "100"]]},
  //     {address: "addr2", schedules: [[100, 200, "100"]]},
  //   ]
  // );
  // await testVestingUpdateVestingAccount(
  //   terra,
  //   apTeam,
  //   haloVesting,
  //   {address: "addr1", schedules: [[100, 110, "200"]]}
  // );
  // await testQueryVestingConfig(terra, haloVesting);
  // await testQueryVestingAccount(terra, haloVesting, "addr0");
  // await testQueryVestingAccounts(terra, haloVesting, undefined, undefined);

  // Test query for HALO gov
  // await testGovUpdateConfig(terra, apTeam, pleb, haloGov, apTeam2.key.accAddress, undefined, undefined, undefined, undefined, undefined, undefined);
  // await testGovExecutePoll(terra, apTeam, haloGov, 1);
  // await testGovEndPoll(terra, apTeam, haloGov, 1);
  // await testGovSnapshotPoll(terra, apTeam, haloGov, 1);
  // await testGovWithdrawVotingTokens(terra, apTeam, haloGov, "11");
  // await testGovCastVote(terra, apTeam, haloGov, 1, VoteOption.YES, "1");
  // await testGovRegisterContracts(terra, apTeam, haloGov, terraswapToken);
  // await testGovExecutePollForRegistrarSettings(terra, apTeam, haloGov, terraswapToken, "1000000", 100, "0.5", "0.1");
  // await testQueryGovConfig(terra, haloGov);
  // await testQueryGovState(terra, haloGov);
  // await testQueryGovPoll(terra, haloGov, 1);
  // await testQueryGovPolls(terra, haloGov, undefined, undefined, undefined);
  // await testQueryGovStaker(terra, haloGov, "voter1");
  // await testQueryGovVoters(terra, haloGov, 1, undefined, undefined);

  // Test query for HALO staking
  // await testStakingUnbond(terra, apTeam, haloStaking, "100");
  // await testStakingWithdraw(terra, apTeam, haloStaking);
  // await testQueryStakingConfig(terra, haloStaking);
  // await testQueryStakingStakerInfo(terra, haloStaking, "addr000", undefined);
  // await testQueryStakingState(terra, haloStaking);

  // Test query for LBP Factory
  // await testFactoryUpdateConfig(terra, apTeam, factoryContract, undefined, undefined, undefined, undefined, haloCollector, undefined);
  // await testFactoryUpdatePair(terra, apTeam, factoryContract, pairContract, 1639234038);
  // const lbp_start_time = "12/14/2021 00:00:00Z";
  // const lbp_end_time = "12/17/2021 00:00:00Z";
  // await testFactoryCreatePair(
  //   terra,
  //   apTeam,
  //   lbpFactoryContract,
  //   terraswapToken,
  //   "uusd",
  //   datetimeStringToUTC(lbp_start_time),
  //   datetimeStringToUTC(lbp_end_time),
  //   "96",
  //   "50",
  //   "4",
  //   "50",
  //   "HALO <-> UST Pair"
  // );
  // await getPairContractLpToken(terra, lbpPairContract);
  // await testFactoryUnregister(terra, apTeam, lbpFactoryContract, terraswapToken, "uusd");
  // await testQueryFactoryConfig(terra, factoryContract);
  // await testQueryFactoryPair(terra, factoryContract, terraswapToken);
  // await testQueryFactoryPairs(terra, factoryContract);

  // Test query for LBP Pair
  // await testPairProvideLiquidity(terra, apTeam2, terraswapToken, pairContract, "100000000", "100000000");
  // await testPairWithdrawLiquidity(terra, apTeam, pairContract, lpTokenContract, "100000000");
  // await testPairSwapNativeToHalo(terra, apTeam, pairContract, "100000000");
  // await testPairSwapHaloToNative(terra, apTeam, pairContract, terraswapToken, "100000000");
  // await testQueryPairPair(terra, pairContract);
  // await testQueryPairPool(terra, pairContract);
  // await testQueryPairSimulationNativeToHalo(terra, pairContract);
  // await testQueryPairSimulationHaloToNative(terra, pairContract, terraswapToken);
  // await testQueryPairReverseSimulationNativeToHalo(terra, pairContract);
  // await testQueryPairReverseSimulationHaloToNative(terra, pairContract, terraswapToken);

  // Test query for LBP Router
  // await testRouterSwapOperations(terra, apTeam, routerContract, terraswapToken, apTeam.key.accAddress, "100000000");
  // await testQueryRouterConfig(terra, routerContract);
  // await testQueryRouterSimulateSwapOperations(terra, routerContract, terraswapToken, "100000000");

  // Test query for LBP Token
  // await testQueryTokenBalance(terra, terraswapToken, apTeam.key.accAddress);
  // await testQueryTokenInfo(terra, terraswapToken);
  // await testQueryTokenMinter(terra, terraswapToken);
  // await testQueryTokenMarketingInfo(terra, terraswapToken);
}
