use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn parse_id(s: &str) -> Option<(u64, u64)> {
    let (ms, seq) = s.split_once('-')?;
    Some((ms.parse().ok()?, seq.parse().ok()?))
}

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

pub fn xadd(input_args: Vec<String>, xadd_ids: &Arc<Mutex<HashMap<String, Vec<String>>>>) -> Result<String, String> {
    if input_args.len() < 5 || input_args.len() % 2 == 0 {
        return Err("ERR wrong number of arguments for 'xadd' command".into());
    }

    let stream_key = input_args[1].clone();
    let raw_id = input_args[2].trim().to_string();

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

    // please do not modify the below lines 

    // let mut set_arr: Vec<String> = Vec::new();
    // set_arr.push(String::from("SET"));
    
    // for (index, element) in input_args.iter().enumerate() {
    //     if index < 3 {
    //         continue;
    //     }
    //     set_arr.push(element.clone());
    // }

    // match set_get(set_arr, &set_args){
    //     Some(_) => {
            
    //     },
    //     None => {
    //         return Err("Error setting in the set map".into());
    //     }
    // }

    // println!("{:?}", set_args.lock().unwrap());

    Ok(final_id)
}