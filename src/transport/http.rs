use oxhttp::{
    model::{InvalidHeader, Method, Request, Status, Url},
    Client,
};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use tracing::error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpTransport(#[serde(deserialize_with = "deserialize_http_url")] pub String);

impl TryFrom<String> for HttpTransport {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Url::parse(value.as_str()).map_err(|e| format!("{:?}", e))?;
        Ok(HttpTransport(value))
    }
}

fn deserialize_http_url<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Cow::<str>::deserialize(deserializer)?;
    if let Err(e) = Url::parse(&s).map_err(D::Error::custom) {
        println!("Failed to deserialize url. Got `{}`", s);
        error!("Failed to deserialize url. Got `{}`", s);
        return Err(e);
    }
    Ok(s.into())
}

#[derive(Debug)]
pub enum HttpErr {
    FailedToAddHeader(InvalidHeader),
    IO(std::io::Error),
    BodyIO(std::io::Error),
    FailStatus(String, Status),
    Other(String),
}

impl HttpTransport {
    pub fn post(&self, params: &[u8]) -> Result<Vec<u8>, HttpErr> {
        let client = Client::new();
        // have to copy here
        let params = params.to_vec();
        let req = Request::builder(
            Method::POST,
            self.0
                // safe to unwrap because serde deserialize checks parsing
                .parse()
                .unwrap(),
        )
        .with_header("content-type", "application/json")
        .map_err(|e| HttpErr::FailedToAddHeader(e))?
        .with_body(params);
        let res = client.request(req).map_err(|e| HttpErr::IO(e))?;

        let status = res.status();
        if !status.is_successful() {
            return Err(HttpErr::FailStatus(self.0.clone(), status));
        }
        let res = res.into_body().to_vec().map_err(|e| HttpErr::BodyIO(e))?;

        Ok(res)
    }

    pub fn get(&self) -> Result<Vec<u8>, HttpErr> {
        let client = Client::new();
        let req = Request::builder(
            Method::GET,
            self.0
                // safe to unwrap because serde deserialize checks parsing
                .parse()
                .unwrap(),
        )
        .build();
        let res = client.request(req).map_err(|e| HttpErr::IO(e))?;

        let status = res.status();
        if !status.is_successful() {
            return Err(HttpErr::FailStatus(self.0.clone(), status));
        }
        let res = res.into_body().to_vec().map_err(|e| HttpErr::BodyIO(e))?;

        Ok(res)
    }

    pub(crate) fn new(http: &str) -> Result<Self, String> {
        Url::parse(http).map_err(|e| {
            format!(
                "Failed to parse http transport url = {} err = {:?}",
                http, e
            )
        })?;
        Ok(Self(http.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::EnvHttp;
    use serde_json::json;

    #[test]
    fn test_deserialize() {
        assert!(HttpTransport::new("").is_err());
        println!("{:?}", HttpTransport::new("https://etherscan.io/"));
        assert!(HttpTransport::new("https://etherscan.io/").is_ok());
    }

    #[test]
    fn test_request() {
        let transport = EnvHttp::http().unwrap();
        let result = transport
            .post(
                serde_json::to_vec(&json!({
                    "jsonrpc":"2.0",
                    "method":"eth_getTransactionReceipt",
                    "params":[
                        "0xb2fea9c4b24775af6990237aa90228e5e092c56bdaee74496992a53c208da1ee"
                    ],
                    "id":1
                }))
                .unwrap()
                .as_slice(),
            )
            .unwrap();
        let res_str = String::from_utf8(result).unwrap();
        println!("{:?}", res_str);
    }
}
