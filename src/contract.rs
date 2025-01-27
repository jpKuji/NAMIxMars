#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, instantiate2_address, to_json_binary, wasm_execute, Binary, Coin, Decimal, Deps,
    DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_ica_controller::types::{callbacks::IcaControllerCallbackMsg, query_msg::IcaQueryResult};
use cw_utils::nonpayable;
use kujira::{KujiraMsg, KujiraQuery};
use mars_types::credit_manager::{Action, ActionCoin, ExecuteMsg as CreditManagerExecuteMsg};

use crate::config::{Config, ConfigResponse};
use crate::error::ContractError;
use crate::handler::{
    channels::{try_close_channel, try_create_channel},
    create_vault::try_create_vault,
    deposit::try_deposit,
    ica::{execute_ica, extract_packet_memo},
    move_funds::try_move_funds,
    try_withdraw,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{self, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "mars_controller";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<KujiraQuery>,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<KujiraMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let code_id = msg.cw_ica_controller_code_id;
    let code_info = deps.querier.query_wasm_code_info(code_id)?;

    // Create mutable config to update outpost addresses
    let mut config = Config::new(msg.clone());
    let mut cw_ica_controller_msgs = vec![];

    // Update each outpost with its predicted address and create instantiate messages
    for outpost in &mut config.outposts {
        // Initialize the outpost configuration
        let controller_init_msg = cw_ica_controller::types::msg::InstantiateMsg {
            owner: Some(msg.owner.to_string()),
            channel_open_init_options: outpost.channel_open_init_options.clone(),
            send_callbacks_to: Some(env.contract.address.to_string()),
        };

        let label = format!(
            "NAMI ICA Controller - Mars {}",
            outpost.mars_red_bank_contract
        );
        let salt = format!(
            "{}_{}",
            outpost.mars_red_bank_contract,
            env.block.time.seconds()
        );

        let creator_cannonical = deps.api.addr_canonicalize(env.contract.address.as_str())?;
        // Convert checksum to bytes
        let checksum_bytes = code_info.checksum.as_slice();

        let predicted_contract_addr =
            instantiate2_address(checksum_bytes, &creator_cannonical, salt.as_bytes()).unwrap();

        // Update the outpost with the predicted address
        outpost.cw_ica_controller_contract = deps
            .api
            .addr_humanize(&predicted_contract_addr)?
            .to_string();

        let instantiate_msg = WasmMsg::Instantiate2 {
            code_id,
            msg: to_json_binary(&controller_init_msg)?,
            funds: vec![],
            label: label.into(),
            admin: Some(env.contract.address.to_string()),
            salt: salt.as_bytes().into(),
        };

        cw_ica_controller_msgs.push(instantiate_msg);
    }

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

    let response = match msg {
        ExecuteMsg::Deposit(msg) => try_deposit(&config, msg.destination),
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
                                    ["deposit", address, denom, amount, destination] => {
                                        let stable_amount: Uint128 = if let Some(query_result) =
                                            query_result
                                        {
                                            match query_result {
                                                IcaQueryResult::Success { responses, .. } => {
                                                    // let response = responses.get(0).unwrap();
                                                    // let amount = response.amount.clone();
                                                    // amount
                                                    Uint128::zero()
                                                }
                                                IcaQueryResult::Error(error) => {
                                                    return Err(ContractError::IcaQueryError {
                                                        error: error.to_string(),
                                                    });
                                                }
                                            }
                                        } else {
                                            Uint128::zero()
                                        };
                                        state.total_stables += stable_amount;
                                        state.deposit_redemption_rate = Decimal::from_ratio(
                                            state.total_stables,
                                            state.virtual_receipt,
                                        );

                                        // create new SendCosmosMsgs to call the red_bank contract
                                        let coin = Coin {
                                            denom: denom.to_string(),
                                            amount: amount
                                                .parse::<u128>()
                                                .map(Uint128::new)
                                                .map_err(|_| ContractError::InvalidAmount {})?,
                                        };
                                        let funds = vec![coin];

                                        let outpost = config
                                            .find_destination_outpost(destination)
                                            .ok_or(ContractError::DestinationNotFound {
                                                destination: destination.to_string(),
                                            })?;

                                        let update_credit_msg = wasm_execute(
                                            outpost.mars_red_bank_contract.clone(),
                                            &CreditManagerExecuteMsg::UpdateCreditAccount {
                                                account_id: None,
                                                account_kind: None,
                                                actions: vec![
                                                    Action::Deposit(coin),
                                                    Action::Lend(ActionCoin::from(coin)),
                                                ],
                                            },
                                            funds,
                                        )?
                                        .into();

                                        let msg = execute_ica(
                                            outpost.cw_ica_controller_contract.clone(),
                                            None,
                                            vec![update_credit_msg],
                                            vec![],
                                        )?;

                                        Ok(Response::new().add_message(msg))
                                    }
                                    _ => {
                                        // Handle invalid format
                                        return Err(ContractError::InvalidMemoFormat(
                                            memo.to_string(),
                                        ));
                                    }
                                }
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
