mod args;
mod utils;

use std::time::{Duration, Instant};

use async_signals::Signals;
use async_std::io::{ErrorKind, Result};
use async_std::net::{Shutdown, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;

use crate::args::*;
use crate::utils::*;

async fn process(mut stream: TcpStream, delay: Duration, length: usize, cap: usize) -> Result<()> {
    let addr = stream.peer_addr()?;

    match add_connection(addr, cap).await {
        true => {
            println!("Got connection from {}", addr);

            let now = Instant::now();

            loop {
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

fn main() -> Result<()> {
    task::block_on(async {
        task::spawn(async {
            let supported = vec![1, 2, 15]; // NOTE: SIGHUP, SIGINT, SIGTERM
            let mut signals = Signals::new(supported).unwrap();

            while let Some(_) = signals.next().await {
                println!("Quitting");
                std::process::exit(0);
            }
        });

        let args = parse_app_args();

        let cap = usize::from(args.cap);
        let delay = Duration::from_millis(args.delay);
        let length = usize::from(args.length);

        let ipv4_listener = TcpListener::bind(args.ipv4_addr()).await?;
        let ipv6_listener = TcpListener::bind(args.ipv6_addr()).await?;

        println!("Listening on {}", ipv4_listener.local_addr()?);
        println!("Listening on {}", ipv6_listener.local_addr()?);

        let ipv4_incoming = ipv4_listener.incoming();
        let ipv6_incoming = ipv6_listener.incoming();

        // TODO: .merge is unstable yet, but it will be one day
        let mut incoming = ipv4_incoming.chain(ipv6_incoming);

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            stream.set_nodelay(true).ok();

            task::spawn(async move {
                process(stream, delay, length, cap).await.ok();
            });
        }

        Ok(())
    })
}
