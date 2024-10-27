use crate::error::ContractError;
use cosmwasm_std::Response;
use kujira::KujiraMsg;

/// Creates a credit account on the mars outpost on the target chain.
/// Only callable by the contract owner.
pub fn try_create_vault() -> Result<Response<KujiraMsg>, ContractError> {
    // Prepare the red_bank deposit message to the credit account
    // Prepare the cw execute Msg to the right cw_ica_controller based on the chain
    !unimplemented!()
}
