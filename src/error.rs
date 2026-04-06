use std::fmt;

#[derive(Debug)]
pub enum FinanceError {
    Database(surrealdb::Error),
    Parse(String),
    Validation(String),
    Io(std::io::Error),
    Unknown(String),
}

impl fmt::Display for FinanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinanceError::Database(e) => write!(f, "数据库错误: {}", e),
            FinanceError::Parse(s) => write!(f, "解析错误: {}", s),
            FinanceError::Validation(s) => write!(f, "验证错误: {}", s),
            FinanceError::Io(e) => write!(f, "IO 错误: {}", e),
            FinanceError::Unknown(s) => write!(f, "未知错误: {}", s),
        }
    }
}

impl std::error::Error for FinanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FinanceError::Database(e) => Some(e),
            FinanceError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<surrealdb::Error> for FinanceError {
    fn from(e: surrealdb::Error) -> Self {
        FinanceError::Database(e)
    }
}

impl From<std::io::Error> for FinanceError {
    fn from(e: std::io::Error) -> Self {
        FinanceError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, FinanceError>;
