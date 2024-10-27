use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Response, StdResult, Uint128};
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

pub struct VirtualReceipts(Map<Addr, Uint128>);

impl VirtualReceipts {
    pub fn new() -> Self {
        Self(VIRTUAL_RECEIPTS)
    }

    pub fn store(&self, deps: DepsMut, address: Addr, amount: Uint128) -> StdResult<Response> {
        self.0.save(deps.storage, address, &amount)?;
        Ok(Response::new())
    }

    pub fn get(&self, deps: Deps, address: Addr) -> StdResult<Uint128> {
        self.0.load(deps.storage, address)
    }

    pub fn update(&self, deps: DepsMut, address: Addr, amount: Uint128) -> StdResult<Response> {
        self.0
            .update(deps.storage, address, |existing| -> StdResult<Uint128> {
                let current = existing.unwrap_or(Uint128::zero());
                Ok(current + amount)
            })?;
        Ok(Response::new())
    }

    pub fn query(&self, deps: Deps, address: Addr) -> StdResult<Uint128> {
        Ok(self
            .0
            .load(deps.storage, address)
            .unwrap_or(Uint128::zero()))
    }
}
