use json::JsonValue;

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
    pub fn new(json: &JsonValue) -> anyhow::Result<Self> {
        let store = &json["store"];
        if !store.is_string() {
            anyhow::bail!("Identifier missing store or is not correct type (string)");
        }
        let store = store.as_str().unwrap();
        let store = match store {
            "inputs" => IdentifierStore::Inputs,
            "outputs" => IdentifierStore::Outputs,
            "globals" => IdentifierStore::Globals,
            s => { anyhow::bail!("identifier store wrong value: {s}") }
        };


        let name = &json["name"];
        if !name.is_string() {
            anyhow::bail!("Identifier missing name or is not correct type (string)");
        }
        let name = name.as_str().unwrap().to_string();

        Ok(Self { store, name })
    }

    pub fn get(&self, i: &JsonValue, o: &JsonValue, g: &JsonValue) -> anyhow::Result<Value> {
        todo!();
    }

    pub fn set(&self, o: &mut JsonValue, g: &mut JsonValue, v: &Value) -> anyhow::Result<()> {
        todo!();
        Ok(())
    }
}
