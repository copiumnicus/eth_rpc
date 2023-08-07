use super::{JRCall, EthRpc, JRError};
use ethers::types::{Block, H256, U256};

impl EthRpc {
    pub fn get_block_by_number(&self, number: U256) -> Result<Block<H256>, JRError> {
        let payload = JRCall::new(
            "eth_getBlockByNumber",
            vec![JRCall::to_value(number)?, JRCall::to_value(false)?],
        )?;
        self.no_ratelimit_rpc(payload)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_block_by_number() {
        let client = EthRpc::from_env().unwrap();
        let res = client.get_block_by_number(U256::from(17633288)).unwrap();
        assert_eq!(res.size, Some(U256::from(312244)));
    }
}
