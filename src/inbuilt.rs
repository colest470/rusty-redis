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

use std::time::{SystemTime, UNIX_EPOCH};

fn parse_id(s: &str) -> Option<(u64, u64)> {
    let (ms, seq) = s.split_once('-')?;
    Some((ms.parse().ok()?, seq.parse().ok()?))
}

/// Given a time part `ms`, find the next sequence number to use in this stream.
/// Returns `None` only if `ms-0` would violate the monotonicity rule (not needed
/// here, but kept for symmetry / future refactors).
fn next_seq(existing: Option<&Vec<String>>, ms: u64) -> u64 {
    let default = if ms == 0 { 1 } else { 0 };
    match existing {
        Some(entries) => entries
            .iter()
            .rev()
            .find_map(|s| {
                let (m, seq) = parse_id(s)?;
                if m == ms { Some(seq) } else { None }
            })
            .map(|last_seq| last_seq + 1)
            .unwrap_or(default),
        None => default,
    }
}

pub fn xadd(input_args: Vec<String>, _set_args: &Arc<Mutex<HashMap<String, String>>>, xadd_ids: &Arc<Mutex<HashMap<String, Vec<String>>>>) -> Result<String, String> {
    if input_args.len() < 5 || input_args.len() % 2 == 0 {
        return Err("ERR wrong number of arguments for 'xadd' command".into());
    }

    let stream_key = input_args[1].clone();
    let raw_id = input_args[2].clone();

    let mut map = xadd_ids.lock().unwrap();

    let (new_ms, new_seq) = if raw_id == "*" {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "ERR system clock before UNIX epoch".to_string())?
            .as_millis() as u64;

        let existing = map.get(&stream_key);

        let seq = if let Some(last) = existing.and_then(|e| e.iter().rev().find_map(|s| parse_id(s))) {
            let (last_ms, last_seq) = last;
            if now_ms > last_ms {
                next_seq(existing, now_ms)
            } else {
                last_seq + 1
            }
        } else {
            next_seq(existing, now_ms)
        };

        let (new_ms, new_seq) = if let Some(last) = existing.and_then(|e| e.iter().rev().find_map(|s| parse_id(s))) {
            let (last_ms, last_seq) = last;
            if now_ms > last_ms {
                (now_ms, seq)
            } else {
                (last_ms, last_seq + 1)
            }
        } else {
            (now_ms, seq)
        };

        (new_ms, new_seq)
    } else {
        match raw_id.split_once('-') {
            Some((ms_str, "*")) => {
                let ms: u64 = ms_str.parse().map_err(|_| {
                    "ERR Invalid stream ID specified as stream command argument".to_string()
                })?;
                let seq = next_seq(map.get(&stream_key), ms);
                (ms, seq)
            }
            Some(_) => match parse_id(&raw_id) {
                Some(p) => p,
                None => {
                    return Err(
                        "ERR Invalid stream ID specified as stream command argument".into(),
                    )
                }
            },
            None => {
                return Err(
                    "ERR Invalid stream ID specified as stream command argument".into(),
                )
            }
        }
    };

    if new_ms == 0 && new_seq == 0 {
        return Err("ERR The ID specified in XADD must be greater than 0-0".into());
    }

    if let Some(existing) = map.get(&stream_key) {
        if let Some(last) = existing.iter().rev().find_map(|s| parse_id(s)) {
            if (new_ms, new_seq) <= last {
                return Err(
                    "ERR The ID specified in XADD is equal or smaller than the target stream top item"
                        .into(),
                );
            }
        }
    }

    let final_id = format!("{}-{}", new_ms, new_seq);

    let mut ids = Vec::new();
    ids.push(final_id.clone());
    for pair in input_args[3..].chunks(2) {
        ids.push(pair[0].clone());
        ids.push(pair[1].clone());
    }

    map.entry(stream_key).or_default().extend(ids);

    Ok(final_id)
}