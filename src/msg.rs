use crate::config::ConfigResponse;
use crate::config::ConfigUpdate;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::to_json_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::Binary;
use cosmwasm_std::StdResult;
use cosmwasm_std::Uint128;
use cw_ica_controller::types::callbacks::IcaControllerCallbackMsg;
use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub outposts: Vec<Outpost>,
    pub cw_ica_controller_code_id: u64,
}

#[cw_serde]
pub struct Outpost {
    pub mars_red_bank_contract: String,
    pub cw_ica_controller_contract: String,
    pub channel_open_init_options: ChannelOpenInitOptions,
    pub account_id: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Initiate a deposit from the user to a mars vault on an outpost. This msg will only dispatch an ica query.
    Deposit(DepositMsg),
    // Initiate a withdraw from the user to a mars vault on an outpost. This msg will only dispatch an ica query.
    Withdraw(WithdrawMsg),
    // Create a new mars vault on an outpost using the credit-vault functionality.
    CreateVault {},
    // Create a new channel with the ICA controller on an outpost.
    CreateChannel {},
    // Close a channel with the ICA controller on an outpost.
    CloseChannel {},
    // Lend / Unlend assets in a mars vault on an outpost.
    MoveFunds(MoveFundsMsg),
    /// The callback message from the ICA controller contract.
    ReceiveIcaCallback(IcaControllerCallbackMsg),
    // Update the contract configuration by the owner
    UpdateConfig(ConfigUpdate),
}

#[cw_serde]
pub struct DepositMsg {
    pub destination: String,
}

#[cw_serde]
pub struct WithdrawMsg {
    pub amount: Uint128,
}

#[cw_serde]
pub struct MoveFundsMsg {
    /// Type of action: set_idle or deploy
    pub action: Action,
    /// Denom of the base token
    pub denom: String,
    /// Amount to withdraw from the ghost vault
    pub amount: Uint128,
    pub chain: String,
}

#[cw_serde]
pub enum Action {
    On,
    Off,
}

#[cw_serde]
pub enum CallbackType {}

impl CallbackType {
    pub fn to_json_binary(&self) -> StdResult<Binary> {
        to_json_binary(&self)
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct Positions {
    pub account_id: String,
}

// Create Credit Account
#[cw_serde]
pub struct CreateCreditAccount(AccountKind);
