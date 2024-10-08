use json::JsonValue;

use crate::error::{SML_Result, SML_Error};


#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    List(Vec<Box<Value>>),
}

impl Value {
    pub fn new(json: &JsonValue) -> SML_Result<Self> {
        if json.is_string() {
            let s = json.as_str().unwrap().to_string();
            Ok(Self::String(s))
        }
        else if json.is_number() {
            Ok(Self::Number(json.as_f64().unwrap()))
        }
        else if json.is_boolean() {
            Ok(Self::Bool(json.as_bool().unwrap()))
        }
        else if json.is_array() {
            let mut list = Vec::new();
            for item in json.members() {
                list.push(Box::new(Value::new(item)?));
            }
            Ok(Self::List(list))
        }
        else {
            Err(SML_Error::JsonFormatError("Value expects a json number, string, array, or boolean. Got null, object, or empty.".to_string()))
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Number(v) => *v != 0.0,
            Self::String(v) => !v.is_empty(),
            Self::List(v) => !v.is_empty(),
        }
    }

    pub fn as_json(&self) -> JsonValue {
        match &self {
            Self::Bool(b) => JsonValue::Boolean(*b),
            Self::String(s) => JsonValue::String(s.to_string()),
            Self::Number(n) => JsonValue::Number((*n).into()),
            Self::List(l) => {
                JsonValue::Array(l.iter().map(|v| v.as_json()).collect())
            }
        }
    }
}


impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s1), Self::String(s2)) => s1 == s2,
            (Self::Bool(b1), Self::Bool(b2)) => b1 == b2,
            (Self::Number(n1), Self::Number(n2)) => n1 == n2,
            (Self::List(l1), Self::List(l2)) => l1 == l2,
            _ => false
        }
    }
}
