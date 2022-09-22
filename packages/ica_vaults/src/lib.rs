mod checks;
mod ibc_msg;
pub mod ica_controller_msg;
pub mod ica_host_msg;
pub mod utils;

use cosmwasm_std::IbcOrder;

pub use crate::checks::{check_order, check_version, SimpleIcaError};
pub use crate::ibc_msg::{
    BalancesResponse, DispatchResponse, IbcQueryResponse, PacketMsg, ReceiveIbcResponseMsg, StdAck,
    WhoAmIResponse,
};

pub const IBC_APP_VERSION: &str = "ica-vaults-v1";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
// we use this for tests to ensure it is rejected
pub const BAD_APP_ORDER: IbcOrder = IbcOrder::Ordered;
