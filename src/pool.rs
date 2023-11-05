use std::collections::HashSet;
use std::net::SocketAddr;

use async_std::sync::Mutex;

pub struct ConnectionPool {
    capacity: usize,
    inner: Mutex<HashSet<SocketAddr>>,
}

impl ConnectionPool {
    pub fn with_capacity(capacity: usize) -> ConnectionPool {
        ConnectionPool {
            capacity, // NOTE: HashSet::with_capacity only garantees the least capacity
            inner: Mutex::new(HashSet::with_capacity(capacity)),
        }
    }

    pub async fn insert(&self, addr: SocketAddr) -> bool {
        let mut guard = self.inner.lock().await;
        guard.len() < self.capacity && guard.insert(addr)
    }

    pub async fn remove(&self, addr: &SocketAddr) -> bool {
        self.inner.lock().await.remove(addr)
    }
}
