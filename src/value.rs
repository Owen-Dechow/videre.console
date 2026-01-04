use std::fmt::Display;

use serde_json::{Number, Value};
use unicode_width::UnicodeWidthStr;

use crate::cells::VidereMap;

#[derive(Debug)]
pub enum VidereValue {
    Number(Number),
    String(String),
    Bool(bool),
    Null,
    Array,
    Object,
}

impl Display for VidereValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VidereValue::Number(number) => number.to_string(),
                VidereValue::String(s) => format!("\"{s}\""),
                VidereValue::Bool(b) => b.to_string(),
                VidereValue::Null => String::from("null"),
                VidereValue::Array => String::from("[]"),
                VidereValue::Object => String::from("{}"),
            }
        )
    }
}

impl VidereValue {
    pub fn from_json_val(map: &mut VidereMap, layer: usize, val: Value) -> Self {
        match val {
            Value::Null => VidereValue::Null,
            Value::Bool(b) => VidereValue::Bool(b),
            Value::Number(n) => VidereValue::Number(n),
            Value::String(s) => VidereValue::String(s),
            Value::Array(arr) => {
                map.add_arr_to_layer(layer + 1, arr);
                VidereValue::Array
            }
            Value::Object(obj) => {
                map.add_obj_to_layer(layer + 1, obj);
                VidereValue::Object
            }
        }
    }

    pub fn get_min_width(&self) -> usize {
        match self {
            VidereValue::Number(number) => number.to_string().len(),
            VidereValue::String(s) => s.width() + 2,
            VidereValue::Bool(b) => match b {
                true => 4,
                false => 5,
            },
            VidereValue::Null => 4,
            VidereValue::Array | VidereValue::Object => 2,
        }
    }
}

