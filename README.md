# eth_rpc

[![Test](https://github.com/copiumnicus/eth_rpc/actions/workflows/test.yaml/badge.svg)](https://github.com/copiumnicus/eth_rpc/actions/workflows/test.yaml)

## Features

eth_rpc is a simple, portable eth rpc (partial) implementation with features like:

- [x] Helper methods: token symbol, token decimals, ethers-rs abigen compatible eth_call, get function for [revm](https://github.com/bluealloy/revm) `AccountInfo` model
- [x] transports like: Ipc, Http, EnvHttp, RandomizeHttp (for that sweet multiple node provider links setup under the hood)
- [x] Config serialization support
- [x] Most common tx submission errors
- [x] [json rpc batch](https://sajya.github.io/docs/batch/)
- [x] Sync (take back control over your thread runtime and run it on linux threads not on tokio)
- [x] Rate limit (for alchemy for instance) and network error protection

## (Incomplete) Features Overview

All calls under the hood (EXCEPT TX SUBMISSION FOR FULL CONTROL) use no ratelimit rpc

```rust
pub fn no_ratelimit_rpc<R>(&self, jr: JRCall) -> Result<R, JRError>
where
    R: for<'a> Deserialize<'a>,
```

The abigen compatible eth call:

```rust
/// simple wrapper on top of eth call to work with abigen! macro
pub fn eth_call_typed<R>(&self, to: H160, calldata: impl AbiEncode) -> Result<R, JRError>
where
    R: AbiDecode,
```

The json rpc (chunked) batching compatibility:

```rust
/// in case of error returns first JRError encountered
pub fn batch(&self, requests: Vec<JRCall>) -> Result<Vec<SafeJRResult>, JRError>
```

Erc20 helpers:

```rust
pub fn get_balance(&self, token: H160, account: H160) -> Result<U256, JRError>;
pub fn get_decimals(&self, token: H160) -> Result<u8, JRError>;
pub fn get_symbol(&self, token: H160) -> Result<String, JRError>;
/// for MKR token
pub fn get_bytes32_symbol(&self, token: H160) -> Result<String, JRError>;
```

Custom tx submit error (will handle more based on upstream demand)
```rust
#[derive(Debug)]
pub enum SubmitTxError {
    JRErr(JRError),
    NonceTooLow,
    ReplacementUnderpriced,
    BaseGasPriceTooLow(String),
}
```

Example rpc serialized (json) config (uses all https randomly under the hood):

```json
{
  "batch_chunk_size": 5,
  "transport": {
    "RandomizeHttps": [
      "https://omniscient-few-darkness.quiknode.pro/fake0/",
      "https://eth-mainnet.g.alchemy.com/v2/fake1",
      "https://eth-mainnet.g.alchemy.com/v2/fake2",
      "https://eth-mainnet.g.alchemy.com/v2/fake3"
    ]
  }
}
```

Functions for:
- [X] eth_blockNumber
- [X] eth_estimateGas
- [X] eth_gasPrice
- [X] eth_getBlockByNumber (using ethers Block type)
- [X] eth_getLogs (using custom log type)
- [X] eth_getStorageAt
- [X] eth_getTransactionByHash
- [X] eth_getTransactionCount
- [X] eth_getTransactionReceipt (using ethers TransactionReceipt)

## Testing

Populate the `source_env.sh` based on `source_env_example.sh` and run the tests with `sh ./regression.sh`
