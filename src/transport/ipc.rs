use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    time::Duration,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpcConfig {
    pub ipc_path: String,
    pub read_timeout_millis: u64,
}

#[derive(Debug)]
pub enum IpcError {
    IO(std::io::Error),
    EOF,
}

impl IpcConfig {
    pub fn send_ipc(&self, params: &[u8]) -> Result<Vec<u8>, IpcError> {
        let mut stream = UnixStream::connect(self.ipc_path.clone()).map_err(|e| IpcError::IO(e))?;
        stream
            .set_read_timeout(Some(
                Duration::from_millis(self.read_timeout_millis).clone(),
            ))
            .map_err(|e| IpcError::IO(e))?;
        stream.write_all(params).map_err(|e| IpcError::IO(e))?;
        let response = get_ipc_response(&mut stream)?;
        stream
            .shutdown(Shutdown::Both)
            .map_err(|e| IpcError::IO(e))?;
        Ok(response)
    }
}

/// linux kernel pipe read size
const DEFAULT_KERNEL_READ_SIZE: usize = 128 * 1024;

fn get_ipc_response(stream: &mut UnixStream) -> Result<Vec<u8>, IpcError> {
    get_ipc_response_with_read_size::<DEFAULT_KERNEL_READ_SIZE>(stream)
}

fn get_ipc_response_with_read_size<const N: usize>(
    stream: &mut UnixStream,
) -> Result<Vec<u8>, IpcError> {
    let mut response = Vec::new();
    // default kernel size buffer for reads
    let buf = &mut [0; N];
    let mut counter = 0;
    loop {
        // handle read
        let bytes_read = stream.read(buf).map_err(|e| IpcError::IO(e))?;
        // println!("bytes read {}", bytes_read);
        if bytes_read == 0 {
            // if we read nothing then say eof
            if counter == 0 {
                // println!("eof?");
                return Err(IpcError::EOF);
            }
            // data might be valid, might be invalid, no way of knowing since
            // it is multiple of `N`
            // println!("eof msg end");
            return Ok(response);
        }
        counter += 1;
        response.extend(buf[0..bytes_read].iter());
        if bytes_read != N {
            // println!("msg end");
            return Ok(response);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{thread::sleep, time::Duration};

    fn run_test_chunk<const N: usize>() {
        let dummy_data: Vec<u8> = vec![1; 10];
        let (mut sock0, mut sock1) = UnixStream::pair().unwrap();

        let inner_data = dummy_data.clone();
        let handle1 = std::thread::spawn(move || {
            sleep(Duration::from_millis(100));
            sock1.write_all(&inner_data)
        });

        let handle0 = std::thread::spawn(move || get_ipc_response_with_read_size::<N>(&mut sock0));

        let res = handle0.join().unwrap().unwrap();
        println!("write res {:?}", handle1.join());
        println!("got res {:?}", res);
        assert_eq!(dummy_data, res);
    }

    #[test]
    fn test_ipc_chunked_data() {
        run_test_chunk::<2>();
        run_test_chunk::<3>();
    }
}
