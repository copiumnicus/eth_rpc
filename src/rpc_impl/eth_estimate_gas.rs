use super::{JRCall, JRClient, JRError};
use ethers::types::{transaction::eip2718::TypedTransaction, U256};

impl JRClient {
    pub fn estimate_gas(&self, tx: TypedTransaction) -> Result<u64, JRError> {
        let payload = JRCall::new("eth_estimateGas", vec![tx])?;
        let v: U256 = self.no_ratelimit_rpc(payload)?;
        Ok(v.as_u64())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ethers::{
        abi::AbiEncode,
        prelude::abigen,
        types::{Bytes, H160},
    };

    abigen!(
        _Erc20,
        r#"[
        function symbol() external view returns (string)
    ]"#
    );

    #[test]
    fn test_estimate_gas() {
        let client = JRClient::from_env().unwrap();

        let input = Bytes::from(SymbolCall {}.encode());
        let mut tx = TypedTransaction::default();
        tx.set_to(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
                .parse::<H160>()
                .unwrap(),
        );
        tx.set_data(input);

        let res = client.estimate_gas(tx).unwrap();
        assert_eq!(res, 31434);
    }
}
