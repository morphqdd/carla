use anyhow::Result;
use crate::async_io::executor::Executor;

mod async_io;

fn main() -> Result<()> {
    let mut executor = Executor::new();
    executor.spawn(async {
        let x = test().await;
        println!("{x}");
    });
    
    executor.run()?;
    Ok(())
}

async fn test() -> usize {
    10
}
