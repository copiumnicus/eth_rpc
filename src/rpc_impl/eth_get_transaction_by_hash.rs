use super::{JRCall, JRClient, JRError};
use ethers::types::{Transaction, H256};

impl JRClient {
    pub fn get_transaction_by_hash(&self, hash: H256) -> Result<Transaction, JRError> {
        let payload = JRCall::new("eth_getTransactionByHash", vec![hash])?;
        self.no_ratelimit_rpc(payload)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_transaction() {
        let client = JRClient::from_env().unwrap();
        let res = client
            .get_transaction_by_hash(
                "0x6b7b84c2474ba1df72487c1b69ddc78496a938913b1b71d66974b96cc168fa83"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        println!("{:#?}", res)
    }
}
