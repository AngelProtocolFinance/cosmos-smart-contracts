use crate::curves::{decimal, Constant, Curve, DecimalPlaces, Linear, SquareRoot};
use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// name of the supply token
    pub name: String,
    /// symbol / ticker of the supply token
    pub symbol: String,
    /// number of decimal places of the supply token, needed for proper curve math.
    /// If it is eg. HALO, where a balance of 10^6 means 1 HALO, then use 6 here.
    pub decimals: u8,
    /// this is the cw20 reserve token address
    /// For Charity Shares, this is the address of the HALO CW20 Contract
    pub reserve_denom: String,
    /// number of decimal places for the reserve token, needed for proper curve math.
    /// Same format as decimals above, eg. if it is uatom, where 1 unit is 10^-6 ATOM, use 6 here
    pub reserve_decimals: u8,
    /// enum to store the curve parameters used for this contract
    /// if you want to add a custom Curve, you should make a new contract that imports this one.
    /// write a custom `instantiate`, and then dispatch `your::execute` -> `cw20_bonding::do_execute`
    /// with your custom curve as a parameter (and same with `query` -> `do_query`)
    pub curve_type: CurveType,
    // days of unbonding
    pub unbonding_period: u64,
}

pub type CurveFn = Box<dyn Fn(DecimalPlaces) -> Box<dyn Curve>>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CurveType {
    /// Constant always returns `value * 10^-scale` as spot price
    Constant { value: Uint128, scale: u32 },
    /// Linear returns `slope * 10^-scale * supply` as spot price
    Linear { slope: Uint128, scale: u32 },
    /// SquareRoot returns `slope * 10^-scale * supply^(power)` as spot price
    SquareRoot {
        slope: Uint128,
        power: Uint128,
        scale: u32,
    },
}

impl CurveType {
    pub fn to_curve_fn(&self) -> CurveFn {
        match self.clone() {
            CurveType::Constant { value, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(Constant::new(decimal(value, scale), places))
                };
                Box::new(calc)
            }
            CurveType::Linear { slope, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(Linear::new(decimal(slope, scale), places))
                };
                Box::new(calc)
            }
            CurveType::SquareRoot {
                slope,
                power,
                scale,
            } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(SquareRoot::new(
                        decimal(slope, scale),
                        decimal(power, scale),
                        places,
                    ))
                };
                Box::new(calc)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Buy will attempt to purchase as many supply tokens as possible.
    /// You must send only CW20 reserve tokens (HALO)
    Buy {},
    /// DonorMatch will attempt to receive the CW20 reserve tokens (HALO).
    /// It will also attempt to send the dao tokens (CS) to "donor" & "endowment" contract.
    /// You must send only CW20 reserve tokens (HALO)
    DonorMatch {
        amount: Uint128,
        donor: String,
        accounts_contract: String,
        endowment_id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Buy will attempt to purchase as many supply tokens as possible.
    /// You must send only reserve tokens in that message
    // Buy {},
    /// Implements CW20. Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Implements CW20. Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Implements CW20.  Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Implements CW20 "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
    /// Claim all tokens available for the message sender
    ClaimTokens {},
    // Implements CW20. Receive is a base message to receive tokens to a this contract and trigger an action
    /// on the receiving contract.
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the reserve and supply quantities, as well as the spot price to buy 1 token
    CurveInfo {},
    /// Implements CW20. Returns the current balance of the given address, 0 if unset.
    Balance { address: String },
    /// Implements CW20. Returns metadata on the contract - name, decimals, supply, etc.
    TokenInfo {},
    /// Implements CW20 "allowance" extension.
    /// Returns how much spender can use from owner account, 0 if unset.
    Allowance { owner: String, spender: String },
    /// Returns claims for an address
    Claims { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurveInfoResponse {
    // how many reserve tokens have been received
    pub reserve: Uint128,
    // how many supply tokens have been issued
    pub supply: Uint128,
    pub spot_price: Decimal,
    pub reserve_denom: String,
}
