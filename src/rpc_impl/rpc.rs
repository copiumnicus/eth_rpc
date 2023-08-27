use super::jr_call::JRCall;
use crate::{
    transport::{RpcTransport, RpcTransportErr},
    EnvHttp, HttpErr,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fmt::Debug, ops::Deref, time::Duration};
use tracing::{error, warn};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JRRes {
    pub result: Option<Value>,
    pub error: Option<Value>,
    #[serde(with = "super::safe_id")]
    pub id: u64,
}

/// after all safety checks
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SafeJRResult {
    pub result: Value,
    pub id: u64,
}

impl SafeJRResult {
    pub fn try_deserialize<T>(self) -> Result<T, JRError>
    where
        T: for<'a> Deserialize<'a>,
    {
        serde_json::from_value(self.result).map_err(|e| JRError::ResponseDoesNotMatchType(e))
    }
}

impl TryFrom<Value> for SafeJRResult {
    type Error = JRError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // then parse as json rpc response
        let result: JRRes =
            serde_json::from_value(value).map_err(|e| JRError::ResponseNotJsonRpcResponse(e))?;

        // check if result has error
        if let Some(e) = result.error {
            // catch alchemy ratelimit
            if let Ok(s) = serde_json::to_string(&e) {
                if s.contains("exceeded its compute units") {
                    return Err(JRError::RateLimited(e));
                }
            }
            return Err(JRError::JsonRpcResultError(e));
        }
        // check if result exists
        if result.result.is_none() {
            return Err(JRError::EmptyResponse);
        }
        // return result
        Ok(Self {
            result: result.result.unwrap(),
            id: result.id,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EthRpc {
    pub transport: RpcTransport,
    pub batch_chunk_size: Option<usize>,
    #[serde(default)]
    pub disable_ratelimit_protection: bool,
}

#[derive(Debug)]
pub enum JRError {
    FailInitialize(String),
    JRCallSerialize(serde_json::Error),
    Transport(RpcTransportErr),
    ResponseNotUtf8(std::string::FromUtf8Error),
    ResponseNotJson(serde_json::Error),
    ResponseNotJsonRpcResponse(serde_json::Error),
    /// not guaranteed to catch the ratelimit error
    /// it might masquarade as some other error here
    RateLimited(Value),

    ResponseDoesNotMatchType(serde_json::Error),

    JsonRpcResultError(Value),
    BatchMissingResponses,
    EmptyResponse,

    Extension(String),
}

impl JRError {
    pub fn is_network_or_ratelimit(&self) -> bool {
        if matches!(self, Self::RateLimited(_)) {
            return true;
        }
        if let Self::Transport(e) = &self {
            // failed in network io
            if e.is_http_network() {
                return true;
            }
            // don't handle ipc because its implementation is uncertain :)
        }
        false
    }
}

impl EthRpc {
    pub fn from_env() -> Result<Self, JRError> {
        let http = EnvHttp::http().map_err(|e| JRError::Transport(e))?;
        Ok(Self {
            disable_ratelimit_protection: false,
            batch_chunk_size: Some(20_000),
            transport: RpcTransport::Http(http),
        })
    }
    pub fn with_http(http: impl ToString) -> Result<Self, JRError> {
        Ok(Self {
            disable_ratelimit_protection: false,
            batch_chunk_size: Some(20_000),
            transport: RpcTransport::with_http(http)?,
        })
    }
    pub fn no_ratelimit_rpc<R>(&self, jr: JRCall) -> Result<R, JRError>
    where
        R: for<'a> Deserialize<'a>,
    {
        let params = jr.to_vec()?;
        if self.disable_ratelimit_protection {
            return self.call_rpc_transport(params.as_slice());
        }
        self.inner_no_ratelimit_rpc(params.as_slice())
    }
    /// no ratelimit and no network errors
    pub(crate) fn inner_no_ratelimit_rpc<R>(&self, params: &[u8]) -> Result<R, JRError>
    where
        R: for<'a> Deserialize<'a>,
    {
        let mut backoff_sec = 1;
        let mut count = 0;
        loop {
            count += 1;
            let res = self.call_rpc_transport(params);
            if let Err(e) = res {
                if count > 4 {
                    error!("Retries fail: {:?}", e);
                    return Err(e);
                }
                if e.is_network_or_ratelimit() {
                    warn!("Network err or rate limited!");
                    std::thread::sleep(Duration::from_secs(backoff_sec));
                    backoff_sec += 1;
                    continue;
                }
                error!("{:?}", e);
                return Err(e);
            }

            return res;
        }
    }

    /// unwraps jsonrpc response into a desired type
    pub(crate) fn call_rpc_transport<R>(&self, request: &[u8]) -> Result<R, JRError>
    where
        R: for<'a> Deserialize<'a>,
    {
        let result: Value = self.__call_transport(request)?;
        let safe_jr_result: SafeJRResult = result.try_into()?;
        let value = safe_jr_result.result;
        Ok(serde_json::from_value(value).map_err(|e| JRError::ResponseDoesNotMatchType(e))?)
    }

    /// single rpc call casting ret bytes into any type
    pub(crate) fn __call_transport<R>(&self, request: &[u8]) -> Result<R, JRError>
    where
        R: for<'a> Deserialize<'a>,
    {
        let res = match self.transport.send(request) {
            Ok(v) => v,
            Err(e) => {
                if let RpcTransportErr::Http(http_err) = &e {
                    if let HttpErr::FailStatus(_, status) = &http_err {
                        if status.deref() == &429 {
                            return Err(JRError::RateLimited(json!(429)));
                        }
                    }
                }
                return Err(JRError::Transport(e));
            }
        };
        serde_json::from_slice(res.as_slice()).map_err(|e| JRError::ResponseNotJson(e))
    }
}
