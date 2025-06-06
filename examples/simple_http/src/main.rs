use carla::net::TcpListener;
use carla::Result;

#[carla::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    while let Ok((mut client, addr)) = listener.accept().await {
        println!("{} connected", addr);
        let mut buf = vec![0; 1024];
        client.read(&mut buf).await?;
        println!("{} bytes read", buf.len());
        println!("{}", String::from_utf8_lossy(&buf));

        let content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Title</title>
</head>
<body>
    Hello, world!
</body>
</html>"#;
        let res = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",content.len(), content);
        println!("{}", res);
        if let Ok(bytes) = client.write(
            &res.as_bytes(),
        ).await {
            println!("{} bytes write", bytes);
        }
    }
    Ok(())
}