use serde::{Deserialize, Serialize};
use std::fmt;
use surrealdb::Datetime;

/// 账户类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// 银行卡
    BankCard,
    /// 电子钱包（支付宝、微信）
    EWallet,
    /// 现金
    Cash,
    /// 投资理财
    Investment,
    /// 信用卡
    Credit,
    /// 其他
    Other,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountType::BankCard => write!(f, "银行卡"),
            AccountType::EWallet => write!(f, "电子钱包"),
            AccountType::Cash => write!(f, "现金"),
            AccountType::Investment => write!(f, "投资理财"),
            AccountType::Credit => write!(f, "信用卡"),
            AccountType::Other => write!(f, "其他"),
        }
    }
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Other
    }
}

/// 账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// 永久唯一ID（如 "acc_alipay"）
    pub id: String,
    /// 显示名称（可修改）
    pub name: String,
    /// 账户类型
    pub account_type: AccountType,
    /// 父账户ID（用于子账户，如信用卡副卡）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// 创建时间
    pub created_at: Datetime,
}

impl Account {
    /// 创建新账户
    pub fn new(id: impl Into<String>, name: impl Into<String>, account_type: AccountType) -> Self {
        let now = Datetime::from(chrono::Utc::now());
        Self {
            id: id.into(),
            name: name.into(),
            account_type,
            parent_id: None,
            created_at: now,
        }
    }

    /// 设置父账户（子账户）
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_new() {
        let account = Account::new("acc_alipay", "支付宝", AccountType::EWallet);
        assert_eq!(account.id, "acc_alipay");
        assert_eq!(account.name, "支付宝");
        assert!(matches!(account.account_type, AccountType::EWallet));
        assert!(account.parent_id.is_none());
    }

    #[test]
    fn test_account_with_parent() {
        let account =
            Account::new("acc_cmb_li", "招招理财", AccountType::Investment).with_parent("acc_cmb");
        assert_eq!(account.parent_id, Some("acc_cmb".to_string()));
    }

    #[test]
    fn test_account_type_display() {
        assert_eq!(AccountType::BankCard.to_string(), "银行卡");
        assert_eq!(AccountType::EWallet.to_string(), "电子钱包");
        assert_eq!(AccountType::Cash.to_string(), "现金");
    }
}
