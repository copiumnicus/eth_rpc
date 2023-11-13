use crate::{EthRpc, JRCall, JRError};
use ethers::types::{H160, U256};
use revm::{
    interpreter::analysis::to_analysed,
    primitives::{ruint::aliases::B160, AccountInfo, Bytecode, Bytes, B256},
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
/// hex prefixed code hash
fn hash_code(s: &str) -> Result<[u8; 32], String> {
    let v: Vec<u8> = crate::hex::to_vec(s)?;
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
            let bytes = Bytes::from(crate::hex::to_vec(x).map_err(|e| JRError::Extension(e))?);
            let bytecode = to_analysed(Bytecode::new_raw(bytes));
            Some(bytecode)
        }
    };
    Ok((code_hash, code))
}

impl EthRpc {
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
            balance: ethers_to_alloy(balance),
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
            source: B160::from_be_bytes(address.0),
            balance: ethers_to_alloy(balance),
            nonce: nonce.as_u64(),
        })
    }
}

fn ethers_to_alloy(u: U256) -> revm::primitives::alloy_primitives::U256 {
    let mut bytes = [0; 32];
    u.to_big_endian(&mut bytes);
    revm::primitives::alloy_primitives::U256::from_be_slice(bytes.as_slice())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_account_info() {
        let client = EthRpc::from_env().unwrap();
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
        assert_eq!(
            serde_json::to_string(&res.balance).unwrap(),
            // 18.252325471589916
            "\"0xfd4d494fbaee5d33\"".to_string()
        );
        let res = client.get_account_partial(settlement, block).unwrap();
        println!("{:?}", res);
        // if serializes correctly the value is valid too
        assert_eq!(
            serde_json::to_string(&res.source).unwrap(),
            "\"0x9008d19f58aabd9ed0d60971565aa8510560ab41\"".to_string()
        );
        assert_eq!(
            serde_json::to_string(&res.balance).unwrap(),
            "\"0x173fbe57973ed5e\"".to_string()
        );
    }
}
