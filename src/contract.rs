use std::os::macos::raw::stat;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, wasm_execute, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, Uint128,
};
use cw2::set_contract_version;
use cw_ica_controller::types::callbacks::IcaControllerCallbackMsg;
use cw_ica_controller::types::query_msg::IcaQueryResult;
use cw_utils::nonpayable;
use kujira::{KujiraMsg, KujiraQuery};
use mars_types::credit_manager::{Action, ActionCoin, ExecuteMsg as CreditManagerExecuteMsg};

use crate::config::{Config, ConfigResponse};
use crate::error::ContractError;
use crate::handler::channels::{try_close_channel, try_create_channel};
use crate::handler::create_vault::try_create_vault;
use crate::handler::deposit::try_deposit;
use crate::handler::ica::{execute_ica, extract_packet_memo};
use crate::handler::move_funds::try_move_funds;
use crate::handler::try_withdraw;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{self, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "mars_controller";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<KujiraQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<KujiraMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // let contract = env.contract.address;
    // let code_id = msg.cw_ica_controller_code_id;
    // let code_info = deps.querier.query_wasm_code_info(code_id)?;

    // Initialize the contract configuration
    let config = Config::new(msg.clone());

    // TODO: Initialize the ICA controller for each outpost using instantiate2 and update the address in state
    // let mut cw_ica_controller_msgs = vec![];

    // for outpost in config.outposts {
    //     // Initialize the outpost configuration
    //     let controller_init_msg = cw_ica_controller::types::msg::InstantiateMsg {
    //         owner: Some(msg.owner.to_string()),
    //         channel_open_init_options: outpost.channel_open_init_options,
    //         send_callbacks_to: Some(contract.to_string()),
    //     };

    //     let label = format!("NAMI ICA Controller - Mars {}", outpost.chain);
    //     let salt = salt.unwrap_or(env.block.time.seconds().to_string());

    //     let creator_cannonical = deps.api.addr_canonicalize(env.contract.address.as_str())?;

    //     let contract_addr = deps.api.addr_humanize(&instantiate2_address(
    //         &code_info.checksum,
    //         &creator_cannonical,
    //         salt.as_bytes(),
    //     )?)?;

    //     let instantiate_msg = WasmMsg::Instantiate2 {
    //         code_id,
    //         msg: to_json_binary(&controller_init_msg)?,
    //         funds: vec![],
    //         label: label.into(),
    //         admin: Some(env.contract.address.to_string()),
    //         salt: salt.as_bytes().into(),
    //     };

    //     cw_ica_controller_msgs.push(instantiate_msg);
    // }

    config.save(deps.storage, deps.api)?;

    Ok(
        Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("owner", config.owner.to_string()), // .add_messages(cw_ica_controller_msgs)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<KujiraQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<KujiraMsg>, ContractError> {
    let mut config = Config::load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    // Create IBC queries to calculate current total fund amount. React on the callback data.

    let response = match msg {
        ExecuteMsg::Deposit {} => try_deposit(&config),
        ExecuteMsg::Withdraw(msg) => {
            nonpayable(&info)?;
            try_withdraw(msg)
        }
        ExecuteMsg::CreateVault {} => {
            ensure!(config.owner == info.sender, ContractError::Unauthorized {});
            try_create_vault()
        }
        ExecuteMsg::CreateChannel {} => {
            ensure!(config.owner == info.sender, ContractError::Unauthorized {});
            try_create_channel()
        }
        ExecuteMsg::CloseChannel {} => {
            ensure!(config.owner == info.sender, ContractError::Unauthorized {});
            try_close_channel()
        }
        ExecuteMsg::MoveFunds(msg) => {
            ensure!(config.owner == info.sender, ContractError::Unauthorized {});
            // Ensure no funds are sent
            nonpayable(&info)?;

            try_move_funds(msg)
        }
        ExecuteMsg::ReceiveIcaCallback(msg) => {
            match msg {
                IcaControllerCallbackMsg::OnChannelOpenAckCallback {
                    channel,
                    ica_address,
                    tx_encoding,
                } => !unimplemented!("OnChannelOpenAckCallback"),
                IcaControllerCallbackMsg::OnTimeoutPacketCallback {
                    original_packet,
                    relayer,
                } => !unimplemented!("OnTimeoutPacketCallback"),
                IcaControllerCallbackMsg::OnAcknowledgementPacketCallback {
                    ica_acknowledgement: _,
                    original_packet,
                    relayer: _,
                    query_result,
                } => {
                    // Based on the memo the packet was sent with, we can determine the action to take

                    let packet_memo = extract_packet_memo(&original_packet)?;
                    if let Some(memo) = packet_memo {
                        match memo {
                            memo if memo.starts_with("deposit") => {
                                let parts: Vec<&str> = memo.split('/').collect();

                                match parts.as_slice() {
                                    ["deposit", address, denom, amount] => {
                                        // Now you have all variables
                                        // address: &str - the address
                                        // denom: &str - the denomination
                                        // amount: &str - the amount
                                    }
                                    _ => {
                                        // Handle invalid format
                                        return Err(ContractError::InvalidMemoFormat(
                                            memo.to_string(),
                                        ));
                                    }
                                }
                                // update the state using query result
                                let stable_amount: Uint128 =
                                    if let Some(query_result) = query_result {
                                        match query_result {
                                            IcaQueryResult::Success { responses, .. } => {
                                                // check for the right response
                                            }
                                            IcaQueryResult::Error(error) => {}
                                        }
                                    };
                                state.total_stables += stable_amount;
                                state.deposit_redemption_rate =
                                    Decimal::from_ratio(state.total_stables, state.virtual_receipt);

                                // create new SendCosmosMsgs to call the red_bank contract
                                let cw_ica_controller =
                                    config.outposts[0].cw_ica_controller_addr.clone();
                                let mars_address_chain =
                                    config.outposts[0].mars_address_chain.clone();
                                let coin = Coin {
                                    denom: "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4".to_string(),
                                    amount: Uint128::from(1000000u128),
                                };
                                let funds = vec![coin];

                                let update_credit_msg = wasm_execute(
                                    mars_address_chain,
                                    &CreditManagerExecuteMsg::UpdateCreditAccount {
                                        account_id: (),
                                        account_kind: (),
                                        actions: vec![
                                            Action::Deposit(coin),
                                            Action::Lend(ActionCoin::from(coin)),
                                        ],
                                    },
                                    funds,
                                )?;

                                let msg = execute_ica(
                                    cw_ica_controller,
                                    None,
                                    vec![update_credit_msg],
                                    vec![],
                                )?;

                                Ok(Response::new().add_message(msg))
                            }
                            memo if memo.starts_with("withdraw") => !unimplemented!("withdraw"),
                            memo if memo.starts_with("move_funds") => !unimplemented!("move_funds"),
                            _ => Err(ContractError::UnknownMemo {}),
                        }
                        // ica_callback_execute(msg, &mut state, &mut config, deps.api)?;
                    } else {
                        Err(ContractError::UnknownMemo {})
                    }
                }
            }
        }
        ExecuteMsg::UpdateConfig(msg) => {
            ensure!(
                info.sender.clone() == config.owner,
                ContractError::Unauthorized {}
            );
            nonpayable(&info)?;

            config.apply_update(msg, deps.api)?;
            config.save(deps.storage, deps.api)?;
            config.validate(deps.api)?;

            Ok(Response::new().add_attribute("method", "update_config"))
        }
    }?;

    STATE.save(deps.storage, &state)?;

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps<KujiraQuery>, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let config = Config::load(_deps.storage)?;
    Ok(match msg {
        QueryMsg::Config {} => to_json_binary(&ConfigResponse::from(config)),
    }?)
}

#[cfg(test)]
mod tests {}
