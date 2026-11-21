use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod helper;
mod inbuilt;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let set_hash: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let is_multi: Arc<Mutex<HashMap<u16, bool>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (mut stream, _peer) = listener.accept().await.unwrap();
        let set_hash = Arc::clone(&set_hash);
        let is_multi = Arc::clone(&is_multi);

        is_multi.lock().unwrap().insert(_peer.port(), false);

        tokio::spawn(async move {
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

                        // is_multi.lock().unwrap().get(&_peer.port()).copied().unwrap();

                        if is_multi.lock().unwrap().get(&_peer.port()).copied().unwrap() {
                            let result_string = inbuilt::multi(input_arg.clone(), &mut qued_commands);

                            println!("{result_string}");
                        }

                        match input_arg.first().map(String::as_str) {
                            Some("PING") => {
                                if let Err(e) = stream.write_all(b"+PONG\r\n").await {
                                    println!("write error: {e}");
                                    return;
                                }
                            }
                            Some("ECHO") if input_arg.len() >= 2 => {
                                let msg = &input_arg[1];
                                let msg_format = format!("${}\r\n{}\r\n", msg.len(), msg);
                                if stream.write_all(msg_format.as_bytes()).await.is_err() {
                                    return;
                                }
                            }
                            Some("SET") | Some("GET") => {
                                let res = inbuilt::set_get(input_arg.clone(), &set_hash);

                                match res {
                                    Some(v) if v == "OK" => {
                                        let _ = stream.write_all(b"+OK\r\n").await;
                                    }
                                    Some(v) => {
                                        let formatted = format!("${}\r\n{}\r\n", v.len(), v);
                                        let _ = stream.write_all(formatted.as_bytes()).await;
                                    }
                                    None => {
                                        let _ = stream.write_all(b"$-1\r\n").await;
                                    }
                                }
                            },
                            Some("INCR") => {
                                match inbuilt::increment(input_arg.clone(), &set_hash) {
                                    Some(s) => {
                                        let formated_string = format!(":{}\r\n", s);
                                        let _ = stream.write_all(formated_string.as_bytes()).await;
                                    },
                                    None => {
                                        let err = b"-ERR value is not an integer or out of range\r\n";
                                        let _ = stream.write_all(err).await;
                                    }
                                }
                            },
                            Some("MULTI") if is_multi.lock().unwrap().get(&_peer.port()).copied().unwrap_or(false) => {
                                let _ = stream.write_all(b"-ERR MULTI calls can not be nested\r\n").await;
                            },
                            Some("MULTI") => {
                                // match inbuilt::multi(input_arg.clone(), &mut qued_commands) {
                                //     Some(_) => {
                                //         is_multi = true;
                                //         let _ = stream.write_all(b"+OK\r\n").await;
                                //     },
                                //     None => {
                                //         let _ = stream.write_all(b"$-1\r\n").await;
                                //     }
                                // }

                                is_multi.lock().unwrap().insert(_peer.port(), true);

                                println!("{_peer} is set to true");

                                let ret = inbuilt::multi(input_arg.clone(), &mut qued_commands);

                                let _ = stream.write_all(format!("+{}\r\n", ret).as_bytes()).await;
                            },
                            Some("EXEC") => {
                                is_multi.lock().unwrap().insert(_peer.port(), false);
                                let reply = inbuilt::exec(&mut qued_commands, &set_hash);

                                println!("{reply}");

                                let _ = stream.write_all(reply.as_bytes()).await;
                            },
                            _ => {
                                let msg = format!(
                                    "-ERR unknown command '{}'\r\n",
                                    input_arg.first().map(String::as_str).unwrap_or("")
                                );
                                if stream.write_all(msg.as_bytes()).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("read error: {e}");
                        return;
                    }
                }
            }
        });
    }
}