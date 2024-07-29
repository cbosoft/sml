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

    #[error("Identifier \"{0}\" doesn't refer to existing value.")]
    IdentifierNameError(String),

    #[error("Identifier error. {0}")]
    IdentifierError(String),

    #[error("Input store is immutable and cannot be written to.")]
    InputsWriteError,

    #[error("Syntax error: {0}")]
    SyntaxError(String),

}


#[allow(non_camel_case_types)]
pub type SML_Result<T> = Result<T, SML_Error>;
