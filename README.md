# Angel Protocol Smart Contracts
 
## Components

### Core Contracts
- [Accounts](./contracts/accounts) - Implementation of the Charity Endowment Accounts. 
- [Registrar](./contracts/registrar) - Contracts for the creation and management of Endowment Accounts smart contracts
core platform of smart contracts that support multiple verticals of specialized smart contracts. 
- [Index Fund](./contracts/index-fund) - Contract that acts as a gateway for donors and Terra Charity Alliance members to donate to a groups of charitites as a single Index Fund (grouped by UN SDGs).

### Gateway Contracts
- [Vaults](./contracts/vaults) - Vaults allow endowments to invest their funds into various TeFi/DeFi protocols to earn yield, based on their Strategy allocations.  


## Getting setup locally to develop:

### Requirements: 
- [LocalTerra](https://github.com/terra-project/localterra)(main branch)
- Rust
- Cargo

To keep code styles consistant, please install and use the code linter, [Rustfmt]().
