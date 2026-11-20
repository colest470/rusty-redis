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
            let key = set_args[1].clone();
            let val = set_args[2].clone();

            let expiry_ms = match (set_args.get(3).map(String::as_str), set_args.get(4)) {
                (Some("EX"), Some(n)) => Some(n.parse::<u64>().ok()? * 1000),
                (Some("PX"), Some(n)) => Some(n.parse::<u64>().ok()?),
                _ => None,
            };

            set_hash.lock().unwrap().insert(key.clone(), val.clone());

            if let Some(ms) = expiry_ms {
                let map = Arc::clone(set_hash);
                set_timeout(
                    move || {
                        let mut guard = map.lock().unwrap();
                        if guard.get(&key).map(|v| v.as_str()) == Some(val.as_str()) {
                            guard.remove(&key);
                        }
                    },
                    ms,
                );
            }

            Some(String::from("OK"))
        }

        "GET" => {
            let guard = set_hash.lock().unwrap();
            match guard.get(&set_args[1]) {
                Some(n) => Some(n.clone()),
                None => {
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

pub fn increment(input_args: Vec<String>, set_hash: &Arc<Mutex<HashMap<String, String>>>) -> Option<i32> {
    let key = input_args[1].clone();
    let mut guard = set_hash.lock().unwrap();

    match guard.get(&key){
        Some(n) => {
            let mut _value: i32 = 1;

            match n.clone().parse::<i32>() {
                Ok(y) => _value = y + 1,
                Err(e) => {
                    println!("{e}");
                    return None
                }
            }

            match guard.insert(key, _value.to_string()) {
                Some(_) => {
                    return Some(_value);
                },
                None => println!("Error in the key value!")
            }

        },
        None => {
            guard.insert(key, 1.to_string());
            return Some(1)

            // let res_ = guard.insert(key, 1.to_string());

            // println!("{:?}", res_);

            // match res_ {
            //     Some(_) => {
            //         return Some(1);
            //     },
            //     None => println!("Error incrementing an invalid key")
            // }
        }
    }
    None
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

pub fn multi(input_args: Vec<String>, qued_commands: &mut Vec<String>) -> Option<String> {
    qued_commands.push(input_args.join(" "));

    return Some(String::from("OK"))
}

pub fn exec(qued_commands: &mut Vec<String>, set_hash: &Arc<Mutex<HashMap<String, String>>>) -> String {
    if qued_commands.is_empty() {
        println!("Empty queue");
        return "*0\r\n".to_string();
    }

    let mut out = format!("*{}\r\n", qued_commands.len());

    for line in qued_commands.drain(..) {
        let args: Vec<String> = line.split_whitespace().map(String::from).collect();

        let res: Option<String> = match args.first().map(String::as_str) {
            Some("SET") | Some("GET") => return set_get(args, set_hash).unwrap(),
            Some("INCR") => return increment(args, set_hash).unwrap().to_string(),
            _ => None,
        };

        match res {
            Some(v) if v == "OK" => out.push_str("+OK\r\n"),
            Some(v) => out.push_str(&format!("${}\r\n{}\r\n", v.len(), v)),
            None => out.push_str("$-1\r\n"),
        }
    }
    out
}