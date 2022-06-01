  // ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  // TestNet bombay-12
  networkInfo: {
    url: "https://pisco-lcd.terra.dev",
    chainId: "pisco-1",
  },
  // TestNet MoneyMarket Contract
  anchorMoneyMarket: "",
  // TestNet AP / DANO Treasury wallet
  apTreasury: "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly",
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
    registrar: "",
    indexFund: "",
    anchorVault1: "",
    anchorVault2: "",
    endowmentContract1: "",
    endowmentContract2: "",
    endowmentContract3: "",
    endowmentContract4: "",
    cw4GrpApTeam: "",
    cw3ApTeam: "",
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
    terraswap_token_code: 148,
    terraswap_factory: "",
    halo_token_contract: "",
    halo_ust_pair_contract: "",
    halo_ust_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "20000000000",
    native_liquidity: "1000000000", // reduced to 1000 UST due to faucet limitations
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

// ---------------------------------------------------------------------------------------------------
// MainNet information
// ---------------------------------------------------------------------------------------------------
export const mainnet = {
  // MainNet columbus-5
  networkInfo: {
    url: "https://lcd.terra.dev",
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
    registrar: "",
    indexFund: "",
    anchorVault: "",
    cw4GrpApTeam: "",
    cw3ApTeam: "",
    cw4GrpOwners: "", // NO LONGER USED!!
    cw3GuardianAngels: "", // NO LONGER USED!!
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
    terraswap_token_code: 3,
    terraswap_factory: "",
    halo_token_contract: "",
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
