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

            // Parse optional expiry: EX <seconds> or PX <milliseconds>
            let expiry_ms = match (set_args.get(3).map(String::as_str), set_args.get(4)) {
                (Some("EX"), Some(n)) => Some(n.parse::<u64>().ok()? * 1000),
                (Some("PX"), Some(n)) => Some(n.parse::<u64>().ok()?),
                _ => None,
            };

            // Insert immediately.
            set_hash.lock().unwrap().insert(key.clone(), val.clone());

            // If TTL given, schedule removal.
            if let Some(ms) = expiry_ms {
                let map = Arc::clone(set_hash);
                set_timeout(
                    move || {
                        let mut guard = map.lock().unwrap();
                        // Only remove if the value hasn't changed since.
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

fn set_timeout<F>(callback: F, delay_ms: u64)
where
    F: FnOnce() + Send + 'static,
{
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay_ms));
        callback();
    });
}