use super::{EthRpc, JRCall, JRError};

impl EthRpc {
    /// node is syncing, some nodes wont tell you the truth tho for different blockchains
    pub fn is_syncing(&self) -> Result<bool, JRError> {
        let payload = JRCall::new("eth_syncing", vec![] as Vec<()>)?;
        // if node crashed for instance you don't want to protect this request
        self.inner_no_ratelimit_rpc(payload.to_vec()?.as_slice())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_syncing() {
        let client = EthRpc::from_env().unwrap();
        let res = client.is_syncing().unwrap();
        println!("{:#?}", res)
    }
}
