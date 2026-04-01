pub mod color;
pub mod detail;
pub mod interactive;
pub mod table;

use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("Error formatting JSON: {e}"),
    }
}
