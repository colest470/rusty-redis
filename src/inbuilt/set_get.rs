use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn set_get(set_args: Vec<String>, set_hash: &Arc<Mutex<HashMap<String, String>>>) -> Option<String> {
    let cmd = set_args.first()?.to_uppercase();

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
                    set_timeout(move || {
                        let mut guard = map.lock().unwrap();
                        if guard.get(&key).map(String::as_str) == Some(val.as_str()) {
                            guard.remove(&key);
                        }
                    }, ms);
                }
            }

            Some("OK".to_string())
        }
        "GET" => {
            let key = set_args.get(1)?;
            let guard = set_hash.lock().unwrap();
            guard.get(key).cloned().or_else(|| {
                println!("Get key invalid!");
                None
            })
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

fn set_timeout<F>(callback: F, delay_ms: u64)
where
    F: FnOnce() + Send + 'static,
{
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay_ms));
        callback();
    });
}