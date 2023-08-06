use super::{JRCall, JRClient, JRError};
use ethers::types::U256;

impl JRClient {
    pub fn get_block_number(&self) -> Result<u64, JRError> {
        let payload = JRCall::new("eth_blockNumber", Vec::new() as Vec<()>)?;
        let value: U256 = self.no_ratelimit_rpc(payload)?;
        Ok(value.as_u64())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_block_number() {
        let client = JRClient::from_env().unwrap();
        let res = client.get_block_number().unwrap();
        println!("{:#?}", res)
    }
}
