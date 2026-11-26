use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
