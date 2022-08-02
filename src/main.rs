mod args;
mod utils;

use std::time::{Duration, Instant};

use async_signals::Signals;
use async_std::io::{ErrorKind, Result};
use async_std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;

use crate::args::*;
use crate::utils::*;

#[cfg(all(target_env = "musl"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn process(mut stream: TcpStream, delay: Duration, length: usize, cap: usize) -> Result<()> {
    let addr = stream.peer_addr()?;

    match add_connection(addr, cap).await {
        true => {
            println!("Got connection from {}", addr);

            let now = Instant::now();

            loop {
                // TODO: stream.interval is unstable yet, but it will be one day
                let msg = generate_random_alphanumeric(length).await;

                if let Some(e) = stream.write_all(msg.as_bytes()).await.err() {
                    if e.kind() != ErrorKind::WouldBlock {
                        break;
                    }
                }

                task::sleep(delay).await;
            }

            let elapsed = now.elapsed() - delay;

            println!("{} has gone yet wasted ~{:.2?}", addr, elapsed);
        }
        false => {
            stream.shutdown(Shutdown::Both)?;
        }
    }

    remove_connection(addr).await;

    Ok(())
}

async fn listen(addr: SocketAddr, delay: Duration, length: usize, cap: usize) {
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("Listening on {}", addr);
            listener
        }
        Err(ref e) => {
            println!("Cannot listen on {}: {}", addr, e);
            return;
        }
    };

    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = match stream {
            Ok(stream) => stream,
            Err(ref e) => {
                println!("Cannot obtain a TCP stream: {}", e);
                continue;
            }
        };
        stream.set_nodelay(true).ok(); // we do not really care if it clicks or not

        task::spawn(async move {
            process(stream, delay, length, cap).await.ok();
        });
    }
}

#[async_std::main]
async fn main() {
    task::spawn(async {
        // NOTE: SIGHUP = 1, SIGINT = 2, SIGTERM = 15
        let mut signals = Signals::new([1, 2, 15]).unwrap();

        while let Some(_) = signals.next().await {
            println!("Quitting");
            std::process::exit(0);
        }
    });

    let args = Args::parse();

    let cap = usize::from(args.cap);
    let delay = Duration::from_millis(args.delay);
    let length = usize::from(args.length);

    // NOTE: I do not know how to make it more efficient.
    // Chaining/merging circus with optional listeners and incomings, dressed with
    // boxed dynamic Stream trait objects, does not look like a better solution
    // than having two separate tasks handling two streams at most.
    // Maybe there is a better way.
    for addr in args.addrs() {
        task::spawn(listen(addr, delay, length, cap));
    }

    loop {}
}
