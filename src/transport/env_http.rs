use crate::{HttpTransport, RpcTransportErr};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvHttp {
    #[serde(default = "default_var")]
    #[serde(deserialize_with = "de_validate_env_http")]
    pub env_var_name: String,
}

// don't impl default directly not to leak the datastructure without checking for env being populated
fn default_var() -> String {
    "ETH_RPC_HTTP".into()
}

fn de_validate_env_http<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Cow::<str>::deserialize(deserializer)?;
    // check that env has http_url populated
    {
        let maybe_http_url: String = std::env::var((&s).to_string()).map_err(D::Error::custom)?;
        // check that you can get http transport from that env var
        HttpTransport::new(&maybe_http_url).map_err(D::Error::custom)?;
    }
    Ok(s.into())
}

fn validate_env_http(env_key: &str) -> Result<(), String> {
    let maybe_http_url: String =
        std::env::var((env_key).to_string()).map_err(|e| format!("{:?}", e))?;
    // check that you can get http transport from that env var
    HttpTransport::new(&maybe_http_url)?;
    Ok(())
}

impl EnvHttp {
    /// try to get from `ETH_RPC_HTTP`, used in tests
    pub fn http() -> Result<HttpTransport, RpcTransportErr> {
        let env_http = EnvHttp::new_custom(default_var().as_str())?;
        env_http.get_http()
    }
    /// defaults to `ETH_RPC_HTTP`
    pub fn new() -> Result<Self, RpcTransportErr> {
        EnvHttp::new_custom(default_var().as_str())
    }
    pub fn new_custom(env_var_name: &str) -> Result<Self, RpcTransportErr> {
        validate_env_http(env_var_name).map_err(|e| RpcTransportErr::FailedToGetEnv(e))?;
        let env_var_name = env_var_name.to_string();
        Ok(Self { env_var_name })
    }
    pub(crate) fn get_http(&self) -> Result<HttpTransport, RpcTransportErr> {
        let s = std::env::var(self.env_var_name.clone())
            .map_err(|e| RpcTransportErr::FailedToGetEnv(format!("{}", e)))?;
        HttpTransport::new(s.as_str())
            .map_err(|e| RpcTransportErr::FailedToGetEnv(format!("{}", e)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_env_http() {
        // using default env var name
        assert!(serde_json::from_value::<EnvHttp>(json!({})).is_ok());
        // using random name verify that it fails at deserialization stage and does not pretend to have access to some url
        assert!(serde_json::from_value::<EnvHttp>(json!({
            "env_var_name": "asefihasoief"
        }))
        .is_err());
    }
}
