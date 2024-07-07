use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};

use scc::HashSet;

/// Represents a pool of unique socket addresses.
pub struct ConnectionPool {
    inner: HashSet<SocketAddr>,
    capacity: u16,
    length: AtomicU16,
}

impl ConnectionPool {
    /// Creates a new pool with a given capacity.
    pub fn with_capacity(capacity: u16) -> ConnectionPool {
        ConnectionPool {
            inner: HashSet::with_capacity(capacity as usize),
            // NOTE: HashSet::with_capacity only garantees the least capacity
            // but we want a precise value
            capacity,
            length: AtomicU16::new(0),
        }
    }

    /// Inserts an address into the pool.
    pub async fn insert(&self, addr: SocketAddr) -> bool {
        if self.length.load(Ordering::Acquire) < self.capacity
            && self.inner.insert_async(addr).await.is_ok()
        {
            self.length.fetch_add(1, Ordering::Release);
            return true;
        }
        false
    }

    /// Removes an address from the pool.
    pub async fn remove(&self, addr: &SocketAddr) -> bool {
        if self.inner.remove_async(addr).await.is_some() {
            self.length.fetch_sub(1, Ordering::Release);
            return true;
        }
        false
    }
}
