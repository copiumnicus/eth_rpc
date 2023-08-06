use super::{JRCall, JRClient, JRError};
use ethers::types::{H160, U256};

impl JRClient {
    pub fn get_transaction_count(&self, address: H160) -> Result<u64, JRError> {
        let payload = JRCall::new(
            "eth_getTransactionCount",
            vec![format!("{:?}", address), format!("latest")],
        )?;
        let v: U256 = self.no_ratelimit_rpc(payload)?;
        Ok(v.as_u64())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_nonce() {
        let client = JRClient::from_env().unwrap();
        let res = client
            .get_transaction_count(
                "0xD8b9c8e1a94baEAaf4D1CA2C45723eb88236130E"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        println!("{:#?}", res)
    }
}
