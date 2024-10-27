use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::{Item, Map};

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

/// Map to hold the virtual receipts of each user to account for deposits.
pub const VIRTUAL_RECEIPTS: Map<Addr, Uint128> = Map::new("virtual_receipts");
impl VIRTUAL_RECEIPTS {
    pub fn store(deps: DepsMut, address: Addr, amount: Uint128) -> StdResult<Response> {
        VIRTUAL_RECEIPTS.save(deps.storage, address, &amount)?;
        Ok(Response::new())
    }

    pub fn get(deps: Deps, address: Addr) -> StdResult<Uint128> {
        VIRTUAL_RECEIPTS
            .load(deps.storage, address)
            .unwrap_or(Uint128::zero())
    }

    pub fn update(deps: DepsMut, address: Addr, amount: Uint128) -> StdResult<Response> {
        VIRTUAL_RECEIPTS.update(deps.storage, address, |existing| -> StdResult<Uint128> {
            let current = existing.unwrap_or(Uint128::zero());
            Ok(current + amount)
        })?;
        Ok(Response::new())
    }

    pub fn query(deps: Deps, address: Addr) -> StdResult<Uint128> {
        Ok(VIRTUAL_RECEIPTS
            .load(deps.storage, address)
            .unwrap_or(Uint128::zero()))
    }
}
