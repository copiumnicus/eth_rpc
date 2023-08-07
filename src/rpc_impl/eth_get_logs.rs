use super::rpc::{JRClient, JRError};
use crate::rpc_impl::jr_call::JRCall;
use ethers::types::{H160, H256, U256};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GetLogs {
    pub from_block: U256,
    pub to_block: U256,
    pub topics: Vec<H256>,
    pub address: H160,
}

pub mod vec_bytes_hex {
    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::borrow::{Borrow, Cow};

    /// convert vector of bytes to lowercase hex string prefixed with `0x`
    fn to_str(v: Vec<u8>) -> String {
        format!("0x{}", to_str_no_pre(v))
    }

    fn to_str_no_pre(v: Vec<u8>) -> String {
        format!(
            "{}",
            v.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join("")
        )
    }

    pub fn serialize<S>(value: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(to_str(value.clone()).as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Cow::<str>::deserialize(deserializer)?;
        crate::hex::to_vec(s.borrow()).map_err(D::Error::custom)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetLogsEvent {
    /// don't accept `true`
    pub removed: bool,
    pub address: H160,
    pub transaction_hash: H256,
    pub topics: Vec<H256>,
    #[serde(with = "vec_bytes_hex")]
    pub data: Vec<u8>,
    pub block_number: U256,
    pub log_index: U256,
}

impl JRClient {
    pub fn get_logs(
        &self,
        from_block: u64,
        to_block: u64,
        topics: Vec<H256>,
        address: H160,
    ) -> Result<Vec<GetLogsEvent>, JRError> {
        debug!(
            "Fetch logs on range: `{}` - `{}` for `{:?}`",
            from_block, to_block, address
        );
        let payload = JRCall::new(
            "eth_getLogs",
            vec![GetLogs {
                from_block: U256::from(from_block),
                to_block: U256::from(to_block),
                topics,
                address,
            }],
        )?;
        let logs: Vec<GetLogsEvent> = self.no_ratelimit_rpc(payload)?;
        Ok(logs.into_iter().filter(|a| !a.removed).collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_logs() {
        let client = JRClient::from_env().unwrap();
        let result = client
            .get_logs(
                17240728,
                17240744,
                vec![
                    "0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9"
                        .parse()
                        .unwrap(),
                ],
                "0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(result.len(), 1);
    }
}
