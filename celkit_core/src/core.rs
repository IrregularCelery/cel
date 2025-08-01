use crate::internal::sys::*;
use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    U128(u128),
    I128(i128),
    F32(f32),
    F64(f64),
}

impl ToString for Number {
    fn to_string(&self) -> String {
        match self {
            Number::U8(number) => number.to_string(),
            Number::I8(number) => number.to_string(),
            Number::U16(number) => number.to_string(),
            Number::I16(number) => number.to_string(),
            Number::U32(number) => number.to_string(),
            Number::I32(number) => number.to_string(),
            Number::U64(number) => number.to_string(),
            Number::I64(number) => number.to_string(),
            Number::U128(number) => number.to_string(),
            Number::I128(number) => number.to_string(),
            Number::F32(number) => number.to_string(),
            Number::F64(number) => number.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(Number),
    Text(String),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Struct(String, BTreeMap<String, Value>),
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub context: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: None,
            line: None,
            column: None,
        }
    }

    pub fn with_position(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            message: message.into(),
            context: None,
            line: Some(line),
            column: Some(column),
        }
    }

    pub fn with_context(
        message: impl Into<String>,
        context: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            message: message.into(),
            context: Some(context.into()),
            line: Some(line),
            column: Some(column),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(column)) => {
                write!(
                    f,
                    "Parse error at line {}, column {}: {}",
                    line, column, self.message
                )?;

                if let Some(context) = &self.context {
                    write!(f, "\nContext: {}", context)?;
                }

                Ok(())
            }
            _ => {
                write!(f, "Error: {}", self.message)?;

                Ok(())
            }
        }
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Serialize {
    fn serialize(&self) -> Result<Value>;
}

pub trait Deserialize: Sized {
    fn deserialize(value: Value) -> Result<Self>;
}
