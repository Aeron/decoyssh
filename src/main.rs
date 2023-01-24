mod args;
mod pool;
mod tasks;

use std::time::Duration;

use async_signals::Signals;
use async_std::prelude::*;
use async_std::task;
use once_cell::sync::Lazy;

use crate::args::Args;
use crate::pool::ConnectionPool;
use crate::tasks::listen;

#[cfg(all(target_env = "musl"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

static ARGS: Lazy<Args> = Lazy::new(|| Args::parse());
static POOL: Lazy<ConnectionPool> = Lazy::new(|| ConnectionPool::with_capacity(ARGS.cap.into()));

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

    for addr in ARGS.addrs() {
        task::spawn(listen(addr));
    }

    loop {
        // HACK: drastically lowers the CPU usage of an infinite loop
        task::sleep(Duration::from_secs(60)).await;
    }
}
