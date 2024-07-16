use json::JsonValue;

use crate::error::{SML_Error, SML_Result};
use crate::value::Value;


#[derive(Debug)]
pub enum IdentifierStore {
    Inputs,
    Outputs,
    Globals
}

#[derive(Debug)]
pub struct Identifier {
    store: IdentifierStore,
    name: String
}

impl Identifier {
    pub fn new(json: &JsonValue) -> SML_Result<Self> {
        let store = &json["store"];
        if !store.is_string() {
            return Err(SML_Error::JsonFormatError("Identifier missing `store` or is not correct type".to_string()));
        }
        let store = store.as_str().unwrap();
        let store = match store {
            "inputs" => IdentifierStore::Inputs,
            "outputs" => IdentifierStore::Outputs,
            "globals" => IdentifierStore::Globals,
            s => { return Err(SML_Error::JsonFormatError(format!("Identifier `store` got unexpected value {store:?}. "))); }
        };


        let name = &json["name"];
        if !name.is_string() {
            return Err(SML_Error::JsonFormatError("Identifier missing `name` or is not correct type".to_string()));
        }
        let name = name.as_str().unwrap().to_string();

        Ok(Self { store, name })
    }

    pub fn get(&self, i: &JsonValue, o: &JsonValue, g: &JsonValue) -> SML_Result<Value> {
        todo!();
    }

    pub fn set(&self, o: &mut JsonValue, g: &mut JsonValue, v: &Value) -> SML_Result<()> {
        todo!();
        Ok(())
    }
}
