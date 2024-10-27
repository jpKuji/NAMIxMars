use crate::error::ContractError;
use cosmwasm_std::{
    from_json, wasm_execute, CosmosMsg, Empty, IbcPacket, QueryRequest, StdResult, WasmMsg,
};
use cw_ica_controller::types::msg::ExecuteMsg::SendCosmosMsgs;
use serde::{Deserialize, Serialize};

pub fn create_ibc_identifier() -> String {
    "ica".to_string()
}

pub fn execute_ica(
    cw_ica_controller_address: String,
    memo: Option<String>,
    messages: Vec<CosmosMsg>,
    queries: Vec<QueryRequest>,
) -> Result<WasmMsg, ContractError> {
    let ica_controller_msg = SendCosmosMsgs {
        messages,
        queries,
        packet_memo: if let Some(memo) = memo {
            Some(memo)
        } else {
            None
        },
        timeout_seconds: None,
    };

    let msg = wasm_execute(cw_ica_controller_address, &ica_controller_msg, vec![])?;

    Ok(msg)
}

pub fn query_ica(
    cw_ica_controller_address: String,
    memo: Option<String>,
    queries: Vec<QueryRequest>,
) -> Result<WasmMsg, ContractError> {
    let ica_controller_msg = SendCosmosMsgs {
        messages: vec![],
        queries,
        packet_memo: if let Some(memo) = memo {
            Some(memo)
        } else {
            None
        },
        timeout_seconds: None,
    };

    let msg = wasm_execute(cw_ica_controller_address, &ica_controller_msg, vec![])?;

    Ok(msg)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IcaPacketData {
    messages: Vec<CosmosMsg>,
    #[serde(default)]
    queries: Vec<QueryRequest<Empty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    packet_memo: Option<String>,
}

pub fn extract_packet_memo(packet: &IbcPacket) -> StdResult<Option<String>> {
    let packet_data: IcaPacketData = from_json(&packet.data)?;
    Ok(packet_data.packet_memo)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{to_json_binary, IbcEndpoint, IbcTimeout, IbcTimeoutBlock};

    use super::*;

    #[test]
    fn test_extract_packet_memo() {
        let ica_packet = IcaPacketData {
            messages: vec![],
            queries: vec![],
            packet_memo: Some("test_memo".to_string()),
        };

        let binary_data = to_json_binary(&ica_packet).unwrap();

        let packet = IbcPacket::new(
            binary_data,
            IbcEndpoint {
                port_id: "port".to_string(),
                channel_id: "channel".to_string(),
            },
            IbcEndpoint {
                port_id: "port".to_string(),
                channel_id: "channel".to_string(),
            },
            1,
            IbcTimeout::with_block(IbcTimeoutBlock {
                height: 1,
                revision: 1,
            }),
        );

        let memo_result = extract_packet_memo(&packet);
        assert!(memo_result.is_ok());
        assert_eq!(memo_result.unwrap(), Some("test_memo".to_string()));
    }
}
