use crate::msg::InstantiateMsg;
use crate::{error::ContractError, msg::Outpost};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdResult, Storage};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub owner: Addr,
    pub outposts: Vec<Outpost>,
    pub cw_ica_controller_code_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

impl Config {
    pub fn new(msg: InstantiateMsg) -> Self {
        Self {
            owner: msg.owner,
            outposts: msg.outposts,
            cw_ica_controller_code_id: msg.cw_ica_controller_code_id,
        }
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        CONFIG.load(storage)
    }

    pub fn save(&self, storage: &mut dyn Storage, api: &dyn Api) -> Result<(), ContractError> {
        self.validate(api)?;
        CONFIG.save(storage, self)?;
        Ok(())
    }

    pub fn validate(&self, api: &dyn Api) -> Result<(), ContractError> {
        api.addr_validate(self.owner.as_str())?;

        // Validate each contract address in the outposts
        for outpost in &self.outposts {
            api.addr_validate(outpost.address.as_str())?;
        }

        Ok(())
    }

    pub fn apply_update(&mut self, msg: ConfigUpdate, api: &dyn Api) -> Result<(), ContractError> {
        if let Some(owner) = msg.owner {
            self.owner = owner;
        }

        // Overwrite the complete outpost list
        if let Some(outposts) = msg.outposts {
            self.outposts = outposts;
        }

        if let Some(cw_ica_controller_code_id) = msg.cw_ica_controller_code_id {
            self.cw_ica_controller_code_id = cw_ica_controller_code_id;
        }

        self.validate(api)?;
        Ok(())
    }
}

impl From<Config> for ConfigResponse {
    fn from(config: Config) -> Self {
        Self {
            owner: config.owner,
            outposts: config.outposts,
            cw_ica_controller_code_id: config.cw_ica_controller_code_id,
        }
    }
}

#[cw_serde]
pub struct ConfigUpdate {
    pub owner: Option<Addr>,
    pub outposts: Option<Vec<Outpost>>,
    pub cw_ica_controller_code_id: Option<u64>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
    pub outposts: Vec<Outpost>,
    pub cw_ica_controller_code_id: u64,
}
