use angel_core::structs::{
    AccountStrategies, Allowances, BalanceInfo, Beneficiary, Categories, DonationsReceived,
    EndowmentStatus, EndowmentType, Investments, OneOffVaults, RebalanceDetails,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Env, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr, // DANO/AP Team Address
    pub registrar_contract: Addr,
    pub next_account_id: u32,
    pub max_general_category_id: u8,
}

#[cw_serde]
pub struct OldEndowment {
    pub owner: Addr,            // address that originally setup the endowment account
    pub name: String,           // name of the Endowment
    pub categories: Categories, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub tier: Option<u8>, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub endow_type: EndowmentType,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub status: EndowmentStatus,
    pub deposit_approved: bool, // approved to receive donations & transact
    pub withdraw_approved: bool, // approved to withdraw funds
    pub maturity_time: Option<u64>, // datetime int of endowment maturity (unit: seconds)
    pub strategies: AccountStrategies, // vaults and percentages for locked/liquid accounts donations where auto_invest == TRUE
    pub oneoff_vaults: OneOffVaults, // vaults not covered in account startegies (more efficient tracking of vaults vs. looking up allll vaults)
    pub rebalance: RebalanceDetails, // parameters to guide rebalancing & harvesting of gains from locked/liquid accounts
    pub kyc_donors_only: bool, // allow owner to state a preference for receiving only kyc'd donations (where possible)
    pub pending_redemptions: u8, // number of vault redemptions currently pending for this endowment
    pub proposal_link: Option<u64>, // link back the CW3 Proposal that created an endowment
    pub referral_id: Option<u32>, // at time of creation, the Endowment ID that referred them can be noted, fixed value
}

#[cw_serde]
pub struct Endowment {
    pub owner: Addr,            // address that originally setup the endowment account
    pub name: String,           // name of the Endowment
    pub categories: Categories, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub tier: Option<u8>, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub endow_type: EndowmentType,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub status: EndowmentStatus,
    pub deposit_approved: bool, // approved to receive donations & transact
    pub withdraw_approved: bool, // approved to withdraw funds
    pub maturity_time: Option<u64>, // datetime int of endowment maturity (unit: seconds)
    pub invested_strategies: Investments, // list of strategies that an endowment has invested in
    pub rebalance: RebalanceDetails, // parameters to guide rebalancing & harvesting of gains from locked/liquid accounts
    pub kyc_donors_only: bool, // allow owner to state a preference for receiving only kyc'd donations (where possible)
    pub pending_redemptions: u8, // number of vault redemptions currently pending for this endowment
    pub proposal_link: Option<u64>, // link back the CW3 Proposal that created an endowment
    pub referral_id: Option<u32>, // at time of creation, the Endowment ID that referred them can be noted, fixed value
}

impl Endowment {
    pub fn is_expired(&self, env: &Env) -> bool {
        if let Some(maturity_time) = self.maturity_time {
            if env.block.time > Timestamp::from_seconds(maturity_time) {
                return true;
            }
        }
        false
    }
}

#[cw_serde]
pub struct State {
    pub donations_received: DonationsReceived,
    pub balances: BalanceInfo,
    pub closing_endowment: bool,
    pub closing_beneficiary: Option<Beneficiary>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATES: Map<u32, State> = Map::new("states");
pub const ENDOWMENTS: Map<u32, Endowment> = Map::new("endowments");
pub const ALLOWANCES: Map<(&Addr, &Addr), Allowances> = Map::new("allowances");
