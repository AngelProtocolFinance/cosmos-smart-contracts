# Angel Protocol - Integration Testing Suite

## Requirements

- Mac or Linux computer with x86 processors
- docker
- [rust-optimizer](https://github.com/CosmWasm/rust-optimizer)
- [LocalJuno](https://github.com/CosmosContracts/juno)
- nodejs

Notes:

- **Windows users:** Sorry, but Windows is an inferior OS for software developement. I suggest upgrading to a Mac or install Linux on your PC (I used [arch](https://wiki.archlinux.org/title/installation_guide) btw)
- **M1 Mac users**: Sorry, LocalJuno doesn't run on ARM processors. There is currently no solution to this

## Procedures

### Spin up LocalJuno
1. Clone the junod repo: 
```bash
git clone https://github.com/CosmosContracts/juno.git
```
2. Copy the file named `junod_docker-compose.yml` in this `test-suite` repo over to the base folder of the junod repo, replacing the existing default `docker-compose.yml` file. 
```bash
cp ./junod_docker-compose.yml <juno_repo_path>/docker-compose.yml`
```
3. In the jundo repo base folder run the following to build junod container:
```bash
docker-compose build
```

Once the build is done, you can start your LocalJuno by running
```bash
docker-compose up  # Ctrl + C to quit
```

From time to time, you may need to revert LocalJuno to its initial state. Do this by running
```bash
docker-compose rm
```

How to know if LocalJuno is working properly?
**Go to [https://localhost:1317](http://localhost:1317).** You should see a page with some APIs which can be used to send transactions or query blockchain state.

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

### Optional: LocalJuno constants file setup

In the `src/config` folder there is an example file for setting the constants for your LocalJuno parameters (contracts, init input settings, wallets, etc): `localjunoConstants.ts.example`
In the newly created file, edit the `wasm_path` object's attributes for the `core` to point to the correct local artifacts folders.

```bash
cp ./src/config/localjunoConstants.ts.example ./src/config/localjunoConstants.ts
nano ./src/config/localjunoConstants.ts
```

### Run full setup of contracts & all tests

The npm commands follow a formula to help make it easier to remember commands and ensure we know exactly what we're running and where. It is setup as follows:
`npm run < network >-< action >-< module >`

Network options:

- `localjuno`
- `testnet`
- `mainnet`

Action options:

- `setup`: instantiates and configures all contracts for a module
- `migrate`: migrates all contracts for a module (using wasms in the respective repos)
- `tests`: runs all tests that are active the main tests file (`/src/tests/<testnet | mainnet>.ts`) [AN: LocalJuno & TestNet share the testnet tests]

Module options:

- `core`: Registrar, Accounts, Index Fund, Multisigs, Vaults, etc
- `terraswap`: TerraSwap HALO CW20 token, HALO/axlUSDC Pair, & HALO/axlUSDC Pair LP Token
- `halo`: All "support" contracts for HALO Token (gov, collector, distributor, vesting, etc)

#### Complete steps for the setup of AP contracts ecosystem & running test (ex. using LocalJuno)

```bash
npm install
npm run test:localjuno-setup-core
npm run test:localjuno-setup-terraswap
npm run test:localjuno-setup-halo
npm run test:localjuno-tests
```

**NOTE:** After each of the setup commands, you may see key contract addresses or wasm codes that will need to updated in your constants file before proceeding to run the next command. These commands build upon on another.

**ALSO NOTE:** To run the above on other networks, simply swap out the network option in the npm command.

We are building off the excellent work done by 0xLarry (from whom we lovingly :heart: ~~stole~~ borrowed).
