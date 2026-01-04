mod cells;
mod connections;
mod value;

use std::io::Error;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use serde_json::Value;

use crate::cells::VidereMap;

fn run() -> Result<(), Error> {
    enable_raw_mode()?;

    return Ok(());
}

fn m() {
    if let Err(err) = run() {
        println!("Error while running: {err}");
    }

    if let Err(err) = disable_raw_mode() {
        println!("Error restoring terminal: {err}");
    }
}

const DATA: &str = r#"
{
    "long_array_example": [
        "text_a",
        "text_b",
        "text_c",
        "text_d",
        "text_e",
        "text_f",
        "text_g",
        "text_h",
        [
            "FIRST"
        ],
        [
            "SECOND"
        ],
        [
            "THIRD", null
        ]
    ],
    "example": {
        "test": "This is some test data",
        "empty_table": {},
        "empty_array": [],
        "nested_table": {
            "array": [
                "data\n"
            ],
            "string": "MY_\nSTRING",
            "data": 5
        }
    }
}
"#;

fn main() {
    let data: Value = serde_json::from_str(&DATA.replace("\\", "\\\\")).unwrap();

    if let Value::Object(data) = data {
        let map = VidereMap::from_json_obj(data);
        println!("{}", map.as_table_string())
    }
}
