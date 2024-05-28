use std::fs;
use std::os::raw::{c_int, c_void};
use std::time::Instant;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt};

// extern {
//     fn byteToInt (ptr_in: *mut c_void, len: c_int) -> *mut c_int;
// }

#[tokio::main]
async fn main() {
    const SOCKET_PATH: &str = "/tmp/socket1.sock";
    const BUFFER_SIZE: usize = 1_000_000;
    const BUFFER_THRESHOLD: usize = BUFFER_SIZE - 200_000;
    // Remove socket file
    if fs::metadata(SOCKET_PATH).is_ok() {
        if let Err(e) = fs::remove_file(SOCKET_PATH) {
            eprintln!("Error removing socket file: {}", e);
            return;
        }
    }
    // Create UnixSocket
    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error binding to UNIX socket: {}", e);
            return;
        }
    };
    let mut cnt_recv = 0;
    let mut whole_bytes = 0;

    match listener.accept().await {
        Ok((mut socket, addr)) => {
            println!("Got a client: {addr:?}");
            let mut now = Instant::now();
            let time = Instant::now();

            let mut buffer = vec![0; BUFFER_SIZE];
            let mut buffer_offset: usize = 0;

            let lib_ptr = buffer.as_mut_ptr() as *mut c_void;
            let lib_len_max = buffer.len() as c_int;

            loop {
                let body_slice: &mut [u8] = &mut buffer[buffer_offset..];
                match socket.read(body_slice).await {
                    Ok(len_recv) => {
                        if len_recv > body_slice.len() {
                            println!("Error receiving data: data is to long");
                            return;
                        };
                        buffer_offset += len_recv;
                        cnt_recv += len_recv;
                    },
                    Err(e) => {
                        eprintln!("Error receiving data: {:?}", e);
                        return;
                    }
                };
                // Using of C-library
                unsafe {
                    //let _result = byteToInt(lib_ptr, lib_len_max);
                    // for i in 0..MAX_NUMBERS {
                    //     println!("result: {}", *result.offset(i.try_into().unwrap()));
                    // }
                }

                if now.elapsed().as_secs() >= 1 {
                    server_bandwidth(cnt_recv, &mut whole_bytes, time);
                    cnt_recv = 0;
                    now = Instant::now();
                }

                if buffer_offset >= BUFFER_THRESHOLD {
                    buffer.iter_mut().for_each(|x| *x = 0);
                    buffer_offset = 0;
                }
            }
        },
        Err(e) => {
            println!("accept function failed: {e:?}");
        }
    };
}

fn server_bandwidth(cnt_bytes: usize, whole_bytes: &mut usize, time: Instant) {
    *whole_bytes += cnt_bytes;
    println!("{} MB/sec\n{} MB total\n{} secs total work time\
    \n________________", cnt_bytes / 1_000, *whole_bytes / 1_000, time.elapsed().as_secs());
}