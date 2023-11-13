use super::{
    jr_call::JRCall,
    rpc::{EthRpc, JRError, SafeJRResult},
};
use itertools::Itertools;
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, error, warn};

impl EthRpc {
    /// returns all results that did not throw err on rpc level
    pub fn batch_collect_err(
        &self,
        requests: Vec<JRCall>,
    ) -> Result<(Vec<SafeJRResult>, Vec<JRError>), JRError> {
        let res: Result<Vec<(Vec<SafeJRResult>, Vec<JRError>)>, JRError> = requests
            .into_iter()
            .chunks(self.batch_chunk_size.unwrap_or(20_000))
            .into_iter()
            .map(|chunk| {
                let chunk: Vec<_> = chunk.into_iter().collect();
                if self.disable_ratelimit_protection {
                    debug!("DR: Getting batch chunk collect err: {}.", chunk.len());
                    return self.dr_batch_chunk(chunk);
                }
                debug!("Getting batch chunk collect err: {}.", chunk.len());
                self.no_ratelimit_batch_chunk(chunk)
            })
            .collect();
        let mut result = Vec::new();
        let mut errs = Vec::new();
        for (res, es) in res? {
            result.extend(res);
            errs.extend(es);
        }

        Ok((
            result
                .into_iter()
                .sorted_by(|a, b| a.id.cmp(&b.id))
                .collect(),
            errs,
        ))
    }
    /// in case of error returns first JRError encountered
    pub fn batch(&self, requests: Vec<JRCall>) -> Result<Vec<SafeJRResult>, JRError> {
        let req_len = requests.len();
        // since its sync can have nice iterators again
        let res: Result<Vec<(Vec<SafeJRResult>, Vec<JRError>)>, JRError> = requests
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
        let mut result = Vec::new();
        for (res, errs) in res? {
            for e in errs {
                // if any err was caught return it instead
                return Err(e);
            }
            result.extend(res)
        }
        // check for silent errs
        if req_len != result.len() {
            return Err(JRError::BatchMissingResponses);
        }
        // Note: responses length is checked in #no_ratelimit_batch_chunk
        Ok(result
            .into_iter()
            .sorted_by(|a, b| a.id.cmp(&b.id))
            .collect())
    }
    /// no ratelimit and no network errors
    fn no_ratelimit_batch_chunk(
        &self,
        requests: Vec<JRCall>,
    ) -> Result<(Vec<SafeJRResult>, Vec<JRError>), JRError> {
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

            return res;
        }
    }
    fn dr_batch_chunk(
        &self,
        requests: Vec<JRCall>,
    ) -> Result<(Vec<SafeJRResult>, Vec<JRError>), JRError> {
        let request = serde_json::to_vec(&requests).map_err(|e| JRError::JRCallSerialize(e))?;
        self.batch_chunk(request.as_slice())
    }
    /// chunk is subset of batch
    fn batch_chunk(&self, requests: &[u8]) -> Result<(Vec<SafeJRResult>, Vec<JRError>), JRError> {
        // we call the underlying because it is rpc batch so it returns vec instead of value
        let response: Vec<Value> = self.__call_transport(requests)?;
        let mut res: Vec<SafeJRResult> = Vec::new();
        let mut errs = Vec::new();
        for r in response {
            match r.try_into() {
                Ok(v) => res.push(v),
                Err(e) => errs.push(e),
            }
        }
        Ok((res, errs))
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

    #[test]
    fn test_batch_collect_err() {
        let mut client = EthRpc::from_env().unwrap();
        client.batch_chunk_size = Some(2);
        let requests = vec![
            JRCall::new_with_id("eth_blockNumber", Vec::new() as Vec<()>, 0).unwrap(),
            JRCall::new_with_id(
                "eth_getBalance",
                // notice invalid address
                vec!["0x407d73d8a49eeb85d07100c1", "latest"],
                1,
            )
            .unwrap(),
            JRCall::new_with_id(
                "eth_getBalance",
                vec!["0x407d73d8a49eeb85d32cf465507dd71d507100c1", "latest"],
                2,
            )
            .unwrap(),
        ];
        let results = client.batch_collect_err(requests).unwrap();
        println!("{:#?}", results);
    }
}
