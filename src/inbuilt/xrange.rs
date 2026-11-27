use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn xrange(input_arg: Vec<String>, xadd_ids: &Arc<Mutex<HashMap<String, Vec<String>>>>) -> Option<String> {
    if input_arg.len() < 4 {
        return None;
    }
    println!("{:?}", input_arg);

    let stream_key = &input_arg[1];
    
    let start_id_str = &input_arg[2];
    let end_id_str = &input_arg[3];

    let mut arr_range = "";

    if input_arg[2] == String::from("-") {
        arr_range = input_arg[3].as_str();
    }

    let start_id = if let Some((_ms, _)) = start_id_str.split_once('-') {
        start_id_str.clone()
    } else {
        format!("{}-0", start_id_str)
    };

    let end_id = if let Some((_ms, _)) = end_id_str.split_once('-') {
        end_id_str.clone()
    } else {
        format!("{}-18446744073709551615", end_id_str)
    };

    let start_tuple = parse_id(&start_id)?;
    let end_tuple = parse_id(&end_id)?;

    let xadd_guard = xadd_ids.lock().unwrap();
    
    let mut response = format!("*0\r\n"); 

    match xadd_guard.get(stream_key) {
        Some(entries) => {
            let mut valid_entries: Vec<(String, Vec<String>)> = Vec::new();
            
            let mut i = 0;
            while i < entries.len() {
                let entry_id = &entries[i];
                if let Some((ms, seq)) = parse_id(entry_id) {
                    let current_tuple = (ms, seq);
                    
                    if current_tuple >= start_tuple && current_tuple <= end_tuple {
                        let mut fields_and_values = Vec::new();
                        let mut j = i + 1;
                        while j < entries.len() {
                            if entries[j].contains('-') && entries[j].split('-').next().unwrap().parse::<u64>().is_ok() {
                                break;
                            }
                            fields_and_values.push(entries[j].clone());
                            j += 1;
                        }
                        
                        valid_entries.push((entry_id.clone(), fields_and_values));
                        i = j;
                        continue;
                    }
                }
                i += 1;
            }
                        
            if !arr_range.is_empty() {
                if let Some((start_str, end_str)) = arr_range.split_once('-') {
                    let start: usize = start_str.trim().parse().expect("Invalid start index");
                    let end: usize = end_str.trim().parse().expect("Invalid end index");

                    valid_entries = valid_entries[start..=end].to_vec();
                }
            }


            if !valid_entries.is_empty() {
                response = format!("*{}\r\n", valid_entries.len());
                for (id, kv_pairs) in valid_entries {
                    response.push_str("*2\r\n");
                    
                    response.push_str(&format!("${}\r\n{}\r\n", id.len(), id));
                    
                    response.push_str(&format!("*{}\r\n", kv_pairs.len()));
                    for kv in kv_pairs {
                        response.push_str(&format!("${}\r\n{}\r\n", kv.len(), kv));
                    }
                }
            }
        },
        None => {
            return None;
        }
    };

    Some(response)
}

fn parse_id(s: &str) -> Option<(u64, u64)> {
    let (ms, seq) = s.split_once('-')?;
    Some((ms.parse().ok()?, seq.parse().ok()?))
}