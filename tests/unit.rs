use carla::async_io::executor::block_on;

#[test]
fn simple_test() -> carla::Result<()> {
    async fn test_x() -> usize {
        10
    }
    
    async fn test_y() -> usize {
        20
    }
    
    block_on(async { 
        let x = test_x().await;
        let y = test_y().await;
        assert_eq!(x, 10);
        assert_eq!(y, 20);
        Ok(())
    })
}