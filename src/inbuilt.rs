use std::collections::HashMap;

// struct Maps {
//     hash: HashMap<String, String>
// }

pub fn set_get(set_args: Vec<String>, set_hash: &mut HashMap<String, String>) -> Option<String>{
    //let mut set_hash: HashMap<String, String> = HashMap::new();// creates a new map instead

    match set_args[0].as_str() {
        "SET" => {
            set_hash.insert(set_args[1].clone(), set_args[2].clone());
            // let _: Maps = Maps {
            //     hash: &set_hash
            // };
            Some(String::from("OK"))
        },
        "GET" => {
            match set_hash.get(&set_args[1]) {
                Some(n) => {
                    Some(n.clone())
                }
                _ => {
                    println!("Get key invalid!");
                    None
                }
            }
        },
        _ => {
            println!("Command unrecognized");
            None
        }
    }
}