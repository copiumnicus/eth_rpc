use crate::{JRCall, JRClient, JRError};
use ethers::{
    abi::{AbiDecode, AbiEncode},
    types::{transaction::eip2718::TypedTransaction, Bytes, H160, U256},
};

impl JRClient {
    pub fn eth_call(&self, tx: TypedTransaction) -> Result<Bytes, JRError> {
        let payload = JRCall::new(
            "eth_call",
            vec![
                serde_json::to_value(tx).map_err(|e| JRError::JRCallSerialize(e))?,
                serde_json::to_value("latest").map_err(|e| JRError::JRCallSerialize(e))?,
            ],
        )?;
        let v: Bytes = self.no_ratelimit_rpc(payload)?;
        Ok(v)
    }
    /// simple wrapper on top of eth call to work with abigen! macro
    pub fn eth_call_typed<R>(&self, to: H160, calldata: impl AbiEncode) -> Result<R, JRError>
    where
        R: AbiDecode,
    {
        let input = Bytes::from(calldata.encode());
        let mut tx = TypedTransaction::default();
        tx.set_to(to);
        tx.set_data(input);
        tx.set_value(U256::from(0));
        let bytes = self.eth_call(tx)?;
        Ok(R::decode(&bytes).map_err(|e| JRError::Extension(format!("{:?}", e)))?)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

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
    fn test_eth_call() {
        let client = JRClient::from_env().unwrap();
        let input = Bytes::from(SymbolCall {}.encode());
        let mut tx = TypedTransaction::default();
        tx.set_to(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
                .parse::<H160>()
                .unwrap(),
        );
        tx.set_data(input);

        let res = client.eth_call(tx).unwrap();
        assert_eq!(res, Bytes::from_str("0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000045553444300000000000000000000000000000000000000000000000000000000").unwrap() );
    }
}
