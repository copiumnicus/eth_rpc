use super::{JRCall, EthRpc, JRError};
use ethers::types::{TransactionReceipt, H256};

impl EthRpc {
    pub fn get_transaction_receipt(&self, hash: H256) -> Result<TransactionReceipt, JRError> {
        let payload = JRCall::new("eth_getTransactionReceipt", vec![hash])?;
        let v: TransactionReceipt = self.no_ratelimit_rpc(payload)?;
        Ok(v)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ethers::types::U256;

    #[test]
    fn test_get_receipt() {
        let client = EthRpc::from_env().unwrap();
        let res = client
            .get_transaction_receipt(
                "0x6b7b84c2474ba1df72487c1b69ddc78496a938913b1b71d66974b96cc168fa83"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(res.cumulative_gas_used, U256::from(1819398));
    }
}
