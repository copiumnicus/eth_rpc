use crate::{JRClient, JRError};
use ethers::{
    prelude::abigen,
    types::{H160, U256},
};

abigen!(
    _Erc20,
    r#"[
    function decimals() external view returns (uint8)
    function symbol() external view returns (string)
    function balanceOf(address account) external view returns (uint256)
]"#
);
// maker guys...
abigen!(
    _Erc21,
    r#"[
    function symbol() external view returns (bytes32)
]"#
);

impl JRClient {
    pub fn get_balance(&self, token: H160, account: H160) -> Result<U256, JRError> {
        let b: BalanceOfReturn = self.eth_call_typed(token, BalanceOfCall { account })?;
        Ok(b.0)
    }

    pub fn get_symbol(&self, token: H160) -> Result<String, JRError> {
        use erc_20::{SymbolCall, SymbolReturn};
        let s: SymbolReturn = self.eth_call_typed(token, SymbolCall {})?;
        Ok(s.0)
    }

    pub fn get_bytes32_symbol(&self, token: H160) -> Result<String, JRError> {
        use erc_21::{SymbolCall, SymbolReturn};
        let s: SymbolReturn = self.eth_call_typed(token, SymbolCall {})?;
        let result = String::from_utf8_lossy(s.0.as_slice()).to_string();
        Ok(result.trim().trim_end_matches(char::from(0)).to_string())
    }

    pub fn get_decimals(&self, token: H160) -> Result<u8, JRError> {
        let d: DecimalsReturn = self.eth_call_typed(token, DecimalsCall {})?;
        Ok(d.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_erc20_functions() {
        let client = JRClient::from_env().unwrap();
        // maker
        let symbol = client
            .get_bytes32_symbol(
                "0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(symbol, String::from("MKR"));

        let token = "0x3472A5A71965499acd81997a54BBA8D852C6E53d"
            .parse()
            .unwrap();

        let res = client
            .get_balance(
                token,
                "0x9008D19f58AAbD9eD0D60971565AA8510560ab41"
                    .parse()
                    .unwrap(),
            )
            .unwrap();
        println!("{:#?}", res);

        let res = client.get_symbol(token).unwrap();
        println!("{:#?}", res);

        let res = client.get_decimals(token).unwrap();
        println!("{:#?}", res);
    }
}
