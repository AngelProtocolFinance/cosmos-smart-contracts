// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://rpc.uni.junomint.com:443/", // "https://rpc.uni.juno.deuslabs.fi",
    chainId: "uni-5",
    walletPrefix: "juno",
    nativeToken: "ujunox",
    gasPrice: "0.0025ujunox",
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
    registrar: "juno1ul4msjc3mmaxsscdgdtjds85rg50qrepvrczp0ldgma5mm9xv8yqpv6zva",
    accounts: "juno12cfxezwfq4q90nk0g5zvzpjf0t9t2gfdn8vlnwkhm9lpde0pd49qwfdvn4",
    indexFund: "juno1mx32w9tnfxv0z5j000750h8ver7qf3xpj09w3uzvsr3hq68f4hxqvzedz6",
    cw4GrpApTeam: "juno1lqgdq9u8zhcvwwwz3xjswactrtq6qzptmlzlh6xspl34dxq32uhqhlphat",
    cw3ApTeam: "juno186ucx5mtdq6ams8rsvvcu7yfw5lhtxue8ykdkyqvlnk3gpc77las5wms6m",
    cw4GrpReviewTeam: "juno14483x4pm76hwpzyvj56ccarl8kls3tdyz2rtve7p0u7lj9dgsjcqft5umc",
    cw3ReviewTeam: "juno1qt0gkcrvcpv765k8ec4tl2svvg6hd3e3td8pvg2fsncrt3dzjefsmyhx8r",
    vaultLocked1: "juno1ppg8jl8gpe9wv7ga5h7k3z2g3889nvnh4qdv9wh5830ngy9a85tq8x3s65",
    vaultLiquid1: "juno1450hrg6dv2l58c0rvdwx8ec2a0r6dd50hn4frk370tpvqjhy8khqcx0908",
    vaultLocked2: "juno1u6sg78xududaer4mwmgh2n2s6ta7az74ghg647a75nsyhcdsrspsneuynh",
    vaultLiquid2: "juno1s0vkxnwyzfeeexqctu9mjargd8utwf90djllz4cf57ausv79k2hs2usqth",
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
    apTreasury: "juno19kvyyqmlqwtfkzmzkrpascj0yaz2j9msl4kqlzlj4nhmxt0gmljs3qa3lm", // CW3 attached to the AP Endowment contract
  },
  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar: "juno16uva5mgmzj78rdwf5hcqv688lyenms6s32zfjectg8zkr4vc3xfq95eueu",
    accounts: "juno1wkfy4u8zj45jqeeyvl7pedzcvaksutug56d0v32p6qw3fmpyw6rq82y648",
    indexFund: "juno1ycpml96cru0ln20zv7qxkc6xuass8lerpk0cfwvgmvyn6zjux55srqkah5",
    cw4GrpApTeam: "juno1ucjmf3nztyq4a6q8tzja9thefq7092s9r4a2yk8267spr2fays2qks9rl4",
    cw3ApTeam: "juno1dftgv4yhy8yqx95c7a3jar9dg5nnq4p2m50nzk6wdlkahd2h4hms3js63a",
    cw4GrpReviewTeam: "juno1p2yg2xy7z39mhnn4r248nfsf9zufa7kxh34g7kh9u7zvjacm4nssx2x6gv",
    cw3ReviewTeam: "juno1et8z5we83ny06j5sra9q9vaxw6e9cp4qjy5agy8emqz8wcmmgheqnq5xdy",
    vaults: [],
  },
  members: [
    { addr: "juno1c2ha303qdjlcpphjjv7s7r8ns2lnqm9ec79lfa", weight: 1 },
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
