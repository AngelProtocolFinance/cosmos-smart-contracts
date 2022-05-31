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
  anchorMoneyMarket: "terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal",
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
    registrar: "terra15upcsqpg57earvp7mc49kl5e7cppptu2ndmpak",
    indexFund: "terra1typpfzq9ynmvrt6tt459epfqn4gqejhy6lmu7d",
    anchorVault1: "terra1mvtfa3zkayfvczqdrwahpj8wlurucdykm8s2zg",
    anchorVault2: "terra16y7du2keuersslsevvqx32z04wy6juyfwjs3ru",
    endowmentContract1: "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v",
    endowmentContract2: "terra1glqvyurcm6elnw2wl90kwlhtzrd2zc7q00prc9",
    endowmentContract3: "terra1vyw5r7n02epkpk2tm2lzt67fyv28qzalwzgzuu",
    endowmentContract4: "terra1jvtf3ccpkr3vymv98vk9nz7wvwmykgv8yk9l3w",
    cw4GrpApTeam: "terra1wpnzy6w9gd3tt9wkvnqkcmzkyc8v0tgz75nuue",
    cw3ApTeam: "terra1qspgamxqn9slwe7ecca4n2fs2xsl5hxvkc9lzs",
    cw4GrpOwners: "terra1ldrkpnysrasq4sg4zu9mgh74wt9nxvk9qgvxtd",
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
    terraswap_token_code: 148,
    terraswap_factory: "terra18qpjm4zkvqnpjpw0zn0tdr8gdzvt8au35v45xf",
    halo_token_contract: "terra1ah3gd4uhxtrpc3qeqn84l6v3wcvfkh3vw25fyl",
    halo_ust_pair_contract: "",
    halo_ust_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "20000000000",
    native_liquidity: "1000000000", // reduced to 1000 UST due to faucet limitations
  },

  // HALO contracts
  halo: {
    airdrop_contract: "terra15n2j80ufyrup8ply5nhjwwerfjpz7cx3m2hcqq",
    collector_contract: "terra12h9ssarf78t3sw3zu5xyr4v3cxjkmqmnxujlzl",
    community_contract: "terra19hhk4wu5yrj90qwh6lt0zkpe65h7dlvlr48ujt",
    distributor_contract: "terra1vuktqwu0n5df0sfswzwwh4wpgznu4d2urflvlk",
    gov_contract: "terra16tw444h6qtzxr4kf2p276qt0u6w3ggtc20xgly",
    gov_hodler_contract: "terra1mcjrurlzmne3hlqvjypyacz9l8xpf4r6zq9sa6",
    staking_contract: "",
    vesting_contract: "terra1h30cngl0hruj46dzh95wepa2n74hzenlvlq6cx",
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
    terraswap_token_code: 3,
    terraswap_factory: "terra1ulgw0td86nvs4wtpsc80thv6xelk76ut7a7apj",
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
