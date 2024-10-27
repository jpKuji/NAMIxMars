use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct State {
    /// Total funds in the contract in USD (6 decimals)
    pub total_stables: Uint128,
    /// Total minted receipt tokens (6 decimals)
    pub virtual_receipt: Uint128,
    /// Redemption rate of receipt tokens to underlying assets
    pub deposit_redemption_rate: Decimal,
}

/// Storage for the State
pub const STATE: Item<State> = Item::new("state");
