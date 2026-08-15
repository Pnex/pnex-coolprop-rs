//! Shared request/response DTOs used across route modules.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A CoolProp output parameter, addressable either by name (`"Dmolar"`) or by
/// its integer index (as returned by `get_param_index`).
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum Param {
    /// Parameter name, e.g. `"T"`, `"P"`, `"Dmolar"`, `"Hmolar"`, ...
    Name(String),
    /// Integer parameter index from `get_param_index`.
    Index(i64),
}

/// An AbstractState input pair, addressable either by name (`"PT_INPUTS"`)
/// or by its integer index (as returned by `get_input_pair_index`).
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum InputPair {
    /// Input pair name, e.g. `"PT_INPUTS"`, `"PQ_INPUTS"`, `"QT_INPUTS"`, ...
    Name(String),
    /// Integer input pair index from `get_input_pair_index`.
    Index(i64),
}

impl Param {
    /// Resolve to an integer parameter index, validating the name if given.
    /// A string of digits is accepted as a raw index (path parameters always
    /// arrive as strings).
    pub fn resolve(&self) -> Result<i64, crate::safe::CoolPropError> {
        match self {
            Param::Index(i) => Ok(*i),
            Param::Name(s) => {
                if let Ok(i) = s.parse::<i64>() {
                    Ok(i)
                } else {
                    crate::safe::get_param_index(s)
                }
            }
        }
    }
}

impl InputPair {
    /// Resolve to an integer input pair index, validating the name if given.
    /// A string of digits is accepted as a raw index (path parameters always
    /// arrive as strings).
    pub fn resolve(&self) -> Result<i64, crate::safe::CoolPropError> {
        match self {
            InputPair::Index(i) => Ok(*i),
            InputPair::Name(s) => {
                if let Ok(i) = s.parse::<i64>() {
                    Ok(i)
                } else {
                    crate::safe::get_input_pair_index(s)
                }
            }
        }
    }
}

/// Single scalar result.
#[derive(Debug, Serialize, ToSchema)]
pub struct DoubleValue {
    pub value: f64,
}

/// Single string result.
#[derive(Debug, Serialize, ToSchema)]
pub struct StringValue {
    pub value: String,
}

/// Single integer index result.
#[derive(Debug, Serialize, ToSchema)]
pub struct IndexValue {
    pub index: i64,
}

/// Single length result.
#[derive(Debug, Serialize, ToSchema)]
pub struct LengthValue {
    pub length: i64,
}

/// Boolean flag result.
#[derive(Debug, Serialize, ToSchema)]
pub struct FlagValue {
    pub value: bool,
}

/// Simple acknowledgement for mutating calls.
#[derive(Debug, Serialize, ToSchema)]
pub struct Ack {
    pub success: bool,
}

/// 2-D result matrix: `shape` is `[points, outputs]` — one row per input
/// point, one column per requested output (mirrors CoolProp's own
/// `PropsSImulti` layout).
#[derive(Debug, Serialize, ToSchema)]
pub struct MatrixValue {
    pub shape: Vec<usize>,
    /// `values[point][output]`.
    pub values: Vec<Vec<f64>>,
}

/// 1-D result array.
#[derive(Debug, Serialize, ToSchema)]
pub struct ArrayValue {
    pub values: Vec<f64>,
}

/// 1-D string array result.
#[derive(Debug, Serialize, ToSchema)]
pub struct StringArrayValue {
    pub values: Vec<String>,
}
