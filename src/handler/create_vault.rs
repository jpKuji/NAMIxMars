use crate::error::ContractError;
use cosmwasm_std::Response;
use kujira::KujiraMsg;

/// Creates a credit account on the mars outpost on the target chain.
/// Only callable by the contract owner.
pub fn try_create_vault() -> Result<Response<KujiraMsg>, ContractError> {
    // Prepare the red_bank message to create a credit account for the ICA
    // Prepare the SendCosmosMsgs cw_ica_controller ExecMsg to the correct cw_ica_controller
}
