use json::JsonValue;

use crate::error::{SML_Result, SML_Error};


#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
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
        else {
            Err(SML_Error::JsonFormatError("Value expects a json number, string, or boolean. Got null, object, array, or empty.".to_string()))
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::Number(v) => *v != 0.0,
            Self::String(v) => v.is_empty(),

    pub fn as_json(&self) -> JsonValue {
        match &self {
            Self::Bool(b) => JsonValue::Boolean(*b),
            Self::String(s) => JsonValue::String(s.to_string()),
            Self::Number(n) => JsonValue::Number((*n).into()),
        }
    }
}


        }
    }
}
