// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://rpc.uni.junomint.com:443/", // "https://rpc.uni.juno.deuslabs.fi",
    chainId: "uni-3",
  },
  mnemonicKeys: {
    apTeam: "pact fancy rough prison twenty dismiss mushroom rival page ship quantum deer rookie system cargo",
    apTeam2: "knee verify salmon erosion brand ten term three cake help certain bus phrase biology cruel",
    apTeam3: "move example spice hint gym liberty weasel drink midnight snow forest vital accident glove dignity",
    apTreasury: "truck vacuum tunnel buzz wealth save come short fit kite poverty fork blade venue sword",
    charity1: "eager warrior prison into alarm motion annual giggle project silver fabric hover garlic satisfy beach",
    charity2: "add buzz humor jump float rotate test rural jazz cave armor pattern update casino undo",
    charity3: "lobster worry angry spoil milk cash asthma unhappy number cave object fortune lens uniform simple",
    pleb: "announce reflect cinnamon regular address copper below funny lens draft gain wage inhale gold loyal",
    tca: "camp mom loud claim grass kick tail into cake wear mistake box grunt stand runway",
  },
  // Should be updated contract addresses after deploying wasms in the testnet
  contracts: {
    registrar: "juno1junqfhptf0tnx843eepl8pksk25wvcgapfhsz6v97u87ax0qjpus55qsga",
    accounts: "juno1gfrzmrs9qj6av5r4q9d5sdysl9m84a2c8ls8yv8lqg4qm6aftfkql2v6df",
    indexFund: "juno1wg3f833gp0qze6g5exsc3wfe4rhjd6w7j8u000usyekngt8aku6sfp32z0",
    cw4GrpApTeam: "juno1zzzj2l20ma43v0296ey453j00ju2uy9havdmavyp6kqeke2pvdysrf3cps",
    cw3ApTeam: "juno16esez5e20tqwah87cujkscxqt37zf48vyhqhj9m8ney64m9mwucqz6dr6f",
    cw4GrpReviewTeam: "juno1pgcp5q80wappe4dc0zurugft88k82t70znznsphpyeweyz5vsjvq2e6dt7",
    cw3ReviewTeam: "juno1g4s92cezesughqge4m9atmp96nykh6dr5244z4y4gl2suvzcjfvqh6e0ma",
    vaultLocked1: "juno1uug8emfy54d5s30cuuv59n4p3fjau6t5drl0xdqtvgkm5ks7jagqr8add8",
    vaultLiquid1: "juno1rlwmzl5de0rusrcaetetyuuh5dcxshr52w0c3srq4kjduxfy82pseevrks",
    vaultLocked2: "juno13lxq4tzj4fjnu57zxen6g5ryhuuzhw8cls6njd07cmg8t0f77hqskfup60",
    vaultLiquid2: "juno1zj5yfu7s032glge2um0qmrjpyhzf8frc9f0m429nkgy4ekrcsjssqey9j0",
    endowId1: 1,
    endowId2: 2,
    endowId3: 3,
  },

  // JunoSwap contracts
  junoswap: {
    junoswap_code: 0,
    junoswap_token_code: 83,
    junoswap_stake_code: 0,
    halo_token_contract: "",
    halo_juno_pool_contract: "",
    halo_juno_pool_lp_token: "",
    halo_juno_pool_lp_staking_addr: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "2000000",
    native_liquidity: "1000000", // reduced to 1 JUNO due to faucet limitations
  },


  // Loopswap contracts
  loopswap: {
    loopswap_token_code: 0,
    loopswap_pair_code: 0,

    loopswap_factory: "",
    loopswap_farming: "",

    loop_token_contract: "",
    loop_juno_pair_contract: "",
    loop_juno_pair_lp_token: "",
    initial_loop_supply: "1000000000000000",
    loop_liquidity: "200000000",
    juno_liquidity: "100000000",
    
    halo_token_contract: "",
    halo_juno_pair_contract: "",
    halo_juno_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "200000000",
    native_liquidity: "100000000",
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
  // MainNet phoenix-1
  networkInfo: {
    url: "https://rpc-juno.itastakers.com",
    chainId: "juno-1",
  },
  mnemonicKeys: {
    apTeam: "",
    apTreasury: "",
  },
  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar: "",
    accounts: "",
    indexFund: "",
    anchorVault: "",
    cw4GrpApTeam: "",
    cw3ApTeam: "",
    cw4GrpReviewTeam: "",
    cw3ReviewTeam: "",
    endowmentIDs: [],
  },
  members: [
    { addr: "", weight: 1 },
    { addr: "", weight: 1 },
    { addr: "", weight: 1 },
    { addr: "", weight: 1 },
    { addr: "", weight: 1 },
  ],

  // JunoSwap contracts
  junoswap: {
    junoswap_token_code: 4,
    junoswap_factory: "",
    halo_token_contract: "",
    halo_ust_pair_contract: "",
    halo_ust_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "",
    native_liquidity: "",

    usdc_juno_pool: "juno1ctsmp54v79x7ea970zejlyws50cj9pkrmw49x46085fn80znjmpqz2n642",
    usdc_juno_pool_staking: "juno1cuu9qxjqukh9drptk2y50r5tvepes7cy55hffh7quvvawk95lxlq6rzzj0",
  },

  // LOOP LP contracts
  loop: {
    junoswap_factory: "",
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
