# Angel Protocol Smart Contracts

## Components

### Core Contracts:
- [Accounts](./contracts/core/accounts) - Implementation of the Charity Endowment Accounts. 
- [Registrar](./contracts/core/registrar) - Contracts for the creation and management of Endowment Accounts smart contracts
core platform of smart contracts that support multiple verticals of specialized smart contracts. 
- [Index Fund](./contracts/core/index-fund) - Contract that acts as a gateway for donors and Terra Charity Alliance members to donate to a groups of charitites as a single Index Fund (grouped by UN SDGs).

### Vault Contracts:
- [Vault](./contracts/vault) - Vault contracts at as bridges, allowing charity endowment accounts to invest their funds into various TeFi/DeFi protocols in order to earn yield, based on their Strategy allocations.

### Guardian Angels Platform (MultiSig) Contracts:
- [AP Team](./contracts/guardian-angels/ap-team-cw3) - MultiSig contract for enabling Angel Protocol Team's stewardship over all Core and Vault contracts.
- [Guardian Angels](./contracts/guardian-angels/guardian-angels-cw3) - Creates a flexible base platform to:
    1. **Execute Restricted Actions:** Allows an Endowment's Owner(in the [Endowment Owners Group](./contracts/guardian-angels/cw4-group)) the ability to create a proposal requesting for the liquidation of their Endowment or other special actions that requires approval from members of the [AP Team Group](./contracts/guardian-angels/cw4-group).
    2. **Social Recovery:** The ability for Endowment Owners to propose updating a optional list of wallet addresses to their Endowment contract, known as "Guardians". If populated, this list Guardians can be invoked should the Owner lose access to their signing key or has it compromised. Guardians may propose to change the owner of the Endowment to an address under the Owner's control (only after approval by N/2+1 majority).

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
  cosmwasm/workspace-optimizer:0.12.1
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.
