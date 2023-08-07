use super::{JRCall, EthRpc, JRError};
use ethers::types::{Transaction, H256};

impl EthRpc {
    pub fn get_transaction_by_hash(&self, hash: H256) -> Result<Transaction, JRError> {
        let payload = JRCall::new("eth_getTransactionByHash", vec![hash])?;
        self.no_ratelimit_rpc(payload)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ethers::types::U256;

    #[test]
    fn test_get_transaction() {
        let client = EthRpc::from_env().unwrap();
        let res = client
            .get_transaction_by_hash(
                "0x6b7b84c2474ba1df72487c1b69ddc78496a938913b1b71d66974b96cc168fa83"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(res.max_priority_fee_per_gas, Some(U256::from(5148613256u128)));
    }
}
