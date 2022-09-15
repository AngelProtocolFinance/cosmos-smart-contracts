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
    registrar: "juno1mhnds5wkr26haxpe33svu0jvwsk2nnrwd89zuthc25kjwmx53tgs7hat9k",
    accounts: "juno16sxyhrfjcq7z5q2fhe66q9tkc4204agdynrksj8y6vkucg3sgnrqdkp7ue",
    donationMatching: "",
    indexFund: "juno1hm8djnz470mrxq67tydezdzhvqsslef4qe5h42aher7grq3venxsprm8py",
    cw4GrpApTeam: "juno1vqwwnmzzgeat62rja9e390fe79aqq56lpamtxcq2c7jzsxxtvwpsc8q5mv",
    cw3ApTeam: "juno1ytnpxkfnt2qeynyspwzu7hzlyulpu4far5gtu9xppalg2p4neecsgdthpj",
    cw4GrpReviewTeam: "juno1nn0k4sf330qvzk797y068c9hdy75ng9tg3fzpkmafgrepnqcwk3sh5nqer",
    cw3ReviewTeam: "juno1sa3mzc7mg7ndlxz0laph3lnvx2v5uwmn286grgyxqurgrztz02hsnnltmx",
    vaultLocked1: "juno1f2rh7uzavg8vayymsv70z9r72f9z7627nthaey4eqv7fy9ytz8hqf9wa3h",
    vaultLiquid1: "juno1w3jj4l4xjs0as9hgkjd0lmchl2ygcprfvhlua97tzgtk9pddeujsrh5c7e",
    vaultLocked2: "juno1k2wx9r9409wqguwwmwrfz2u6kptmtlcsusfmqe8yaqec5f4tzujqskrw9p",
    vaultLiquid2: "juno14kq7mcpdgzu7dmgnj7j9p4ucg5tnqne78dc6hqy6sdxhjpswzn3se5j809",
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
    donationMatching: "",
    indexFund: "",
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
