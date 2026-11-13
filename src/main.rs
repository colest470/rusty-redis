use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (mut stream, _peer) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        println!("{:?}", String::from_utf8_lossy(&buf[..n]));
                        match stream.write_all(b"+PONG\r\n").await {
                            Ok(_) => println!("Pong success!"),
                            Err(e) => { println!("write error: {e}"); return; }
                        }
                    }
                    Err(e) => { println!("read error: {e}"); return; }
                }
            }
        });
    }
}