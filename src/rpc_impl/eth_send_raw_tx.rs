use super::{JRCall, JRClient, JRError};
use ethers::types::{Bytes, H256};

#[derive(Debug)]
pub enum SubmitTxError {
    JRErr(JRError),
    NonceTooLow,
    ReplacementUnderpriced,
    BaseGasPriceTooLow(String),
    Other(String),
}

impl JRClient {
    pub fn send_raw_tx(&self, bytes: Bytes) -> Result<H256, SubmitTxError> {
        let payload = JRCall::new("eth_sendRawTransaction", vec![bytes])
            .map_err(|e| SubmitTxError::JRErr(e))?;
        match self.call_rpc_transport(
            payload
                .to_vec()
                .map_err(|e| SubmitTxError::JRErr(e))?
                .as_slice(),
        ) {
            Ok(v) => Ok(v),
            Err(e) => {
                if let JRError::JsonRpcResultError(v) = &e {
                    let stringified = serde_json::to_string(v).unwrap();
                    if stringified.contains("nonce too low") {
                        return Err(SubmitTxError::NonceTooLow);
                    }
                    if stringified.contains("replacement transaction underpriced") {
                        return Err(SubmitTxError::ReplacementUnderpriced);
                    }
                    if stringified.contains("max fee per gas less than block base fee") {
                        return Err(SubmitTxError::BaseGasPriceTooLow(stringified));
                    }
                }
                return Err(SubmitTxError::JRErr(e));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_nonce() {
        let client = JRClient::from_env().unwrap();
        let res = client
            .get_transaction_count(
                "0xD8b9c8e1a94baEAaf4D1CA2C45723eb88236130E"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(res, 2199);
    }
}
