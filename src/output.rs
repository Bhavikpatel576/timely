use crate::error::TimelyError;
use serde::Serialize;
use serde_json::{json, Value};

pub fn success<T: Serialize>(data: &T) -> Value {
    json!({
        "ok": true,
        "data": serde_json::to_value(data).unwrap_or(Value::Null)
    })
}

pub fn error(err: &TimelyError) -> Value {
    json!({
        "ok": false,
        "error": err.to_string(),
        "error_code": err.error_code()
    })
}

pub fn print_json<T: Serialize>(data: &T) {
    let envelope = success(data);
    println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
}

pub fn print_error_json(err: &TimelyError) {
    let envelope = error(err);
    eprintln!("{}", serde_json::to_string_pretty(&envelope).unwrap());
}
