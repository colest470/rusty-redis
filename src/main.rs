
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod helper;
mod inbuilt;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .unwrap();

    let set_hash: Arc<Mutex<HashMap<String, String>>> =
        Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (mut stream, peer) = listener.accept().await.unwrap();

        let set_hash = Arc::clone(&set_hash);

        tokio::spawn(async move {
            let mut is_multi = false;
            let mut qued_commands: Vec<String> = Vec::new();

            let mut buf = [0u8; 1024];

            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => return,

                    Ok(n) => {
                        let input_arg = match helper::parse_command(&buf[..n]) {
                            Some(v) => v,
                            None => continue,
                        };

                        let command = input_arg
                            .first()
                            .map(String::as_str)
                            .unwrap_or("");

                        if command == "MULTI" {
                            if is_multi {
                                let _ = stream.write_all(b"-ERR MULTI calls can not be nested\r\n").await;
                            } else {
                                is_multi = true;
                                qued_commands.clear();

                                let _ = stream.write_all(b"+OK\r\n").await;
                            }

                            continue;
                        }

                        if command == "EXEC" {
                            if !is_multi {
                                let _ = stream.write_all(b"-ERR EXEC without MULTI\r\n").await;
                                continue;
                            }

                            is_multi = false;

                            let reply = inbuilt::exec(&mut qued_commands, &set_hash);

                            let bytes: Vec<u8> = reply
                                .iter()
                                .flat_map(|f| f.as_bytes().to_vec())
                                .collect();

                            let _ = stream.write_all(&bytes).await;
                            continue;
                        }

                        if command == "DISCARD" {
                            if !is_multi {
                                let _ = stream.write_all(
                                    b"-ERR DISCARD without MULTI\r\n"
                                ).await;
                            } else {
                                is_multi = false;
                                qued_commands.clear();

                                let _ = stream.write_all(b"+OK\r\n").await;
                            }

                            continue;
                        }

                        if is_multi {
                            let result = inbuilt::multi(
                                input_arg.clone(),
                                &mut qued_commands
                            );

                            let response = format!("+QUEUED\r\n");

                            println!("{result}");

                            if stream.write_all(response.as_bytes()).await.is_err() {
                                return;
                            }

                            continue;
                        }

                        match command {
                            "PING" => {
                                if stream.write_all(b"+PONG\r\n").await.is_err() {
                                    return;
                                }
                            }

                            "ECHO" if input_arg.len() >= 2 => {
                                let msg = &input_arg[1];

                                let response = format!("${}\r\n{}\r\n", msg.len(),msg);

                                if stream.write_all(response.as_bytes()).await.is_err() {
                                    return;
                                }
                            }

                            "SET" | "GET" => {
                                match inbuilt::set_get(
                                    input_arg.clone(),
                                    &set_hash
                                ) {
                                    Some(v) if v == "OK" => {
                                        let _ = stream.write_all(b"+OK\r\n").await;
                                    }

                                    Some(v) => {
                                        let response = format!(
                                            "${}\r\n{}\r\n",
                                            v.len(),
                                            v
                                        );

                                        let _ = stream.write_all(
                                            response.as_bytes()
                                        ).await;
                                    }

                                    None => {
                                        let _ = stream.write_all(b"$-1\r\n").await;
                                    }
                                }
                            }

                            "INCR" => {
                                match inbuilt::increment(
                                    input_arg.clone(),
                                    &set_hash
                                ) {
                                    Some(value) => {
                                        let response = format!(
                                            ":{}\r\n",
                                            value
                                        );

                                        let _ = stream.write_all(
                                            response.as_bytes()
                                        ).await;
                                    }

                                    None => {
                                        let _ = stream.write_all(
                                            b"-ERR value is not an integer or out of range\r\n"
                                        ).await;
                                    }
                                }
                            }

                            _ => {
                                let response = format!(
                                    "-ERR unknown command '{}'\r\n",
                                    command
                                );

                                if stream.write_all(response.as_bytes()).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }

                    Err(e) => {
                        println!("Read error from {peer}: {e}");
                        return;
                    }
                }
            }
        });
    }
}
