// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://juno-testnet-rpc.polkachu.com",
    chainId: "uni-6",
    walletPrefix: "juno",
    nativeToken: "ujunox",
    gasPrice: "0.025ujunox",
    axelarGateway:
      "axelar1dv4u5k73pzqrxlzujxg3qp8kvc3pje7jtdvu72npnt5zhq05ejcsn5qme5",
    axelarIbcChannel: "channel-7",
    axelarChainId: "juno",
  },

  mnemonicKeys: {
    apTeam:
      "pact fancy rough prison twenty dismiss mushroom rival page ship quantum deer rookie system cargo",
    apTeam2:
      "knee verify salmon erosion brand ten term three cake help certain bus phrase biology cruel",
    apTeam3:
      "move example spice hint gym liberty weasel drink midnight snow forest vital accident glove dignity",
    apTreasury:
      "truck vacuum tunnel buzz wealth save come short fit kite poverty fork blade venue sword",
    charity1:
      "eager warrior prison into alarm motion annual giggle project silver fabric hover garlic satisfy beach",
    charity2:
      "add buzz humor jump float rotate test rural jazz cave armor pattern update casino undo",
    ast1: "lobster worry angry spoil milk cash asthma unhappy number cave object fortune lens uniform simple",
    ast2: "audit sibling loud strong assume save nose salon travel describe debate pioneer",
    pleb: "announce reflect cinnamon regular address copper below funny lens draft gain wage inhale gold loyal",
    tca: "camp mom loud claim grass kick tail into cake wear mistake box grunt stand runway",
    junoIbcClient:
      "soda tomato draft between amazing grab suit verb help pony elegant oxygen trial cactus coffee",
  },

  wallets: {
    keeper: "juno1kwzx0d3t4m504xj8hluzcvvkyv9qqa7s529642", // AWS KEEPER WALLET (Donatoooor mngmnt wallet for now)
  },

  // Should be updated contract addresses after deploying wasms in the testnet
  contracts: {
    registrar:
      "juno1cgmrq76kx0nh764gumq0r9xa99ec0krvs5y86xqmp8csttc7pwnqh9ftf7",
    cw4GrpApTeam:
      "juno1sj057c5ufqgqjh66y0dgfhvjl6xw2jd96l3lm99fz0pym86uaj9sl62dxn",
    cw3ApTeam:
      "juno17jz7gpzxnrp42q8rt0ultku0205uxye6mq2cwpnvertaqwvcy5dq3q6kdm",
    settingsController:
      "juno1aztgl4v5nnrewnqqlqg9yvwv4s7gtssql9gzvshlyhgt9w80z3aqx5gnvp",
    accounts: "juno1kkzhj4m73p6yylev6ycy6dkneddqy88e5t0kdtla9h0wnnlq3g0q4wmcm4",
    indexFund:
      "juno1ueexvz5hspl8se5448dylq9zpm0fp65qx6g20slj7vuhkeh6yxgqv5x9j6",
    cw4GrpReviewTeam:
      "juno1xwlmdadkrw960fzzl9tqe2u46hxeu44xjyu76zhk343re05x85fqzpnkx3",
    cw3ReviewTeam:
      "juno10kvhxyq56j4tjs8dm8r7yhd0f8ct5hjyrrga0cd6kjqwatej07jqc09fqn",
    swapRouter:
      "juno1gf38ns6acufevhcltvlsh8hedt58xzwn364n5cuvwxpflr99556s4g70un",
    donationMatching: "",
    giftcards:
      "juno1zfjggfdjadcgd4f2jugw440vfdecf5erqfy0ts9f5v9yucmu99rqdsy6j6",
    vaultLocked1: "",
    vaultLiquid1: "",
    vaultLocked2: "",
    vaultLiquid2: "",
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
    // loop_liquidity: "200000000",
    // juno_liquidity: "100000000",

    // halo_token_contract: "",
    // halo_juno_pair_contract: "",
    // halo_juno_pair_lp_token: "",
    // initial_halo_supply: "1000000000000000",
    // halo_liquidity: "200000000",
    // native_liquidity: "100000000",
    lj_pair_loop_liquidity: "200000000",
    lj_pair_juno_liquidity: "100000000",

    malo_token_contract: "",
    malo_juno_pair_contract: "",
    malo_juno_pair_lp_token: "",
    initial_malo_supply: "1000000000000000",
    mj_pair_malo_liquidity: "20000000",
    mj_pair_juno_liquidity: "10000000",

    kalo_token_contract: "",
    kalo_juno_pair_contract: "",
    kalo_juno_pair_lp_token: "",
    initial_kalo_supply: "1000000000000000",
    kj_pair_kalo_liquidity: "20000000",
    kj_pair_juno_liquidity: "10000000",

    malo_kalo_pair_contract: "",
    malo_kalo_pair_lp_token: "",
    mk_pair_malo_liquidity: "10000000",
    mk_pair_kalo_liquidity: "10000000",
  },

  // HALO contracts
  halo: {
    airdrop_contract: "",
    collector_contract: "",
    community_contract: "",
    distributor_contract: "",
    gov_contract: "",
    gov_hodler: "",
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
    url: "https://juno-rpc.polkachu.com",
    chainId: "juno-1",
    walletPrefix: "juno",
    nativeToken: "ujuno",
    gasPrice: "0.075ujuno",
    axelarGateway: "???",
    axelarIbcChannel: "???",
    axelarChainId: "juno",
  },

  wallets: {
    apTreasury:
      "juno1fz8jx4qhkgdrm5vm3s20n670mf872xsp2c0a6cl0yuncquzmj4jss2cfgj", // this is the CW3 of the AP Endowment
    keeper: "juno1kwzx0d3t4m504xj8hluzcvvkyv9qqa7s529642", // AWS KEEPER WALLET (Donatoooor mngmnt wallet for now)
  },

  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar:
      "juno17emcut72n6ycmf54qd0l4mzsefqxnqdhqxzlczxstlkkatdlst5qf9s3qr",
    accounts: "juno1e0w8892n60v0juuugvwptj8f6v3ad56ydr3cgxstmpkggjrqzfhsaqh38c",
    indexFund:
      "juno1yrahlxavwr7juyrty580d24mgvmhknn6h3sgepjtkyg7udvj2l2sujdlqn",
    cw4GrpApTeam:
      "juno15g9u395kprfhxxzfqhfw56rvwfhjzg8k6mjq82u3yg7fxkhprv8stsu8mm",
    cw3ApTeam:
      "juno1sae4p8crnac0h9m27psn205d6k586f7cnm4eshws623v05g95teqvj2s8q",
    cw4GrpReviewTeam:
      "juno1a22f8dxevu3er7vs4lkrca9n8rgf8uvgjd8s2p5eq787vmczq59syuplqx",
    cw3ReviewTeam:
      "juno1vp2q50smgzw64xm2j2ksntej34pnnedaz4qkwdh8zah9kjcaas6s8g92t8",
    swapRouter: "",
    donationMatching: "",
    giftcards:
      "juno17pghl3qreyqnjlq6hun5ymshl0dkfeelcy738dkgk602lzmgcvaq2e4xav",
    settingsController: "",
    vaults: [],
  },
  members: [{ addr: "juno1q6n47p729sla2jekc45rlmrvwchkj4gc39a296", weight: 1 }],

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
    usdc_juno_pool:
      "juno1ctsmp54v79x7ea970zejlyws50cj9pkrmw49x46085fn80znjmpqz2n642",
    usdc_juno_pool_staking:
      "juno1cuu9qxjqukh9drptk2y50r5tvepes7cy55hffh7quvvawk95lxlq6rzzj0",
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
    gov_hodler: "",
    staking_contract: "",
    vesting_contract: "",
  },
} as const;
