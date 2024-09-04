use thiserror::Error;

#[derive(Error, Debug)]
pub enum SelfError {
    #[error("系统异常")]
    DefaultErr(String),
}