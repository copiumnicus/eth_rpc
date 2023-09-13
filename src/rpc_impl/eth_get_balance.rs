use crate::{EthRpc, JRCall, JRError};
use ethers::types::{H160, U256};

impl EthRpc {
    pub fn get_latest_balance(&self, target: H160) -> Result<U256, JRError> {
        let payload = JRCall::new(
            "eth_getBalance",
            vec![JRCall::to_value(target)?, JRCall::to_value("latest")?],
        )?;
        self.no_ratelimit_rpc(payload)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_latest_balance() {
        let client = EthRpc::from_env().unwrap();
        let v = client
            .get_latest_balance(
                "0x4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        println!("titan builder balance {}", v);
        assert!(!v.is_zero());
    }
}
