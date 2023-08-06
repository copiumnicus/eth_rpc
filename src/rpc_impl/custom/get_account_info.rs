use crate::{JRCall, JRClient, JRError};
use ethers::types::{H160, U256};
use revm::{
    interpreter::analysis::to_analysed,
    primitives::{AccountInfo, Bytecode, Bytes, B160, B256},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tiny_keccak::{Hasher, Keccak};

// TODO: extract these dependencies to a separate crate
fn keccak256(slice: Vec<u8>) -> [u8; 32] {
    let mut h = Keccak::v256();
    h.update(slice.as_slice());
    let mut first_key = [0; 32];
    h.finalize(&mut first_key);
    first_key
}
/// convert hex str to a vec of bytes
fn to_vec(mut s: &str) -> Result<Vec<u8>, String> {
    if s.starts_with("0x") {
        s = &s[2..]
    }
    if s.len() % 2 != 0 {
        return Err(format!(
            "Given hex str, is not zero mod 2! len: {:?}",
            s.len()
        ));
    }
    Ok((0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect())
}
/// hex prefixed code hash
fn hash_code(s: &str) -> Result<[u8; 32], String> {
    let v: Vec<u8> = to_vec(s)?;
    Ok(keccak256(v))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountPartial {
    pub source: B160,
    pub balance: revm::primitives::U256,
    pub nonce: u64,
}

fn account_params(address: H160, block: u64) -> Result<Vec<Value>, JRError> {
    // U256 is crucial on block value as it serializes as hex
    Ok(vec![
        JRCall::to_value(address)?,
        JRCall::to_value(U256::from(block))?,
    ])
}

pub fn get_code_hash_and_code(code: String) -> Result<(B256, Option<Bytecode>), JRError> {
    let code_hash: B256 = B256::from(hash_code(code.as_str()).map_err(|e| JRError::Extension(e))?);
    let code = match code.as_ref() {
        "0x" => None,
        x => {
            let bytes = Bytes::from(to_vec(x).map_err(|e| JRError::Extension(e))?);
            let bytecode = to_analysed(Bytecode::new_raw(bytes));
            Some(bytecode)
        }
    };
    Ok((code_hash, code))
}

impl JRClient {
    pub fn get_account_info(&self, address: H160, block: u64) -> Result<AccountInfo, JRError> {
        let params = account_params(address, block)?;
        // the length is already checked
        let mut result = self.batch(vec![
            JRCall::new_with_id("eth_getBalance", params.clone(), 0)?,
            JRCall::new_with_id("eth_getTransactionCount", params.clone(), 1)?,
            JRCall::new_with_id("eth_getCode", params, 2)?,
        ])?;
        let balance: U256 = result.remove(0).try_deserialize()?;
        let nonce: U256 = result.remove(0).try_deserialize()?;
        let code: String = result.remove(0).try_deserialize()?;
        let (code_hash, code) = get_code_hash_and_code(code)?;

        Ok(AccountInfo {
            balance: balance.into(),
            nonce: nonce.as_u64(),
            code_hash: code_hash.into(),
            code,
        })
    }

    pub fn get_account_partial(
        &self,
        address: H160,
        block: u64,
    ) -> Result<AccountPartial, JRError> {
        let params = account_params(address, block)?;
        // the length is already checked
        let mut result = self.batch(vec![
            JRCall::new_with_id("eth_getBalance", params.clone(), 0)?,
            JRCall::new_with_id("eth_getTransactionCount", params.clone(), 1)?,
        ])?;
        let balance: U256 = result.remove(0).try_deserialize()?;
        let nonce: U256 = result.remove(0).try_deserialize()?;

        Ok(AccountPartial {
            source: address.into(),
            balance: balance.into(),
            nonce: nonce.as_u64(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_account_info() {
        let client = JRClient::from_env().unwrap();
        let settlement = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41"
            .parse()
            .unwrap();
        let solver = "0xD8b9c8e1a94baEAaf4D1CA2C45723eb88236130E"
            .parse()
            .unwrap();
        let block = 17613178;
        let res = client.get_account_info(settlement, block).unwrap();
        println!("{:?}", res);
        let res = client.get_account_info(solver, block).unwrap();
        println!("{:?}", res);
        let res = client.get_account_partial(solver, block).unwrap();
        println!("{:?}", res);
        let res = client.get_account_partial(settlement, block).unwrap();
        println!("{:?}", res)
    }
}
