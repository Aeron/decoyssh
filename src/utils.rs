use std::collections::HashSet;
use std::net::SocketAddr;

use async_std::prelude::*;
use async_std::stream::repeat_with;
use async_std::sync::Mutex;
use fastrand::alphanumeric;
use once_cell::sync::Lazy;

const CRLF: &str = "\r\n";
static CONN_POOL: Lazy<Mutex<HashSet<SocketAddr>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub async fn add_connection(addr: SocketAddr, cap: usize) -> bool {
    let mut pool = CONN_POOL.lock().await;

    if pool.len() >= cap {
        return false;
    }

    pool.insert(addr)
}

pub async fn remove_connection(addr: SocketAddr) {
    let mut pool = CONN_POOL.lock().await;

    pool.remove(&addr);
}

pub async fn generate_random_alphanumeric(length: usize) -> String {
    let mut result: String = String::with_capacity(length + CRLF.len());
    let mut stream = repeat_with(alphanumeric).take(length);

    while let Some(ch) = stream.next().await {
        result.push(ch);
    }

    // NOTE: newline is enough, but the protocol expects CRLF
    result.push_str(CRLF);

    result
}
