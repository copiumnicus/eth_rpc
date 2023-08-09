use super::{EthRpc, JRCall, JRError};
use crate::SafeJRResult;
use ethers::types::U256;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// {"jsonrpc":"2.0","id":83,"result":{"currentBlock":"0x110706e",
// "healedBytecodeBytes":"0x0","healedBytecodes":"0x0","healedTrienodeBytes":"0x0",
// "healedTrienodes":"0x0","healingBytecode":"0x0","healingTrienodes":"0x0",
// "highestBlock":"0x110c8d0","startingBlock":"0x110c8b1",
// "syncedAccountBytes":"0x0","syncedAccounts":"0x0","syncedBytecodeBytes":"0x0","syncedBytecodes":"0x0","syncedStorage":"0x0","syncedStorageBytes":"0x0"}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GethSyncingRes {
    #[serde(rename = "currentBlock")]
    pub current_block: U256,
    #[serde(rename = "highestBlock")]
    pub highest_block: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    NotSyncing,
    Syncing,
    SyncingGeth(GethSyncingRes),
}

impl SyncStatus {
    pub fn is_syncing(&self) -> bool {
        if matches!(self, Self::NotSyncing) {
            return false;
        }
        true
    }
}

impl EthRpc {
    /// node is syncing, some nodes wont tell you the truth tho for different blockchains
    pub fn is_syncing(&self) -> Result<SyncStatus, JRError> {
        let payload = JRCall::new("eth_syncing", vec![] as Vec<()>)?;
        let request = payload.to_vec()?;
        let request = request.as_slice();
        // if node crashed for instance you don't want to protect this request
        let result: Value = self.__call_transport(request)?;
        let safe_jr_result: SafeJRResult = result.try_into()?;
        let value = safe_jr_result.result;

        if let Ok(bool) = serde_json::from_value::<bool>(value) {
            return Ok(if bool {
                SyncStatus::Syncing
            } else {
                SyncStatus::NotSyncing
            });
        }

        let result: Value = self.__call_transport(request)?;
        let safe_jr_result: SafeJRResult = result.try_into()?;
        let value = safe_jr_result.result;

        let geth = serde_json::from_value::<GethSyncingRes>(value)
            .map_err(|e| JRError::ResponseDoesNotMatchType(e))?;
        Ok(SyncStatus::SyncingGeth(geth))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_syncing() {
        let client = EthRpc::from_env().unwrap();
        let res = client.is_syncing().unwrap();
        println!("{:#?}", res);
        // port forward a geth node
        let local_client = EthRpc::with_http("http://127.0.0.1:8545").unwrap();
        // it works trust me :)
        println!("{:#?}", local_client.is_syncing())
        // NotSyncing
        // Ok(
        //     SyncingGeth(
        //         GethSyncingRes {
        //             current_block: 17859693,
        //             highest_block: 17877200,
        //         },
        //     ),
        // )
    }
}
