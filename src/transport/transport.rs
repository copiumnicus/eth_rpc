use super::{
    env_http::EnvHttp,
    http::{HttpErr, HttpTransport},
    ipc::{IpcConfig, IpcError},
};
use crate::JRError;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RpcTransport {
    Ipc(IpcConfig),
    Http(HttpTransport),
    RandomizeHttps(Vec<HttpTransport>),
    EnvHttp(EnvHttp),
}

#[derive(Debug)]
pub enum RpcTransportErr {
    Ipc(IpcError),
    Http(HttpErr),
    FailedToGetEnv(String),
    NoHttps,
}

impl RpcTransportErr {
    pub fn is_http_network(&self) -> bool {
        if let Self::Http(e) = &self {
            if let HttpErr::IO(_) = e {
                return true;
            }
        }
        false
    }
}

impl RpcTransport {
    pub fn with_http(http: impl ToString) -> Result<Self, JRError> {
        Ok(RpcTransport::Http(
            http.to_string()
                .try_into()
                .map_err(|e| JRError::FailInitialize(e))?,
        ))
    }

    pub fn send(&self, params: &[u8]) -> Result<Vec<u8>, RpcTransportErr> {
        let http = match self {
            Self::EnvHttp(v) => {
                // have to return here because env returns obj not reference
                return v
                    .get_http()?
                    .post(params)
                    .map_err(|e| RpcTransportErr::Http(e));
            }
            // if ipc return
            Self::Ipc(ipc) => return ipc.send_ipc(params).map_err(|e| RpcTransportErr::Ipc(e)),
            // else keep http
            Self::RandomizeHttps(https) => {
                if https.len() == 0 {
                    return Err(RpcTransportErr::NoHttps);
                }
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..https.len());
                &https[idx]
            }
            Self::Http(http) => &http,
        };
        http.post(params).map_err(|e| RpcTransportErr::Http(e))
    }
}
