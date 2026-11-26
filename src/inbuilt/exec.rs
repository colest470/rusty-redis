use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::increment::increment;
use super::set_get::set_get;

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