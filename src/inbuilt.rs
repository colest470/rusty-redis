use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn set_get(set_args: Vec<String>, set_hash: &Arc<Mutex<HashMap<String, String>>>,) -> Option<String> {
    let cmd = set_args.get(0)?.to_uppercase();

    match cmd.as_str() {
        "SET" => {
            let rest = &set_args[1..];

            let mut assignments: Vec<(String, String, Option<u64>)> = Vec::new();
            let mut i = 0;

            while i < rest.len() {
                let key = match rest.get(i) {
                    Some(k) if !is_flag(k) => k.clone(),
                    Some(k) => {
                        eprintln!("Expected key, found flag '{}'", k);
                        return None;
                    }
                    None => break,
                };
                let val = match rest.get(i + 1) {
                    Some(v) if !is_flag(v) => v.clone(),
                    _ => {
                        eprintln!("Missing value for key '{}'", key);
                        return None;
                    }
                };
                i += 2;

                let mut expiry_ms = None;
                if let Some(flag) = rest.get(i) {
                    let flag_upper = flag.to_uppercase();
                    if flag_upper == "EX" || flag_upper == "PX" {
                        let n = rest.get(i + 1)?.parse::<u64>().ok()?;
                        expiry_ms = Some(if flag_upper == "EX" { n * 1000 } else { n });
                        i += 2;
                    }
                }

                assignments.push((key, val, expiry_ms));
            }

            if assignments.is_empty() {
                eprintln!("SET requires at least one key-value pair");
                return None;
            }

            {
                let mut guard = set_hash.lock().unwrap();
                for (k, v, _) in &assignments {
                    guard.insert(k.clone(), v.clone());
                }
            }

            for (key, val, expiry_ms) in assignments {
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
            }

            Some(String::from("OK"))
        }

        "GET" => {
            let key = set_args.get(1)?;
            let guard = set_hash.lock().unwrap();
            match guard.get(key) {
                Some(v) => Some(v.clone()),
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

fn is_flag(token: &str) -> bool {
    matches!(token.to_uppercase().as_str(), "EX" | "PX")
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

pub fn multi(input_args: Vec<String>, qued_commands: &mut Vec<String>) -> String {
    qued_commands.push(input_args.join(" "));

    return String::from("OK");
}

pub fn exec(qued_commands: &mut Vec<String>, set_hash: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let mut out = format!("*{}\r\n", qued_commands.len());

    for line in qued_commands.drain(..) {
        let args: Vec<String> = line
            .split_whitespace()
            .map(String::from)
            .collect();

        match args.first().map(String::as_str) {
            Some("SET") => {
                match set_get(args, set_hash) {
                    Some(_) => out.push_str("+OK\r\n"),
                    None => out.push_str("-ERR command failed\r\n"),
                }
            }

            Some("GET") => {
                match set_get(args, set_hash) {
                    Some(v) => {
                        out.push_str(&format!("${}\r\n{}\r\n", v.len(), v));
                    }
                    None => out.push_str("$-1\r\n"),
                }
            }

            Some("INCR") => {
                match increment(args, set_hash) {
                    Some(v) => out.push_str(&format!(":{}\r\n", v)),
                    None => out.push_str("-ERR value is not an integer or out of range\r\n"),
                }
            }

            _ => out.push_str("-ERR unknown command\r\n"),
        }
    }

    out
}

pub fn type_checker(input_args: Vec<String>, set_hash: &Arc<Mutex<HashMap<String, String>>>, xadd_ids: &Arc<Mutex<HashMap<String, Vec<String>>>>) -> String {
    match xadd_ids.lock().unwrap().get(input_args.last().unwrap().as_str()) {
        Some(v) => {
            println!("{}", v.first().unwrap());
            return String::from("stream");
        }
        None => {
            println!("Stream not found");
        }
    }

    match set_hash.lock().unwrap().get(input_args.last().unwrap()) {
        Some(s) => {
        match s.parse::<i32>(){
            Ok(_) => {
                return String::from("integer");
            },
            Err(_) => return String::from("string")
        }
        },
        None => return String::from("none")
    }
}

pub fn xadd(input_args: Vec<String>, _set_args: &Arc<Mutex<HashMap<String, String>>>, xadd_ids: &Arc<Mutex<HashMap<String, Vec<String>>>>) -> Option<String> {
    if input_args.len() % 2 == 0 {
        return None;
    }

    let mut ids = Vec::new();
    let mut map = xadd_ids.lock().unwrap();
    let mut set_arr = Vec::new();

    for (index, _) in input_args.iter().enumerate(){
        if index == 0 {
            continue;
        } else if index == 1 {
            ids.push(input_args[index+1].clone());
            continue;
        } else if index % 2 == 1{
            ids.push(input_args[index].clone());
            set_arr.push(input_args[index].clone());
        } else {
            set_arr.push(input_args[index].clone());
        }
    }

    set_arr[0] = String::from("SET");

    println!("{:?}: set_arr to be sent to set and get fn", set_arr);
    println!("{:?}: ids to be set in the array and starts with the hash id", ids);
    println!("{:?}: input args", input_args[1]);

    match map.insert(input_args[1].clone(), ids.clone()) {
    None => {
        println!("{:?}", map.clone());
        return Some(ids[0].clone());
    }
    Some(_v) => {
        println!("Error error");
        return None;
    }
}

    // match set_get(set_arr, &set_args) {
    //     None => {
    //         match map.insert(input_args[1].clone(), ids.clone()) { // &Arc<Mutex<HashMap<String, Vec<String>>>> it is the way i want to save my xadd hash keys and the forst element in the vec is the id
    //             Some(_v) => {
    //                 println!("{:?}", map.clone());
    //                 return Some(ids[0].clone());
    //             },
    //             None => {
    //                 println!("Error error");
    //                 return None;
    //             }
    //         }
    //     }
    //     Some(_) => {
    //         return None;
    //     }
    // }

}