use std::time::Duration;

use super::{
    jr_call::JRCall,
    rpc::{EthRpc, JRError, SafeJRResult},
};
use itertools::Itertools;
use serde_json::Value;
use tracing::{debug, error, warn};

impl EthRpc {
    /// in case of error returns first JRError encountered
    pub fn batch(&self, requests: Vec<JRCall>) -> Result<Vec<SafeJRResult>, JRError> {
        // since its sync can have nice iterators again
        let res: Result<Vec<Vec<SafeJRResult>>, JRError> = requests
            .into_iter()
            .chunks(self.batch_chunk_size.unwrap_or(20_000))
            .into_iter()
            .map(|chunk| {
                let chunk: Vec<_> = chunk.into_iter().collect();
                if self.disable_ratelimit_protection {
                    debug!("DR: Getting batch chunk: {}.", chunk.len());
                    return self.dr_batch_chunk(chunk);
                }
                debug!("Getting batch chunk: {}.", chunk.len());
                self.no_ratelimit_batch_chunk(chunk)
            })
            .collect();
        // Note: responses length is checked in #no_ratelimit_batch_chunk
        Ok(res?
            .into_iter()
            .flatten()
            .sorted_by(|a, b| a.id.cmp(&b.id))
            .collect())
    }
    /// no ratelimit and no network errors
    fn no_ratelimit_batch_chunk(
        &self,
        requests: Vec<JRCall>,
    ) -> Result<Vec<SafeJRResult>, JRError> {
        let requests_len = requests.len();
        let request = serde_json::to_vec(&requests).map_err(|e| JRError::JRCallSerialize(e))?;
        let mut backoff_sec = 1;
        let mut count = 0;
        loop {
            count += 1;
            let res = self.batch_chunk(request.as_slice());
            if let Err(e) = res {
                if count > 4 {
                    error!("Retries fail: {:?}", e);
                    return Err(e);
                }
                if e.is_network_or_ratelimit() {
                    warn!("Batch rpc: network err or rate limited!");
                    std::thread::sleep(Duration::from_secs(backoff_sec));
                    backoff_sec += 1;
                    continue;
                }
                // error!("{:?}", e);
                return Err(e);
            }

            let res = res?;

            if res.len() != requests_len {
                return Err(JRError::BatchMissingResponses);
            }

            return Ok(res);
        }
    }
    fn dr_batch_chunk(&self, requests: Vec<JRCall>) -> Result<Vec<SafeJRResult>, JRError> {
        let request = serde_json::to_vec(&requests).map_err(|e| JRError::JRCallSerialize(e))?;
        self.batch_chunk(request.as_slice())
    }
    /// chunk is subset of batch
    fn batch_chunk(&self, requests: &[u8]) -> Result<Vec<SafeJRResult>, JRError> {
        // we call the underlying because it is rpc batch so it returns vec instead of value
        let response: Vec<Value> = self.__call_transport(requests)?;
        response.into_iter().map(|a| a.try_into()).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_batch() {
        let client = EthRpc::from_env().unwrap();
        let requests = vec![
            JRCall::new_with_id("eth_blockNumber", Vec::new() as Vec<()>, 0).unwrap(),
            JRCall::new_with_id(
                "eth_getBalance",
                vec!["0x407d73d8a49eeb85d32cf465507dd71d507100c1", "latest"],
                1,
            )
            .unwrap(),
        ];
        let results = client.batch(requests).unwrap();
        println!("{:#?}", results);
    }
}
