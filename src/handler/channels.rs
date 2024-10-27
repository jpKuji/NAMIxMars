use crate::error::ContractError;
use cosmwasm_std::Response;
use kujira::KujiraMsg;

pub fn try_create_channel() -> Result<Response<KujiraMsg>, ContractError> {
    unimplemented!()
}

pub fn try_close_channel() -> Result<Response<KujiraMsg>, ContractError> {
    unimplemented!()
}
