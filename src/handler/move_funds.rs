use crate::{error::ContractError, msg::MoveFundsMsg};
use cosmwasm_std::Response;
use kujira::KujiraMsg;

pub fn try_move_funds(_msg: MoveFundsMsg) -> Result<Response<KujiraMsg>, ContractError> {
    unimplemented!()
}
