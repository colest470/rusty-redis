use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

mod helper;
mod inbuilt;

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

                        // let input = String::from_utf8_lossy(&buf[..n]);

                        let input_arg = helper::parse_command(&buf).unwrap();

                        match input_arg.first().map(String::as_str) {
                            Some("PING") => {
                                match stream.write_all(b"+PONG\r\n").await {
                                    Ok(_) => println!("Pong success!"),
                                    Err(e) => { println!("write error: {e}"); return; }
                                }
                            }
                            Some("ECHO") if input_arg.len() >= 2 => {
                                let msg = &input_arg[1];
                                let msg_format = format!("${}\r\n{}\r\n", msg.len(), msg);
                                let _ = stream.write_all(msg_format.as_bytes()).await.unwrap();
                            },
                            Some("SET") | Some("GET") if input_arg.len() >= 3 => {
                                let res = inbuilt::set_get(input_arg.iter().map(|s| s.clone()).collect()).unwrap();

                                if res.as_str() == "OK" {
                                    let _ = stream.write_all(b"+OK\r\n").await.unwrap();
                                }

                                let formated = format!("${}\r\n{}\r\n", res.len(), res);
                                let _ = stream.write_all(formated.as_bytes()).await.unwrap();
                            }
                            _ => {
                                println!("Unrecognized command");
                            }
                        }

                        println!("{:?}", input_arg);
                    }
                    Err(e) => { println!("read error: {e}"); return; }
                }
            }
        });
    }
}
