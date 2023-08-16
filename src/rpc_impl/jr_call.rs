use super::rpc::JRError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// this needs to be like that for batch rpc calls to work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JRCall {
    #[serde(with = "super::safe_id")]
    #[serde(default)]
    pub id: u64,
    pub method: String,
    pub params: Value,
    pub jsonrpc: String,
}

impl JRCall {
    pub fn set_id(mut self, id: usize) -> Self {
        self.id = id as u64;
        self
    }
    pub fn to_value<T>(v: T) -> Result<Value, JRError>
    where
        T: Serialize,
    {
        serde_json::to_value(v).map_err(|e| JRError::JRCallSerialize(e))
    }
    pub fn to_vec(&self) -> Result<Vec<u8>, JRError> {
        serde_json::to_vec(&self).map_err(|e| JRError::JRCallSerialize(e))
    }
    pub fn new<T>(method: impl ToString, params: T) -> Result<Self, JRError>
    where
        T: Serialize,
    {
        Self::new_with_id(method, params, 0)
    }
    pub fn new_with_id<T>(method: impl ToString, params: T, id: u64) -> Result<Self, JRError>
    where
        T: Serialize,
    {
        Ok(Self {
            params: serde_json::to_value(params).map_err(|e| JRError::JRCallSerialize(e))?,
            id,
            method: method.to_string(),
            jsonrpc: "2.0".into(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_id() {
        let call: JRCall = serde_json::from_str(
            r#"{"jsonrpc": "2.0", "method": "eth_subscription", "params": {"subscription": "0xc581f553014f3d768aab1b45643fea51", "result": {"chainId": "0x1", "to": "0x709ccbd5c29e0f7bb2c315bb8c6b44dd2c6fc56e", "value": "0xe27c49886e60000", "data": "0x", "accessList": [], "nonce": "0x5f55f", "maxPriorityFeePerGas": "0x0", "maxFeePerGas": "0x5d21dba000", "gas": "0x33450", "type": "0x2", "hash": "0x0becb0e1d4fe219da225590e61475ba8a4f5bb285f45a721f1f7814648a6145c", "from": "0xbf94f0ac752c739f623c463b5210a7fb2cbb420b"}}}"#,
        )
        .unwrap();
        assert_eq!(call.id, 0);
    }
}
