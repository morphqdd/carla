use std::io::ErrorKind;
use std::net;
use std::pin::Pin;
use std::task::{Context, Poll};
use anyhow::{anyhow, Result};
use crate::async_io::reactor::REACTOR;

pub struct TcpListener {
    listener: net::TcpListener,
}

impl TcpListener {
    pub fn bind(addr: &str) -> Result<Self> {
        let listener = net::TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener })
    }
    
    pub fn accept(&self) -> AcceptStream {
        let reactor = REACTOR.read()
            .map_err(|e| anyhow!("{e}")).unwrap();
        reactor.add(&self.listener);
        AcceptStream {
            listener: &self.listener
        }
    }
}

pub struct AcceptStream<'listener> {
    listener: &'listener net::TcpListener,
}

impl Future for AcceptStream<'_> {
    type Output = Result<(net::TcpStream, net::SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.listener.accept() {
            Ok(val) => Poll::Ready(Ok(val)),
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                let mut reactor = REACTOR.write()
                    .map_err(|e| anyhow!("{e}"))?;
                reactor.wake_on_readable(self.listener, cx);
                
                Poll::Pending
            }
            Err(err) => Poll::Ready(Err(anyhow!("{err}")))
        }
    }
}

impl Drop for AcceptStream<'_> {
    fn drop(&mut self) {
        REACTOR.write().unwrap().remove(&self.listener);
    }
}