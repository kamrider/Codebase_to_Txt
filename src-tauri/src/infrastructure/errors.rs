use std::fmt::Display;

pub const E_ROOT_REQUIRED: &str = "E_ROOT_REQUIRED";
pub const E_ROOT_INVALID: &str = "E_ROOT_INVALID";
pub const E_ROOT_NOT_DIR: &str = "E_ROOT_NOT_DIR";
pub const E_PATH_OUTSIDE_ROOT: &str = "E_PATH_OUTSIDE_ROOT";
pub const E_DIRPATH_NOT_DIR: &str = "E_DIRPATH_NOT_DIR";
pub const E_OUTPUT_REQUIRED: &str = "E_OUTPUT_REQUIRED";
pub const E_OUTPUT_IS_DIR: &str = "E_OUTPUT_IS_DIR";
#[allow(dead_code)]
pub const E_OUTPUT_EXISTS: &str = "E_OUTPUT_EXISTS";
pub const E_IO_READ: &str = "E_IO_READ";
pub const E_IO_WRITE: &str = "E_IO_WRITE";
pub const E_RULE_INVALID_GLOB: &str = "E_RULE_INVALID_GLOB";

pub fn coded(code: &str, message: impl Into<String>) -> String {
    format!("[{code}] {}", message.into())
}

pub fn read_error(context: &str, error: impl Display) -> String {
    coded(E_IO_READ, format!("{context}: {error}"))
}

pub fn write_error(context: &str, error: impl Display) -> String {
    coded(E_IO_WRITE, format!("{context}: {error}"))
}
