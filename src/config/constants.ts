  // ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  // TestNet pisco-1
  networkInfo: {
    url: "https://pisco-lcd.terra.dev",
    chainId: "pisco-1",
  },
  // TestNet MoneyMarket Contract
  anchorMoneyMarket: "",
  // TestNet AP / DANO Treasury wallet
  apTreasury: "terra1tc2yp07pce93uwnneqr0cptqze6lvke9edal3l",  // apTeam wallet address
  mnemonicKeys: {
    apTeam:
      "forward stone width wrist outer elder supply summer extra erosion spring unlock rhythm sail goose once city ivory eight diesel upper measure betray purchase",
    apTeam2:
      "custom review state crisp modify sell trick replace bone wolf ridge paper later collect topple income owner head turkey estate canyon tone copy inhale",
    apTeam3:
      "law cause body surround problem join swift shy lumber start immense spray mandate organ pledge butter modify fossil pluck demise link bus rebel misery",
    charity1:
      "source multiply curtain modify nurse party valid awesome road local focus retreat route agree spot rule false cloud dwarf six relief clay unhappy thank",
    charity2:
      "stick dumb cabin wish great impact fork save trade crime today seed tortoise base enter topic physical glue maple cliff over myth marble loyal",
    charity3:
      "write obscure shop lunar fruit attend media abuse spirit lens illegal pluck rally cave stamp gadget burger rigid minute index paper voice eight again",
    pleb: "shoot cry panther mesh blind embrace bottom exchange forest dad polar popular siege idea sure guard disorder toss above tube gaze finish whip column",
    tca: "win height tragic load when music day issue game track promote midnight desert ordinary thunder barely ahead wealth bundle force spray shop cushion mystery",
  },
  // Should be updated contract addresses after deploying wasms in the testnet
  contracts: {
    registrar: "terra1jpdrgx66yhz23yjs0nzthjtrafrhfmmst67h73atn6km8pnuh7ys42dnlj",
    indexFund: "terra1h6j0jhetc3qef4m8ypksa82jmn5wlue9dxncpx53zxvznlsukdwq9vagev",
    anchorVault1: "",
    anchorVault2: "",
    endowmentContract1: "terra1lf9r0s4w5fht76u6pddnxhvj2j09y62al27ldtjrznrj04tph8aspte3zg",
    endowmentContract2: "terra15faa0u5mpr3vj2vaktp5pq99d40c6jsdtdfl9dcuaa2l5034h66qrqhlj7",
    endowmentContract3: "terra17n3h42egrnqj5uh0fjuegtmhh9qrfpxvqtfj5awrn9wmvcf6zm0sfhtg8u",
    endowmentContract4: "terra1frzxamuqnnnmr6kaa4wzyrg6q2vty70tmpwhg6hv6a80ltjpluqskrav7y",
    cw4GrpApTeam: "terra138ejrx6fu62gx7jwnypjfjxt2p9epm083892tymxh93kzhysu74qak3clf",
    cw3ApTeam: "terra1hjx9hspsm62x7aasxktxsv75asquyss9ns3wmr03xsqyp3wfj4vqxpq5l6",
    cw4GrpOwners: "",
    cw3GuardianAngels: "",
  },

  // LBP contracts
  lbp: {
    factory_contract: "",
    router_contract: "",
    pair_contract: "",
    lp_token_contract: "",
    // HALO/UST Pair token supply amount
    halo_token_amount: "80000000000",
    native_token_amount: "1300000000", // adjusted down from localterra/mainnet values due to faucet limitations
    // HALO/UST Pair start/end times
    lbp_start_time: "2021-12-18T02:04:00.000Z",
    lbp_end_time: "2021-12-19T02:04:00.000Z",
    // HALO/UST Pair start/end weights
    token_start_weight: "96",
    token_end_weight: "50",
    native_start_weight: "4",
    native_end_weight: "50",
    slippage_tolerance: "0.01",
  },

  // TerraSwap contracts
  terraswap: {
    terraswap_token_code: 83,
    terraswap_factory: "terra1jha5avc92uerwp9qzx3flvwnyxs3zax2rrm6jkcedy2qvzwd2k7qk7yxcl",
    halo_token_contract: "terra1xqvq0sglawp39crdax6729uexcp49c842tlmvk26wwkdjlq9qx2skrd7rf",
    halo_uluna_pair_contract: "terra1p6qq27havgzu9p9rxw9zqflghs23ffm9gppf95ea63w9h2ahwd9sg0l989",
    halo_uluna_pair_lp_token: "terra19hv35wnhgxnr64rr96z0y8qxgc85q8my9g249cpxmyzxxwaw0zpq0n3mak",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "2000000",
    native_liquidity: "1000000", // reduced to 1 LUNA due to faucet limitations
  },

  // HALO contracts
  halo: {
    airdrop_contract: "terra1dxav8ulphpqvvv00jd6hkvx7smfzhyn2345s3uydvpvr6fd8salqcc88d9",
    collector_contract: "terra1qhslq202gvj4e9w5mgkl096rmy63waeulkay88sjp6kk6vehcqsq04n7nd",
    community_contract: "terra1jc0ykn63fseg9h8lxwgqg9cehf9r9dzg6gtgtl4nr3kejvnltndqfscvfv",
    distributor_contract: "terra1z48excnads2hmqnuur2ag5v2qkrxrv4az8zxq66v8nawznrysqfqtaxnjq",
    gov_contract: "terra1jdvcmnzkf7zc8ua2kqnlac8uhyl9wp0ucgrfv9mds9xzgndspv3srlkrn9",
    gov_hodler_contract: "terra19qhtl7gwj46x98ne0vr4fwayee87u5fqcj3njqkyrtuxx0h6j9vs59mmtx",
    staking_contract: "terra1ev7rjtp26e0gy8szclz78a4depwl8l7cg369my93z0qrsqt8tc2qf2yzja",
    vesting_contract: "terra1c5elxxtafafpmrsc63dg32tptc4qp5rw65nrj3xuhs4whka6xvpstwwyyc",
  },
} as const;

// ---------------------------------------------------------------------------------------------------
// MainNet information
// ---------------------------------------------------------------------------------------------------
export const mainnet = {
  // MainNet phoenix-1
  networkInfo: {
    url: "https://phoenix-lcd.terra.dev",
    chainId: "phoenix-1",
  },
  // MainNet MoneyMarket Contract
  anchorMoneyMarket: "",
  // MainNet AP / DANO Treasury wallet
  apTreasury: "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly",
  mnemonicKeys: {
    apTeam:
      "forward stone width wrist outer elder supply summer extra erosion spring unlock rhythm sail goose once city ivory eight diesel upper measure betray purchase",
  },
  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar: "terra1nwk2y5nfa5sxx6gtxr84lre3zpnn7cad2f266h",
    indexFund: "terra19cevhng6nunl7gmc90sph0syuqyvtqn7mlhwz0",
    anchorVault: "terra172ue5d0zm7jlsj2d9af4vdff6wua7mnv6dq5vp",
    cw4GrpApTeam: "terra1eueh924845wwsc2mna5u3ysn79q66kwqgq26mj",
    cw3ApTeam: "terra1zrwpm9htqdh80nhqcuvw999cexvtmu0xt4dks5",
    cw4GrpOwners: "terra1lycc2zyhd676294c604euh8hxw7h6jrjd68x83", // NO LONGER USED!!
    cw3GuardianAngels: "terra1jd2n0ze7er80x9h8k3x006aypaxs7mvrggdmn9", // NO LONGER USED!!
    endowmentContracts: [],
  },
  members: [
    { addr: "terra1numzqm5mgr56ftd4y8mfen7705nfs4vpz5jf0s", weight: 1 },
    { addr: "terra1wvsugzhszkstexl0v6fv86c9ryjy8xm6u9t2fk", weight: 1 },
    { addr: "terra103rakc90xgcuxaee6alqhkmnp7qh92hwt0hxur", weight: 1 },
    { addr: "terra1kqk3x5mscrl94z6jfqam78rrdg42uyc3w63mye", weight: 1 },
    { addr: "terra1qxma5jlwlxx8mfu5ge7rnq3x03asaptd4fvaa4", weight: 1 },
  ],

  // LBP contracts
  lbp: {
    factory_contract: "",
    router_contract: "",
    pair_contract: "",
    lp_token_contract: "",
    // HALO/UST Pair token supply amount
    halo_token_amount: "80000000000000",
    native_token_amount: "1300000000000",
    // HALO/UST Pair start/end times
    lbp_start_time: "2021-12-19T15:00:00.000Z",
    lbp_end_time: "2021-12-22T15:00:00.000Z",
    // HALO/UST Pair start/end weights
    token_start_weight: "96",
    token_end_weight: "50",
    native_start_weight: "4",
    native_end_weight: "50",
    slippage_tolerance: "0.01",
  },

  // TerraSwap contracts
  terraswap: {
    terraswap_token_code: 4,
    terraswap_factory: "terra1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjqxl5qul",
    halo_token_contract: "terra1w8kvd6cqpsthupsk4l0clwnmek4l3zr7c84kwq",
    halo_ust_pair_contract: "",
    halo_ust_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "",
    native_liquidity: "",
  },

  // LOOP LP contracts
  loop: {
    terraswap_factory: "",
    pair_contract: "",
    lp_token_contract: "",
  },

  // HALO contracts
  halo: {
    airdrop_contract: "",
    collector_contract: "",
    community_contract: "",
    distributor_contract: "",
    gov_contract: "",
    gov_hodler_contract: "",
    staking_contract: "",
    vesting_contract: "",
  },
} as const;
