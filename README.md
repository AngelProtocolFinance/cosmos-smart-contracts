# Angel Protocol - Integration Testing Suite

## Requirements

- Mac or Linux computer with x86 processors
- docker
- [rust-optimizer](https://github.com/CosmWasm/rust-optimizer)
- [LocalTerra](https://github.com/terra-money/LocalTerra)
- nodejs

Notes:

- **Windows users:** Sorry, but Windows is an inferior OS for software developement. I suggest upgrading to a Mac or install Linux on your PC (I used [arch](https://wiki.archlinux.org/title/installation_guide) btw)
- **M1 Mac users**: Sorry, LocalTerra doesn't run on ARM processors. There is currently no solution to this

## Procedures

### Spin up LocalTerra

```bash
git clone https://github.com/terra-money/LocalTerra.git
cd LocalTerra
git checkout v0.5.0  # important to get the right version!
```

Edit `LocalTerra/config/config.toml` as follows. This speeds up LocalTerra's blocktime which improves our productivity.

```diff
##### consensus configuration options #####
[consensus]

wal_file = "data/cs.wal/wal"
- timeout_propose = "3s"
- timeout_propose_delta = "500ms"
- timeout_prevote = "1s"
- timeout_prevote_delta = "500ms"
- timeout_precommit_delta = "500ms"
- timeout_commit = "5s"
+ timeout_propose = "500ms"
+ timeout_propose_delta = "500ms"
+ timeout_prevote = "500ms"
+ timeout_prevote_delta = "500ms"
+ timeout_precommit_delta = "500ms"
+ timeout_commit = "500ms"
```

Edit `LocalTerra/config/genesis.json` as follows. This fixes the stability fee ("tax") on Terra stablecoin transfers to a constant value (0.1%) so that our test transactions give reproducible results.

```diff
"app_state": {
  "treasury": {
    "params": {
      "tax_policy": {
-       "rate_min": "0.000500000000000000",
-       "rate_max": "0.010000000000000000",
+       "rate_min": "0.001000000000000000",
+       "rate_max": "0.001000000000000000",
      },
-     "change_rate_max": "0.000250000000000000"
+     "change_rate_max": "0.000000000000000000"
    }
  }
}
```

Once done, start LocalTerra by

```bash
docker-compose up  # Ctrl + C to quit
```

From time to time, you may need to revert LocalTerra to its initial state. Do this by

```bash
docker-compose rm
```

How to know if LocalTerra is working properly:

1. **Go to [https://localhost:1317/swagger/](http://localhost:1317/swagger/).** You should see a page with some APIs which can be used to send transactions or query blockchain state. However, we will be using [terra.js]() library to do this instead of from the swagger page
2. **Go to [this Terra Finder page](https://finder.terra.money/localterra/address/terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v).** Don't forget to select "LocalTerra" from the network selector on top right of the page. You should see an account with huge amounts of Luna and stablecoins. This is one of the accounts we will be using for the tests

### Compile contracts

```bash
# .zshrc or .bashrc
# set the optimizer version to whichever latest version of optimizer (currently it is 0.11.5):
alias workspace-optimizer='docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.4'
```

```bash
# from the root folder in the angelprotocol-smart-contracts repo
workspace-optimizer
```

### Create and configure wasms paths file

You need to tell the test suite where to find the wasms artifacts files locally for the various repos it works with.

In the `src/config` folder there is an example file for setting the parameters that point to your local wasm folders: `wasmPaths.ts.example`
In the newly created file, edit the `wasm_path` object's attributes for the `core` and `lbp` to point to the correct local artifacts folders.

```bash
cp ./src/config/wasmPaths.ts.example ./src/config/wasmPaths.ts
nano ./src/config/wasmPaths.ts
```

### Optional: LocalTerra constants file setup

In the `src/config` folder there is an example file for setting the constants for your LocalTerra parameters (contracts, init input settings, wallets, etc): `localterraConstants.ts.example`
In the newly created file, edit the `wasm_path` object's attributes for the `core` and `lbp` to point to the correct local artifacts folders.

```bash
cp ./src/config/localterraConstants.ts.example ./src/config/localterraConstants.ts
nano ./src/config/localterraConstants.ts
```

### Run full setup of contracts & all tests

The npm commands follow a formula to help make it easier to remember commands and ensure we know exactly what we're running and where. It is setup as follows:
`npm run < network >-< action >-< module >`

Network options:

- `localterra`
- `testnet`
- `mainnet`

Action options:

- `setup`: instantiates and configures all contracts for a module
- `migrate`: migrates all contracts for a module (using wasms in the respective repos)
- `tests`: runs all tests that are active the main tests file (`/src/tests/<testnet | mainnet>.ts`) [AN: LocalTerra & TestNet share the testnet tests]

Module options:

- `core`: Registrar, Accounts, Index Fund, Multisigs, Vaults, etc
- `terraswap`: TerraSwap HALO CW20 token, HALO/UST Pair, & HALO/UST Pair LP Token
- `halo`: All "support" contracts for HALO Token (gov, collector, distributor, vesting, etc)
- `lbp`: Astroport LBP contracts (Factory, Pair, LP Token)

#### Complete steps for the setup of AP contracts ecosystem & running test (ex. using LocalTerra)

```bash
npm install
npm run test:localterra-setup-core
npm run test:localterra-setup-terraswap
npm run test:localterra-setup-halo
npm run test:localterra-setup-lbp
npm run test:localterra-tests
```

**NOTE:** After each of the setup commands, you may see key contract addresses or wasm codes that will need to updated in your constants file before proceeding to run the next command. These commands build upon on another.

**ALSO NOTE:** To run the above on other networks, simply swap out the network option in the npm command.

We are building off the excellent work done by 0xLarry (from whom we lovingly :heart: ~~stole~~ borrowed).

**ALSO NOTE:** From `v2`, the test scripts related to `lbp` contracts are not maintained.
The reason is that they are no more needed from `v2`.
We just keep them for completeness.