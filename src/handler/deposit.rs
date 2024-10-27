use crate::{config::Config, error::ContractError, msg::Positions};
use cosmwasm_std::{to_json_binary, Response, WasmQuery};
use kujira::KujiraMsg;

use super::ica::query_ica;

/// Creates the correct red_bank deposit message based on the user input and
/// calls the corresponding CW ICA Controller to execute the deposit on the target chain.
/// The user is then credited an amount of virtual receipt tokens.
pub fn try_deposit(
    config: &Config,
    destination: String,
) -> Result<Response<KujiraMsg>, ContractError> {
    // Check if a vault exists on the target chain.
    let outpost = config
        .find_destination_outpost(&destination)
        .ok_or(ContractError::DestinationNotFound { destination })?;

    // Check if account_id exists
    let account_id = outpost
        .account_id
        .as_ref()
        .cloned()
        .ok_or(ContractError::NoCreditAccount {})?;

    let query = cosmwasm_std::QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: outpost.mars_red_bank_contract.clone(),
        msg: to_json_binary(&Positions { account_id })?,
    });
    let msg = query_ica(
        outpost.cw_ica_controller_contract.clone(),
        Some("deposit".to_string()),
        vec![query],
    )?;

    Ok(Response::new().add_message(msg))
}
