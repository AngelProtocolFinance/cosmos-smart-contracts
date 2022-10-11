// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://rpc.uni.juno.deuslabs.fi",
    chainId: "uni-5",
    walletPrefix: "juno",
    nativeToken: "ujunox",
    gasPrice: "0.025ujunox",
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
    registrar: "juno1dmry032p0khv2d5knrtf8q5hzcnv06f8ps24naw9hrpv699zw9xqad4jma",
    accounts: "juno1ltu4ng2gnm5zs3gnmww89udr0kknd5hlds44lq48guem5xxf4nzsawa9sc",
    indexFund: "juno1jxgswmw6kndcqtp545pt0wn4m2rqunf7mzaxncyguuc9nx2tycgqalxeg6",
    cw4GrpApTeam: "juno1ptpsp2shhyqqesd52gwtj0lele46sl3wp3myvxa46zq3r0g596wsztqvur",
    cw3ApTeam: "juno189gdk2aczgkx7p0cyll2whah5afazazhmaw4rmfhh7lda2twyrxseq76zw",
    cw4GrpReviewTeam: "juno129g6ldqjnfpgzyxn3lm5yr67r2zl7e35upsrswejdds49q5s60dqnttzx9",
    cw3ReviewTeam: "juno1dkt5kujyd876wf0xgv3sedsjh6nzlhtdyykaz2e6tmnamzwphryqvaayaj",
    vaultLocked1: "juno1tta5j2tzsgnq2smnwh595alqn8jg6sxll6cym7wg2ufgyvhrnu7qef5sfw",
    vaultLiquid1: "juno1qyh0g7jwgqnxl07h8cs8f5df7aujf5t9xdp4c3a3330qt2gxcnesq5yjrt",
    vaultLocked2: "juno1ew6nzhzskpqqncd0rxlqzd98khgzcfjcy3964q42qm7r2dafuagqrvg634",
    vaultLiquid2: "juno1n0llmlzjqsc60m42q36mzvelyz9dn7yprm0jpp0f457rt9t4km7sk7l483",
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
    walletPrefix: "juno",
    nativeToken: "ujuno",
    gasPrice: "0.025ujuno",
  },
  mnemonicKeys: {
    apTeam: "",
    apTreasury: "juno1q6n47p729sla2jekc45rlmrvwchkj4gc39a296", // Temp use AP Team Admin until we can use the CW3 of the AP Endowment
  },
  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar: "juno17emcut72n6ycmf54qd0l4mzsefqxnqdhqxzlczxstlkkatdlst5qf9s3qr",
    accounts: "juno1e0w8892n60v0juuugvwptj8f6v3ad56ydr3cgxstmpkggjrqzfhsaqh38c",
    indexFund: "juno1yrahlxavwr7juyrty580d24mgvmhknn6h3sgepjtkyg7udvj2l2sujdlqn",
    cw4GrpApTeam: "juno15g9u395kprfhxxzfqhfw56rvwfhjzg8k6mjq82u3yg7fxkhprv8stsu8mm",
    cw3ApTeam: "juno1sae4p8crnac0h9m27psn205d6k586f7cnm4eshws623v05g95teqvj2s8q",
    cw4GrpReviewTeam: "juno1a22f8dxevu3er7vs4lkrca9n8rgf8uvgjd8s2p5eq787vmczq59syuplqx",
    cw3ReviewTeam: "juno1vp2q50smgzw64xm2j2ksntej34pnnedaz4qkwdh8zah9kjcaas6s8g92t8",
    vaults: [],
  },
  members: [
    { addr: "juno1q6n47p729sla2jekc45rlmrvwchkj4gc39a296", weight: 1 },
  ],

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
    usdc_juno_pool: "juno1ctsmp54v79x7ea970zejlyws50cj9pkrmw49x46085fn80znjmpqz2n642",
    usdc_juno_pool_staking: "juno1cuu9qxjqukh9drptk2y50r5tvepes7cy55hffh7quvvawk95lxlq6rzzj0",
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
