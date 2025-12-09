use super::*;

use std::sync::atomic::{AtomicPtr, Ordering};

pub struct Inner(tokio::net::UdpSocket);

impl std::ops::Deref for Inner {
    type Target = tokio::net::UdpSocket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct UdpSocket(AtomicPtr<Arc<Inner>>);

impl From<tokio::net::UdpSocket> for UdpSocket {
    fn from(value: tokio::net::UdpSocket) -> Self {
        let v = Box::new(Arc::new(Inner(value)));
        let ptr = Box::into_raw(v);
        Self(AtomicPtr::new(ptr))
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        self.take();
    }
}

fn conn_closed_err() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::ConnectionAborted,
        "udp socket close requested",
    )
}

impl UdpSocket {
    pub fn from_std(socket: std::net::UdpSocket) -> std::io::Result<Self> {
        tokio::net::UdpSocket::from_std(socket).map(Into::into)
    }

    pub async fn bind<A>(addr: A) -> std::io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        tokio::net::UdpSocket::bind(addr).await.map(Into::into)
    }

    pub async fn connect<A>(&self, addr: A) -> std::io::Result<()>
    where
        A: ToSocketAddrs,
    {
        Ok(tokio::net::UdpSocket::connect(&*self.get()?, addr).await?)
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.get()?.local_addr()
    }

    pub async fn send_to<A>(&self, buf: &[u8], target: A) -> std::io::Result<usize>
    where
        A: ToSocketAddrs,
    {
        self.get()?.send_to(buf, target).await
    }

    pub async fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.get()?.recv_from(buf).await
    }

    fn get(&self) -> std::io::Result<Arc<Inner>> {
        let ptr = self.0.load(Ordering::SeqCst);

        if ptr.is_null() {
            return Err(conn_closed_err());
        }

        Ok(unsafe { &*ptr }.clone())
    }

    fn take(&self) {
        let ptr = self.0.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !ptr.is_null() {
            drop(unsafe { Box::from_raw(ptr) });
        }
    }
    }

#[async_trait]
impl Conn for UdpSocket {
    async fn connect(&self, addr: SocketAddr) -> Result<()> {
        Ok(Self::connect(self, addr).await?)
    }

    async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        Ok(self.get()?.recv(buf).await?)
    }

    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        Ok(self.get()?.recv_from(buf).await?)
    }

    async fn send(&self, buf: &[u8]) -> Result<usize> {
        Ok(self.get()?.send(buf).await?)
    }

    async fn send_to(&self, buf: &[u8], target: SocketAddr) -> Result<usize> {
        Ok(self.get()?.send_to(buf, target).await?)
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.get()?.local_addr()?)
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }

    async fn close(&self) -> Result<()> {
        self.take();
        Ok(())
    }

    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}
