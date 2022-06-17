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
  anchorMoneyMarket: "terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal",
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
    registrar: "terra1hhagf5n2c6dxfqakrldgng7v3ehwq6vthuylmt3egvj2a07arftsrhrss0",
    indexFund: "terra1j0ynhfnv9ac65y43udzkl5tv8f8w4f6rpd7kdzem0cq2jadwxg6s5eawjk",
    anchorVault1: "",
    anchorVault2: "",
    endowmentContract1: "terra1tuhzd6d99hvu46l3le0mf223qhydjy6t5adjnhjmuxndt76zn8lq486ptz",
    endowmentContract2: "terra1zhslrw988pfk96akjewsqdmv6hxm3rxzfwkufsny6gx30t8hklaqr2g8lc",
    endowmentContract3: "terra1c9uxm9p524v2fsd2ma8xn6kesxuwwgdrkrmpe9em9z9uay9f7kuq5flgca",
    endowmentContract4: "terra1ypz4jlcz66aleqpemcpu3pkf6ccs80glzckqkhluqrsts33x6mwqdywu88",
    cw4GrpApTeam: "terra1j37klku4ujy6ku5hqelnap2esk59kqy8gjlh82nc9u57gsgsz3sqf55as6",
    cw3ApTeam: "terra1nv3v2rv54fdr6lp0mzp6fa6cmfwmm0yrkc4538cl70mnlxz0yd2qgcndxv",
    cw4GrpOwners: "",
    cw3GuardianAngels: "",
  },

  // LBP contracts
  lbp: {
    factory_contract: "terra167m64seqj7cucxm5wep3hyu4suqw4sl5s8uzjz",
    router_contract: "terra19dpanzuhtmdsw8ds5zschrh4mnxcejc0ut6dnk",
    pair_contract: "terra17al3hudq2vcxtyvw9008edjhyqaw74mayva2d8",
    lp_token_contract: "terra19zgdunfrx79nqvznqmx4satj5kxndvmrsx502m",
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
  anchorMoneyMarket: "terra1sepfj7s0aeg5967uxnfk4thzlerrsktkpelm5s",
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
    factory_contract: "terra10dl5u40lj50scntv4qmwykfw2zulf77zyv34u0",
    router_contract: "terra1l32eafhapmn9c8m7epyraxa2yty4xngamvewfs",
    pair_contract: "terra1hhpgcp2stvzx952zfxtxg4dhgf60yfzchesj3e",
    lp_token_contract: "terra1kt26adtzwu4yefw37snr73n393vsu8w0hmazxc",
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
    terraswap_factory: "terra16hdjuvghcumu6prg22cdjl96ptuay6r0hc6yns",
    pair_contract: "terra1yjg0tuhc6kzwz9jl8yqgxnf2ctwlfumnvscupp",
    lp_token_contract: "terra17pzt8t2hmx6587zn6yh5ensylm3s9mm4m72v2n",
    // terra12aazc56hv7aj2fcvmhuxve0l4pmayhpn794m0p /// HALO-LOOP PAIR
  },

  // HALO contracts
  halo: {
    airdrop_contract: "terra1pe6mnf0ursz0h80h2hwk690hvrph8vgt9pnw0w",
    collector_contract: "terra1uxqjsgnq30lg5lhlhwd2gmct844vwqcdlv93x5",
    community_contract: "terra1cjaez6nzl08g4q9yklmxqqqcs79j9p0yfjs2mz",
    distributor_contract: "terra1ya34r8qj0fttkrxx435zexshyqe5fe3vlmhnd6",
    gov_contract: "terra1zcmp45vemypvd3j6ek2j2gz4mevjzyv3jc4ree",
    gov_hodler_contract: "terra1vn8ycrkmm8llqcu82qe3sg5ktn6hajs6tkpnx0",
    staking_contract: "",
    vesting_contract: "terra19vv0hu406qpg9gu7uh4wnqr9lz0dlemw20pz4f",
  },
} as const;
