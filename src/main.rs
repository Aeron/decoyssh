mod args;
mod pool;

use std::time::{Duration, Instant};

use async_signals::Signals;
use async_std::io::ErrorKind;
use async_std::net::TcpListener;
use async_std::net::{Shutdown, TcpStream};
use async_std::prelude::*;
use async_std::stream::repeat_with;
use async_std::task;
use fastrand::alphanumeric;
use futures::stream::select_all;
use once_cell::sync::Lazy;

use crate::args::Args;
use crate::pool::ConnectionPool;

#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

static ARGS: Lazy<Args> = Lazy::new(Args::parse);
static POOL: Lazy<ConnectionPool> = Lazy::new(|| ConnectionPool::with_capacity(ARGS.cap.into()));

const CRLF: &str = "\r\n";

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

#[async_std::main]
async fn main() {
    task::spawn(async {
        // NOTE: SIGHUP = 1, SIGINT = 2, SIGTERM = 15
        let mut signals = Signals::new([1, 2, 15]).unwrap();

        while (signals.next().await).is_some() {
            println!("Quitting");
            std::process::exit(0);
        }
    });

    let addrs = ARGS.addrs();
    let mut listeners = Vec::with_capacity(addrs.capacity());

    for addr in addrs {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("Listening on {addr}");
                listeners.push(listener);
            }
            Err(ref e) => {
                eprintln!("Cannot listen on {addr}: {e}");
                std::process::exit(1);
            }
        };
    }

    let mut incoming = select_all(listeners.iter().map(|l| l.incoming()));

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
