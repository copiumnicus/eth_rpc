use super::{EthRpc, JRCall, JRError};
use ethers::types::{Block, H256, U256};

impl EthRpc {
    pub fn get_block_by_number(&self, number: U256) -> Result<Block<H256>, JRError> {
        let payload = JRCall::new(
            "eth_getBlockByNumber",
            vec![JRCall::to_value(number)?, JRCall::to_value(false)?],
        )?;
        self.no_ratelimit_rpc(payload)
    }
    pub fn get_latest_block(&self) -> Result<Block<H256>, JRError> {
        let payload = JRCall::new(
            "eth_getBlockByNumber",
            vec![
                JRCall::to_value(String::from("latest"))?,
                JRCall::to_value(false)?,
            ],
        )?;
        self.no_ratelimit_rpc(payload)
    }
    pub fn get_pending_block(&self) -> Result<Block<H256>, JRError> {
        let payload = JRCall::new(
            "eth_getBlockByNumber",
            vec![
                JRCall::to_value(String::from("pending"))?,
                JRCall::to_value(false)?,
            ],
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
        client.get_pending_block().unwrap();
        client.get_latest_block().unwrap();
    }
}
