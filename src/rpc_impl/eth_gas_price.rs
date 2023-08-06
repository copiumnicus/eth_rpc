use super::{JRCall, JRClient, JRError};
use ethers::types::U256;

impl JRClient {
    pub fn get_gas_price(&self) -> Result<U256, JRError> {
        let payload = JRCall::new("eth_gasPrice", vec![] as Vec<()>)?;
        self.no_ratelimit_rpc(payload)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_gas_price() {
        let client = JRClient::from_env().unwrap();
        let res = client.get_gas_price().unwrap();
        println!("{:#?}", res)
    }
}
