use std::collections::HashSet;
use std::net::SocketAddr;

use async_std::channel::{self, Receiver, Sender};
use async_std::stream::StreamExt;

/// Represents a pool of unique socket addresses.
pub struct ConnectionPool {
    inner: HashSet<SocketAddr>,
    capacity: usize,
    sender: Sender<ConnectionPoolMessage>,
    receiver: Receiver<ConnectionPoolMessage>,
}

impl ConnectionPool {
    /// Creates a new ConnectioPool with a given capacity.
    pub fn with_capacity(capacity: usize) -> ConnectionPool {
        let (sender, receiver) = channel::bounded(capacity);

        ConnectionPool {
            inner: HashSet::with_capacity(capacity),
            // NOTE: HashSet::with_capacity only garantees the least capacity
            // but we want a precise value
            capacity,
            sender,
            receiver,
        }
    }

    /// Starts a message channel listener.
    pub async fn listen(&mut self) {
        while let Some(msg) = self.receiver.next().await {
            match msg {
                ConnectionPoolMessage::Insert(addr, resp) => {
                    let inserted = self.inner.len() < self.capacity && self.inner.insert(addr);

                    resp.send(inserted).await.ok();
                    resp.close();
                }
                ConnectionPoolMessage::Remove(addr) => {
                    self.inner.remove(&addr);
                }
            }
        }
    }

    /// Returns a new ConnectionPoolProxy.
    pub fn proxy(&self) -> ConnectionPoolProxy {
        ConnectionPoolProxy {
            inner: self.sender.clone(),
        }
    }
}

/// Represents a connection pool channel message.
enum ConnectionPoolMessage {
    Insert(SocketAddr, Sender<bool>),
    Remove(SocketAddr),
}

/// Represents a connection pool (sender) proxy.
#[derive(Clone)]
pub struct ConnectionPoolProxy {
    inner: Sender<ConnectionPoolMessage>,
}

impl ConnectionPoolProxy {
    pub async fn insert(&self, addr: SocketAddr) -> bool {
        let (sender, receiver) = channel::bounded(1);
        let msg = ConnectionPoolMessage::Insert(addr, sender);

        if self.inner.send(msg).await.is_err() {
            return false;
        }

        receiver.recv().await.unwrap_or_default()
    }

    pub async fn remove(&self, addr: SocketAddr) -> bool {
        self.inner
            .send(ConnectionPoolMessage::Remove(addr))
            .await
            .is_ok()
    }
}
