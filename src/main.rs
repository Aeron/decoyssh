mod args;
mod pool;

use std::time::{Duration, Instant};

use async_signals::Signals;
use async_std::io::ErrorKind;
use async_std::net::TcpListener;
use async_std::net::{Shutdown, SocketAddr, TcpStream};
use async_std::prelude::*;
use async_std::stream::repeat_with;
use async_std::task;
use fastrand::alphanumeric;
use futures::stream::select_all;
use once_cell::sync::Lazy;

use crate::args::Args;
use crate::pool::{ConnectionPool, ConnectionPoolProxy};

const CRLF: &str = "\r\n";

#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

static ARGS: Lazy<Args> = Lazy::new(Args::parse);

async fn process(
    mut stream: TcpStream,
    pool: ConnectionPoolProxy,
) -> Result<(SocketAddr, Duration), std::io::Error> {
    let addr = stream.peer_addr()?;
    let length = ARGS.length as usize;
    let cap = length + CRLF.len();
    let now = Instant::now();

    if pool.insert(addr).await.unwrap_or_default() {
        let mut buf: String = String::with_capacity(cap);
        let mut rep = repeat_with(alphanumeric);

        while let Some(ch) = rep.next().await {
            if buf.len() < length {
                buf.push(ch);
                continue;
            }

            buf.push_str(CRLF);

            if let Some(err) = stream.write_all(buf.as_bytes()).await.err() {
                if err.kind() != ErrorKind::WouldBlock {
                    break;
                }
            };

            buf.clear();

            task::sleep(Duration::from_millis(ARGS.delay)).await;
        }
    } else {
        stream.shutdown(Shutdown::Both)?;
    }

    pool.remove(addr).await;

    Ok((addr, now.elapsed()))
}

#[async_std::main]
async fn main() {
    task::spawn(async {
        // NOTE: SIGHUP = 1, SIGINT = 2, SIGTERM = 15
        let mut signals = Signals::new([1, 2, 15]).unwrap();

        if signals.next().await.is_some() {
            println!("Quitting");
            std::process::exit(0);
        }
    });

    let mut pool = ConnectionPool::with_capacity(ARGS.cap as usize);
    let proxy = pool.proxy();

    task::spawn(async move {
        pool.manager().await;
    });

    let addrs = ARGS.addrs();
    let mut listeners = Vec::with_capacity(addrs.capacity());

    for addr in addrs {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("Listening on {addr}");
                listeners.push(listener);
            }
            Err(ref err) => {
                eprintln!("Cannot listen on {addr}: {err}");
                std::process::exit(1);
            }
        };
    }

    let mut incoming = select_all(listeners.iter().map(|l| l.incoming()));

    while let Some(stream) = incoming.next().await {
        let stream = match stream {
            Ok(stream) => stream,
            Err(ref err) => {
                eprintln!("Cannot obtain a TCP stream: {err}");
                continue;
            }
        };
        stream.set_nodelay(true).ok(); // we do not really care if it clicks or not

        let pool = proxy.clone();

        task::spawn(async {
            // NOTE: all errors are meaningless for us here
            if let Ok((addr, dur)) = process(stream, pool).await {
                println!("{addr} has gone yet wasted ~{dur:.2?}")
            };
        });
    }
}
