# eth_rpc

[![Test](https://github.com/copiumnicus/eth_rpc/actions/workflows/test.yaml/badge.svg)](https://github.com/copiumnicus/eth_rpc/actions/workflows/test.yaml)

## Features

eth_rpc is a simple, portable eth rpc (partial) implementation with features like:
- [X] Helper methods: token symbol, token decimals, get function for [revm](https://github.com/bluealloy/revm) `AccountInfo` model
- [X] transports like: Ipc, Http, EnvHttp, RandomizeHttp (for that sweet multiple node provider links setup under the hood)
- [X] Config serialization support 
- [X] Most common tx submission errors
- [X] [json rpc batch](https://sajya.github.io/docs/batch/)
- [X] Sync (take back control over your thread runtime and run it on linux threads not on tokio)
- [X] Rate limit (for alchemy for instance) and network error protection

## Testing

Populate the `source_env.sh` based on `source_env_example.sh` and run the tests with `sh ./regression.sh` 
