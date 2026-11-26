use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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