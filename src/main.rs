use anyhow::Result;
use crate::async_io::executor::Executor;
use crate::net::TcpListener;

mod async_io;
mod net;

fn main() -> Result<()> {
    let mut executor = Executor::new();
    executor.spawn(async {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        while let Ok((client, addr)) = listener.accept().await {
            println!("{} connected", addr);
        }
    });
    
    executor.run()?;
    Ok(())
}

async fn test() -> usize {
    10
}
