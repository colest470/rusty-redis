use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn xrange(input_arg: Vec<String>, xadd_ids: &Arc<Mutex<HashMap<String, Vec<String>>>>) -> Option<String> {
    if input_arg.len() < 4 {
        return None;
    }

    let stream_key = &input_arg[1];

    let start_id_str = &input_arg[2];
    let end_id_str = &input_arg[3];

    let start_tuple = parse_range_bound(start_id_str, true)?;
    let end_tuple = parse_range_bound(end_id_str, false)?;

    let entries_snapshot: Vec<String> = {
        let xadd_guard = xadd_ids.lock().unwrap();
        match xadd_guard.get(stream_key) {
            Some(entries) => entries.clone(),
            None => Vec::new(), 
        }
    };

    let mut valid_entries: Vec<(String, Vec<String>)> = Vec::new();

    let mut i = 0;
    while i < entries_snapshot.len() {
        let entry_id = &entries_snapshot[i];
        if let Some(current_tuple) = parse_id(entry_id) {
            if current_tuple >= start_tuple && current_tuple <= end_tuple {
                let mut fields_and_values = Vec::new();
                let mut j = i + 1;
                while j < entries_snapshot.len() {
                    if is_entry_id(&entries_snapshot[j]) {
                        break;
                    }
                    fields_and_values.push(entries_snapshot[j].clone());
                    j += 1;
                }

                valid_entries.push((entry_id.clone(), fields_and_values));
                i = j;
                continue;
            }
        }
        i += 1;
    }

    let mut response = format!("*{}\r\n", valid_entries.len());
    for (id, kv_pairs) in valid_entries {
        response.push_str("*2\r\n");

        response.push_str(&format!("${}\r\n{}\r\n", id.len(), id));

        response.push_str(&format!("*{}\r\n", kv_pairs.len()));
        for kv in kv_pairs {
            response.push_str(&format!("${}\r\n{}\r\n", kv.len(), kv));
        }
    }

    Some(response)
}

fn parse_range_bound(s: &str, is_start: bool) -> Option<(u64, u64)> {
    match s {
        "-" => Some((0, 0)),
        "+" => Some((u64::MAX, u64::MAX)),
        _ => {
            if let Some((ms, seq)) = s.split_once('-') {
                Some((ms.parse().ok()?, seq.parse().ok()?))
            } else {
                let ms: u64 = s.parse().ok()?;
                if is_start {
                    Some((ms, 0))
                } else {
                    Some((ms, u64::MAX))
                }
            }
        }
    }
}

fn is_entry_id(s: &str) -> bool {
    matches!(parse_id(s), Some(_))
}

fn parse_id(s: &str) -> Option<(u64, u64)> {
    let (ms, seq) = s.split_once('-')?;
    Some((ms.parse().ok()?, seq.parse().ok()?))
}