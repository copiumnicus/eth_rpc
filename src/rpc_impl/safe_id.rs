use serde::{de::Error, Deserialize, Deserializer, Serializer};
use serde_json::Value;
use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
};

/// serializes as str
/// deserializes both string and uint

pub fn serialize<S, V>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Display,
{
    serializer.serialize_str(&format!("{}", value))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Cow::<Value>::deserialize(deserializer)?;
    if let Value::String(n) = value.borrow() {
        return n.parse().map_err(D::Error::custom);
    }
    if let Value::Number(n) = value.borrow() {
        if n.is_u64() {
            return Ok(n.as_u64().unwrap());
        }
    }

    Err(D::Error::custom("cant"))
}
