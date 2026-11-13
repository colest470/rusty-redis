#![allow(unused_imports)]
use std::net::TcpListener;
use std::io::{Write, Read};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment the code below to pass the first stage
    
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                //let _ = stream.write_all(b"+PONG\r\n").unwrap(); //.unwrap() changes the Result<_, _> to () which panics if there is an error

                // println!("{:?}", stream); // TcpStream { addr: 127.0.0.1:6379, peer: 127.0.0.1:48790, fd: 4 }

                let mut buf = [0u8; 1024]; // 1mb of 0 values
                
                match stream.read(&mut buf) {
                    Ok(n) => {
                        // println!("{:?}", String::from_utf8_lossy(&mut buf[..n]));
                        if String::from_utf8_lossy(&mut buf[..n]) == "*1\r\n$4\r\nPING\r\n" {
                            let _ = stream.write_all(b"+PONG\r\n");
                            let _ = stream.write_all(b"+PONG\r\n");
                        }
                    }
                    Err(e) => {
                        //println!("error: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                //println!("error: {}", e);
            }
        }
    }
    
}
