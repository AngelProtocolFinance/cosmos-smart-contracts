  // ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://rpc.uni.juno.deuslabs.fi",
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
    registrar: "juno1zde54v47we5lt4sg3jlza387gww2079rppyvggaru9klxjp6mj2qp6ysrx",
    indexFund: "juno1ztfa658hzrny7gjvupljvvq2mxzkaa0glf0tk529wcwl0netg3kstwmhgj",
    vault1: "",
    vault2: "",
    endowmentContract1: "juno1zng3lrjqjp9ursn7n45ljndwgz8h6ctn2zjt3wlqycp8lhwfrwms6lf4ew",
    endowmentContract2: "juno1emd64lgz3hvth9mkm7lu49m3fzyy0utpwrty96ag7qvmyu43c85s5c74ew",
    endowmentContract3: "juno16swjy3ar6g25cfnrnavfl5d4s32plk8xgperckgh592fcf92rlhsvuyrxk",
    endowmentContract4: "juno1j8kxy50nupc42kawthngruhcedgg9czcdpxnq9p8n8nc80d9nd5qtcfw7g",
    cw4GrpApTeam: "juno1y7kmj4qaehy8a0csldxnqptp5ajgzyj4j3ydjjcahsal5ulg6h8qqvdwem",
    cw3ApTeam: "juno1rahc28lsvetcra9fwueekm3u0l00jgdsnry8amcu8j9ah3c60ydqzrd70r",
  },
  // JunoSwap contracts
  junoswap: {
    junoswap_token_code: 83,
    junoswap_factory: "",
    halo_token_contract: "",
    halo_luna_pair_contract: "",
    halo_luna_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "2000000",
    native_liquidity: "1000000", // reduced to 1 JUNO due to faucet limitations
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
    indexFund: "",
    anchorVault: "",
    cw4GrpApTeam: "",
    cw3ApTeam: "",
    endowmentContracts: [],
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
