# CosmWsm Smart Contracts (see `cosmwasm` folder)

![Angel Protocol Endowment Smart Contracts - v1 8 Overview](https://user-images.githubusercontent.com/85138450/191880770-7518d238-45a9-4f9b-9da1-cd94753a64b8.png)

## Components

### Core Contracts:
- [Registrar](./contracts/core/registrar) - Contracts for the creation and management of Endowment Accounts smart contracts
core platform of smart contracts that support multiple verticals of specialized smart contracts. 
- [Accounts](./contracts/core/accounts) - Implementation of the Charity Endowment Accounts. 
- [Index Fund](./contracts/core/index-fund) - Contract that acts as a gateway for donors and Terra Charity Alliance members to donate to a groups of charitites as a single Index Fund (grouped by UN SDGs).

### Normalized Endowments:
- [cw3-endowment](./contracts/normalized-endowment/cw3-endowment) - CW3 Implementation for the Normalized Endowment Accounts (required for all endowments)
- [donation-match](./contracts/normalized-endowment/donation-match) -Donation matching contract for Normalized Endowment Accounts (optional in normalized endowment setup)
- [subdao](./contracts/normalized-endowment/subdao) - Governance (sub-dao) to be used by Normalized Endowment Accounts (optional in normalized endowment setup)
- [subdao-bonding-token](./contracts/normalized-endowment/subdao-bonding-token) - Bonding token that can be used by the SubDao for issuance of it's dao-token (optional in normalized endowment setup)

### Vault Contracts:
- [Vault](./contracts/vault) - Vault contracts act as bridges, allowing charity endowment accounts to invest their funds into various TeFi/DeFi protocols in order to earn yield, based on their Strategy allocations.

### MultiSig Contracts:
- [AP Team Group](./contracts/multisig/cw4-group)
- [AP Team](./contracts/multisig/cw3-apteam) - MultiSig contract for enabling Angel Protocol Team's stewardship over all Core and Vault contracts.
- [Review Team](./contracts/multisig/cw3-applications) - MultiSig contract for enabling Angel Protocol Review Team's to approve Charity endowment applications recieved.
- [Endowment](./contracts/multisig/cw3-endowment) - Allows an Endowment's Members to create a proposal to manage their Endowment or other special actions that requires approval from AP Team CW3.


## Getting setup for development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker
- Cargo
- [LocalTerra](https://github.com/terra-project/localterra)(main branch)

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
cargo integration-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cargo build
```

#### Production

For production builds, run the following from the root folder:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.9
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.


# Integration Testing Suite (see `test-suite` folder)

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
2. Copy the files in `./localjuno_env_files` dir of this `test-suite` repo over to the base folder of the junod repo, replacing the existing default `docker-compose.yml` file. 
```bash
LOCALJUNO_PATH=<juno_repo_path>
cp ./localjuno_env_files/docker-compose.yml $LOCALJUNO_PATH/docker-compose.yml
cp ./localjuno_env_files/setup_junod.sh $LOCALJUNO_PATH/docker/setup_junod.sh
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

### Spin up LocalTerra
1. Clone the LocalTerra repo: 
```bash
git clone https://github.com/terra-money/LocalTerra.git
```
2. Copy the files in `./localterra_env_files` dir of this `test-suite` repo over to the base folder of the LocalTerra repo, replacing the existing default files with same name. 
```bash
LOCALTERRA_PATH=<localterra_repo_path>
cp ./localterra_env_files/genesis.json $LOCALTERRA_PATH/config/genesis.json
cp ./localterra_env_files/app.toml $LOCALTERRA_PATH/config/app.toml
cp ./localterra_env_files/client.toml $LOCALTERRA_PATH/config/client.toml
cp ./localterra_env_files/config.toml $LOCALTERRA_PATH/config/config.toml
cp ./localterra_env_files/docker-compose.yml $LOCALTERRA_PATH/docker-compose.yml
cp ./localterra_env_files/fcd.env $LOCALTERRA_PATH/fcd.env
```

3. In the LocalTerra repo base folder run the following to build LocalTerra container:
```bash
docker-compose build
```

Once the build is done, you can start your LocalTerra by running
```bash
docker-compose up  # Ctrl + C to quit
```

From time to time, you may need to revert LocalTerra to its initial state. Do this by running
```bash
docker-compose rm
```

How to know if LocalTerra is working properly?
**Go to [https://localhost:1307](http://localhost:1307).** You should see a page with some APIs which can be used to send transactions or query blockchain state.

### Compile contracts

```bash
# In ~/.zshrc or ~/.bashrc
# set the optimizer version to whichever latest version of optimizer (currently it is 0.11.5):
alias workspace-optimizer='docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.10'
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

The `yarn tests` command walks you through the process of running actions against our contracts on the various networks we support. To help make it easier we've setup CLI prompts to collect: 
1. The Network you want to interact with.
- `localjuno`
- `testnet`
- `mainnet`
- `localterra`
- `localibc`

2. The Action you want to take. 
Based on your network chosen in step #1, the actions may vary. Valid options will be presented to you in a list. In general, actions are as follows:
- `setup`: instantiates and configures all contracts for a module
- `migrate`: migrates all contracts for a module (using wasms in the respective repos)
- `tests`: runs all tests that are active/uncommented in the main tests file (`/src/tests/<testnet | mainnet>.ts`) [AN: LocalJuno & TestNet share the testnet tests]

3. OPTIONAL: Target for action
This specifies the exact target of the action, where more than one is possible. It's only applicable to `setup` & `migrate` actions. General info about the target categories and what they cover:
- `core`: Registrar, Accounts, Index Fund, Multisigs, Vaults, etc
- `junoswap`: JunoSwap HALO CW20 token, HALO/axlUSDC Pair, & HALO/axlUSDC Pair LP Token
- `halo`: All "support" contracts for HALO Token (gov, collector, distributor, vesting, etc)


**NOTE:** After each of the `setup` action commands is executed, you may see contract addresses or wasm code information displayed. That will need to updated in the `./src/config/constants.ts` file before proceeding to run the next action commands. These commands build upon on another, with `setup` being the starting point for testing on localjuno.


We are building off the excellent work done by 0xLarry (from whom we lovingly :heart: ~~stole~~ borrowed).
