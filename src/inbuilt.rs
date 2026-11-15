use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn set_get(
    set_args: Vec<String>,
    set_hash: &Arc<Mutex<HashMap<String, String>>>,
) -> Option<String> {
    match set_args[0].as_str() {
        "SET" => {
            if set_args.get(3).map(String::as_str) == Some("EX") {
                set_timeout(
                    {
                        let map = Arc::clone(set_hash);
                        let key = set_args[1].clone();
                        let val = set_args[2].clone();
                        move || {
                            map.lock().unwrap().insert(key, val);
                        }
                    },
                    set_args[4].parse::<u64>().unwrap() * 1000, // seconds -> ms
                );
            } else if set_args.get(3).map(String::as_str) == Some("PX") {
                set_timeout(
                    {
                        let map = Arc::clone(set_hash);
                        let key = set_args[1].clone();
                        let val = set_args[2].clone();
                        move || {
                            map.lock().unwrap().insert(key, val);
                        }
                    },
                    set_args[4].parse::<u64>().unwrap(), // already ms
                );
            } else {
                set_hash
                    .lock()
                    .unwrap()
                    .insert(set_args[1].clone(), set_args[2].clone());
            }
            Some(String::from("OK"))
        }

        "GET" => {
            let guard = set_hash.lock().unwrap();
            match guard.get(&set_args[1]) {
                Some(n) => Some(n.clone()),
                _ => {
                    println!("Get key invalid!");
                    None
                }
            }
        }

        _ => {
            println!("Command unrecognized");
            None
        }
    }
}

fn set_timeout<F>(callback: F, delay_ms: u64)
where
    F: FnOnce() + Send + 'static,
{
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay_ms));
        callback();
    });
}