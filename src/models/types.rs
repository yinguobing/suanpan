use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 交易类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxType {
    /// 支出
    Expense,
    /// 收入
    Income,
    /// 转账（账户间）
    Transfer,
    /// 债务变动（借入/偿还债务）
    DebtChange,
    /// 债权变动（借出/收回借款）
    CreditChange,
}

impl fmt::Display for TxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxType::Expense => write!(f, "支出"),
            TxType::Income => write!(f, "收入"),
            TxType::Transfer => write!(f, "转账"),
            TxType::DebtChange => write!(f, "债务变动"),
            TxType::CreditChange => write!(f, "债权变动"),
        }
    }
}

impl FromStr for TxType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "expense" | "支出" => Ok(TxType::Expense),
            "income" | "收入" => Ok(TxType::Income),
            "transfer" | "转账" => Ok(TxType::Transfer),
            "debtchange" | "债务变动" | "借入" | "还债" => Ok(TxType::DebtChange),
            "creditchange" | "债权变动" | "借出" | "收债" => Ok(TxType::CreditChange),
            _ => Err(format!("未知的交易类型: {}", s)),
        }
    }
}

impl Default for TxType {
    fn default() -> Self {
        TxType::Expense
    }
}

/// 数据来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxSource {
    /// AI 解析自然语言
    AiParsed,
    /// 银行账单导入
    CsvImport,
    /// 手动录入
    Manual,
}

impl fmt::Display for TxSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxSource::AiParsed => write!(f, "AI解析"),
            TxSource::CsvImport => write!(f, "CSV导入"),
            TxSource::Manual => write!(f, "手动录入"),
        }
    }
}

impl Default for TxSource {
    fn default() -> Self {
        TxSource::Manual
    }
}
