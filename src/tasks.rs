use std::time::{Duration, Instant};

use async_std::io::ErrorKind;
use async_std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::stream::repeat_with;
use async_std::task;
use fastrand::alphanumeric;

use crate::{ARGS, POOL};

const CRLF: &str = "\r\n";

pub async fn listen(addr: SocketAddr) {
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("Listening on {addr}");
            listener
        }
        Err(ref e) => {
            eprintln!("Cannot listen on {addr}: {e}");
            return;
        }
    };

    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = match stream {
            Ok(stream) => stream,
            Err(ref e) => {
                eprintln!("Cannot obtain a TCP stream: {e}");
                continue;
            }
        };
        stream.set_nodelay(true).ok(); // we do not really care if it clicks or not

        task::spawn(process(stream));
    }
}

async fn process(mut stream: TcpStream) {
    let addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => return,
    };

    let length = usize::from(ARGS.length);
    let cap = length + CRLF.len();

    match POOL.insert(addr).await {
        true => {
            println!("Got connection from {addr}");

            let mut buf: String = String::with_capacity(cap);
            let now = Instant::now();

            loop {
                repeat_with(alphanumeric)
                    .take(length)
                    .for_each(|ch| buf.push(ch))
                    .await;

                buf.push_str(CRLF);

                if let Some(ref e) = stream.write_all(buf.as_bytes()).await.err() {
                    if e.kind() != ErrorKind::WouldBlock {
                        println!("{addr} has gone yet wasted ~{:.2?}", now.elapsed());
                        break;
                    }
                }

                buf.clear();

                task::sleep(Duration::from_millis(ARGS.delay)).await;
            }
        }
        false => {
            stream.shutdown(Shutdown::Both).ok();
        }
    }

    POOL.remove(&addr).await;
}
