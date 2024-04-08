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
            capacity, // NOTE: HashSet::with_capacity only garantees the least capacity
            inner: HashSet::with_capacity(capacity),
            sender,
            receiver,
        }
    }

    /// Starts a message channel listener.
    pub async fn listen(&mut self) {
        while let Some(msg) = self.receiver.next().await {
            match msg {
                ConnectionPoolMessage::Insert(addr, resp) => {
                    let msg = self.inner.len() < self.capacity && self.inner.insert(addr);
                    let _ = resp.send(msg).await;
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
    pub async fn insert(&self, addr: SocketAddr) -> Option<bool> {
        let (sender, receiver) = channel::bounded(1);
        let msg = ConnectionPoolMessage::Insert(addr, sender);

        match self.inner.send(msg).await {
            Ok(_) => receiver.recv().await.ok(),
            Err(_) => None,
        }
    }

    pub async fn remove(&self, addr: SocketAddr) {
        let _ = self.inner.send(ConnectionPoolMessage::Remove(addr)).await;
    }
}
