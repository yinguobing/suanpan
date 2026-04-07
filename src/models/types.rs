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
    /// 银行账单导入
    CsvImport,
    /// 手动录入（CLI 参数）
    Manual,
}

impl fmt::Display for TxSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_type_from_str() {
        // 英文
        assert_eq!(TxType::from_str("expense").unwrap(), TxType::Expense);
        assert_eq!(TxType::from_str("income").unwrap(), TxType::Income);
        assert_eq!(TxType::from_str("transfer").unwrap(), TxType::Transfer);
        assert_eq!(TxType::from_str("debtchange").unwrap(), TxType::DebtChange);
        assert_eq!(TxType::from_str("creditchange").unwrap(), TxType::CreditChange);

        // 中文
        assert_eq!(TxType::from_str("支出").unwrap(), TxType::Expense);
        assert_eq!(TxType::from_str("收入").unwrap(), TxType::Income);
        assert_eq!(TxType::from_str("转账").unwrap(), TxType::Transfer);
        assert_eq!(TxType::from_str("债务变动").unwrap(), TxType::DebtChange);
        assert_eq!(TxType::from_str("债权变动").unwrap(), TxType::CreditChange);

        // 大小写不敏感
        assert_eq!(TxType::from_str("EXPENSE").unwrap(), TxType::Expense);
        assert_eq!(TxType::from_str("Income").unwrap(), TxType::Income);

        // 错误情况
        assert!(TxType::from_str("unknown").is_err());
    }

    #[test]
    fn test_tx_type_display() {
        assert_eq!(TxType::Expense.to_string(), "支出");
        assert_eq!(TxType::Income.to_string(), "收入");
        assert_eq!(TxType::Transfer.to_string(), "转账");
        assert_eq!(TxType::DebtChange.to_string(), "债务变动");
        assert_eq!(TxType::CreditChange.to_string(), "债权变动");
    }

    #[test]
    fn test_tx_type_default() {
        assert_eq!(TxType::default(), TxType::Expense);
    }

    #[test]
    fn test_tx_source_display() {
        assert_eq!(TxSource::CsvImport.to_string(), "CSV导入");
        assert_eq!(TxSource::Manual.to_string(), "手动录入");
    }

    #[test]
    fn test_tx_source_default() {
        assert_eq!(TxSource::default(), TxSource::Manual);
    }
}
