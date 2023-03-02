use cosmwasm_schema::{cw_serde};

use cosmwasm_std::Order;

#[cw_serde]
pub enum OrderBy {
    Asc,
    Desc,
}

impl From<OrderBy> for Order {
    fn from(o: OrderBy) -> Order {
        if o == OrderBy::Asc {
            Order::Ascending
        } else {
            Order::Descending
        }
    }
}
