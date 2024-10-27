use crate::{error::ContractError, msg::WithdrawMsg};
use cosmwasm_std::Response;
use kujira::KujiraMsg;

/// Responsible for withdrawing the user's funds from the target chain lending contract and sending it back to the user.
/// Validates the user's right to withdraw.
/// Then prepares the red_bank message and calls the corresponding CW ICA Controller to execute the deposit on the target chain.
/// The users virtual receipt tokens are then burned.
pub fn try_withdraw(_msg: WithdrawMsg) -> Result<Response<KujiraMsg>, ContractError> {
    // Check if the user has the right amount of virtual receipt tokens.

    // Check the state where to withdraw the money from -> higher utilization outcome
    unimplemented!()
}
