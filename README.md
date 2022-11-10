# Angel Protocol Smart Contracts

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
