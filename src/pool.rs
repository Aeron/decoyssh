use std::collections::HashSet;
use std::net::SocketAddr;

use async_std::channel::{self, Receiver, Sender};
use async_std::stream::StreamExt;

/// Represents a pool of unique socket addresses.
pub struct ConnectionPool {
    capacity: usize,
    inner: HashSet<SocketAddr>,
    channel: (
        Sender<ConnectionPoolMessage>,
        Receiver<ConnectionPoolMessage>,
    ),
}

impl ConnectionPool {
    /// Creates a new ConnectioPool with a given capacity.
    pub fn with_capacity(capacity: usize) -> ConnectionPool {
        ConnectionPool {
            capacity, // NOTE: HashSet::with_capacity only garantees the least capacity
            inner: HashSet::with_capacity(capacity),
            channel: channel::bounded(capacity),
        }
    }

    /// Starts a channel message manager.
    pub async fn manager(&mut self) {
        while let Some(msg) = self.channel.1.next().await {
            match msg {
                ConnectionPoolMessage::Insert { addr, resp } => {
                    let msg = self.inner.len() < self.capacity && self.inner.insert(addr);
                    let _ = resp.send(msg).await;
                    resp.close();
                }
                ConnectionPoolMessage::Remove { addr } => {
                    self.inner.remove(&addr);
                }
            }
        }
    }

    /// Returns a new ConnectionPoolProxy.
    pub fn proxy(&self) -> ConnectionPoolProxy {
        ConnectionPoolProxy {
            inner: self.channel.0.clone(),
        }
    }
}

/// Represents a connection pool channel message.
enum ConnectionPoolMessage {
    Insert {
        addr: SocketAddr,
        resp: Sender<bool>,
    },
    Remove {
        addr: SocketAddr,
    },
}

/// Represents a connection pool (sender) proxy.
#[derive(Clone)]
pub struct ConnectionPoolProxy {
    inner: Sender<ConnectionPoolMessage>,
}

impl ConnectionPoolProxy {
    pub async fn insert(&self, addr: SocketAddr) -> Option<bool> {
        let (sender, receiver) = channel::bounded(1);

        match self
            .inner
            .send(ConnectionPoolMessage::Insert { addr, resp: sender })
            .await
        {
            Ok(_) => receiver.recv().await.ok(),
            Err(_) => None,
        }
    }

    pub async fn remove(&self, addr: SocketAddr) {
        let _ = self
            .inner
            .send(ConnectionPoolMessage::Remove { addr })
            .await;
    }
}
