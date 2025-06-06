use std::io::{Read, Write};
use std::net::TcpStream;
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::async_io::reactor::REACTOR;
use anyhow::{anyhow, Result};

pub struct TcpClient {
    stream: TcpStream,
}

impl TcpClient {
    pub fn new(stream: TcpStream) -> Self {
        
        let reactor = REACTOR.read()
            .map_err(|e| anyhow!("{e}")).unwrap();
        reactor.add(&stream);
        
        Self { stream }
    }
    
    pub fn read<'s, T: AsMut<[u8]>>(&'s mut self, buf: &'s mut T) -> ReadFuture<'s, T> {
        ReadFuture {
            stream: &mut self.stream,
            buf
        }
    }

    pub fn write<'s, T: AsRef<[u8]>>(&'s mut self, buf: &'s T) -> WriteFuture<'s, T> {
        WriteFuture {
            stream: &mut self.stream,
            buf
        }
    }
    
    pub fn flush(&mut self) -> Result<()> {
        Ok(self.stream.flush()?)
    }
}

pub struct ReadFuture<'client, T: AsMut<[u8]>> {
    stream: &'client mut TcpStream,
    buf: &'client mut T
}

impl<T: AsMut<[u8]>> Future for ReadFuture<'_, T> {
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();
        match state.stream.read(state.buf.as_mut()) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                {
                    REACTOR.write().unwrap()
                        .wake_on_readable(&*state.stream, cx);
                }
                Poll::Pending
            }
            Err(err) => Poll::Ready(Err(anyhow!("{}", err))),
        }
    }
}

pub struct WriteFuture<'client, T: AsRef<[u8]>> {
    stream: &'client mut TcpStream,
    buf: &'client T
}

impl<T: AsRef<[u8]>> Future for WriteFuture<'_, T> {
    type Output = Result<usize>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();
        match state.stream.write(state.buf.as_ref()) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                {
                    REACTOR.write().unwrap()
                        .wake_on_writable(&*state.stream, cx);
                }
                Poll::Pending
            }
            Err(err) => Poll::Ready(Err(anyhow!("{}", err))),
        }
    }
}

impl Drop for TcpClient {
    fn drop(&mut self) {
        REACTOR.write().unwrap().remove(&self.stream);
    }
}