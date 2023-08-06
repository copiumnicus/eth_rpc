#!/bin/sh
# source the `ETH_RPC_HTTP` var for tests 
source ./source_env.sh
# run tests show logs
cargo test -- --nocapture