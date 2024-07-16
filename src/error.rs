use std::io;

#[allow(non_camel_case_types)]
#[derive(thiserror::Error, Debug)]
pub enum SML_Error {

    #[error("Failed to serialize")]
    SerializeError(#[from] serde_json::Error),

    #[error("Failed to parse")]
    JsonParseError(#[from] json::Error),

    #[error("Failed to read/write file")]
    IOError(#[from] io::Error),

    #[error("JSON formatting error. {0}")]
    JsonFormatError(String),

    #[error("SML Syntax error error. {0}")]
    BadOperation(String),

    #[error("Nonexistant state {0}")]
    NonexistantState(String),

}


#[allow(non_camel_case_types)]
pub type SML_Result<T> = Result<T, SML_Error>;
