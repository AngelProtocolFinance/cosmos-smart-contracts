# Governance

The Gov Contract contains logic for holding polls and Halo Token (HALO) staking, and allows the Angel Protocol to be governed by its users in a decentralized manner. After the initial bootstrapping of Angel Protocol contracts, the Gov Contract is assigned to be the owner of itself and other contracts.

New proposals for change are submitted as polls, and are voted on by ANC stakers through the voting procedure. Polls can contain messages that can be executed directly without changing the Angel Protocol code.

The Gov Contract keeps a balance of ANC tokens, which it uses to reward stakers with funds it receives from trading fees sent by the Halo Collector and user deposits from creating new governance polls. This balance is separate from the Community Pool, which is held by the Community contract (owned by the Gov contract).
