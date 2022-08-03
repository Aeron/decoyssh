use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

use async_std::prelude::*;
use async_std::stream::repeat_with;
use async_std::sync::Mutex;
use fastrand::alphanumeric;
use once_cell::sync::Lazy;

const CRLF: &str = "\r\n";
const SEP: &str = ", ";

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

// NOTE: Since Vec<...> has no Display trait and such trait cannot be implemented
// directly, the new type idiom is the way to go.
pub struct DisplayableVec<T: Display>(pub Vec<T>);

impl<T: Display> Display for DisplayableVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut s = String::new();

        // NOTE: It can be done thru .enumerate() and index-length comparison or thru
        // slices, but I like this a bit better.
        for a in self.0.iter() {
            s.push_str(&a.to_string());
            s.push_str(SEP);
        }

        write!(f, "{}", s.trim_end_matches(SEP))
    }
}
