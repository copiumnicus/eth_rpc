use super::rpc::JRError;
use serde::Serialize;
use serde_json::Value;

/// this needs to be like that for batch rpc calls to work
#[derive(Debug, Clone, Serialize)]
pub struct JRCall {
    #[serde(with = "super::safe_id")]
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
    pub(crate) fn to_vec(&self) -> Result<Vec<u8>, JRError> {
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
