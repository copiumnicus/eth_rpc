use super::{JRCall, JRClient, JRError};
use ethers::types::{H160, U256};

impl JRClient {
    pub fn get_storage_at(&self, address: H160, index: U256, block: u64) -> Result<U256, JRError> {
        let payload = JRCall::new(
            "eth_getStorageAt",
            vec![
                format!("{:?}", address),
                format!("0x{:x}", index),
                format!("0x{:x}", block),
            ],
        )?;
        self.no_ratelimit_rpc(payload)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_storage() {
        let client = JRClient::from_env().unwrap();
        let res = client
            .get_storage_at(
                "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"
                    .parse()
                    .unwrap(),
                U256::from(5),
                17588244,
            )
            .unwrap();
        println!("{:#?}", res)
    }
}
