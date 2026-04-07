use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::Datetime;

use super::types::{TxSource, TxType};

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// 全局唯一标识（数据库自动生成）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<surrealdb::RecordId>,
    /// 交易发生时间
    pub timestamp: Datetime,
    /// 金额
    pub amount: Decimal,
    /// 货币代码：CNY, USD...
    pub currency: String,
    /// 交易类型
    pub tx_type: TxType,
    /// 来源账户ID（外键，关联account表）
    pub account_from_id: String,
    /// 去向账户ID（外键，关联account表）
    pub account_to_id: Option<String>,
    /// 分类ID（外键，关联category表）
    pub category_id: String,
    /// 原始自然语言或备注
    pub description: Option<String>,
    /// 标签ID列表（外键，关联tag表）
    pub tag_ids: Vec<String>,
    /// 任意扩展数据（JSON）
    pub metadata: Option<Value>,
    /// 记录创建时间
    pub created_at: Datetime,
    /// 修改时间
    pub updated_at: Option<Datetime>,
    /// 数据来源
    pub source: TxSource,
}

impl Transaction {
    /// 创建新的交易记录
    pub fn new(
        amount: Decimal,
        currency: impl Into<String>,
        tx_type: TxType,
        account_from_id: impl Into<String>,
        account_to_id: Option<impl Into<String>>,
        category_id: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Self {
        let now = Datetime::from(Utc::now());
        Self {
            id: None,
            timestamp: now.clone(),
            amount,
            currency: currency.into(),
            tx_type,
            account_from_id: account_from_id.into(),
            account_to_id: account_to_id.map(Into::into),
            category_id: category_id.into(),
            description: description.map(Into::into),
            tag_ids: Vec::new(),
            metadata: None,
            created_at: now.clone(),
            updated_at: None,
            source: TxSource::default(),
        }
    }

    /// 设置交易时间
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Datetime::from(timestamp);
        self
    }

    /// 设置标签IDs
    pub fn with_tag_ids(mut self, tag_ids: Vec<String>) -> Self {
        self.tag_ids = tag_ids;
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// 设置数据来源
    pub fn with_source(mut self, source: TxSource) -> Self {
        self.source = source;
        self
    }
}

impl Default for Transaction {
    fn default() -> Self {
        let now = Datetime::from(Utc::now());
        Self {
            id: None,
            timestamp: now.clone(),
            amount: Decimal::ZERO,
            currency: "CNY".to_string(),
            tx_type: TxType::default(),
            account_from_id: String::new(),
            account_to_id: None,
            category_id: String::new(),
            description: None,
            tag_ids: Vec::new(),
            metadata: None,
            created_at: now.clone(),
            updated_at: None,
            source: TxSource::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_transaction_new() {
        let tx = Transaction::new(
            dec!(100.50),
            "CNY",
            TxType::Expense,
            "acc_alipay",      // account_from_id
            Some("acc_supermarket"), // account_to_id
            "cat_shopping",    // category_id
            Some("日用品"),
        );

        assert_eq!(tx.amount, dec!(100.50));
        assert_eq!(tx.currency, "CNY");
        assert!(matches!(tx.tx_type, TxType::Expense));
        assert_eq!(tx.account_from_id, "acc_alipay");
        assert_eq!(tx.account_to_id, Some("acc_supermarket".to_string()));
        assert_eq!(tx.category_id, "cat_shopping");
        assert_eq!(tx.description, Some("日用品".to_string()));
        assert!(tx.tag_ids.is_empty());
        assert!(tx.metadata.is_none());
        assert!(matches!(tx.source, TxSource::Manual));
    }

    #[test]
    fn test_transaction_with_tag_ids() {
        let tx = Transaction::new(
            dec!(50),
            "CNY",
            TxType::Expense,
            "acc_cash",
            None::<String>,
            "cat_food",
            None::<String>,
        )
        .with_tag_ids(vec!["tag_work".to_string(), "tag_monday".to_string()]);

        assert_eq!(tx.tag_ids.len(), 2);
        assert_eq!(tx.tag_ids[0], "tag_work");
        assert_eq!(tx.tag_ids[1], "tag_monday");
    }

    #[test]
    fn test_transaction_with_source() {
        let tx = Transaction::new(
            dec!(1000),
            "CNY",
            TxType::Income,
            "acc_company",
            Some("acc_cmb"),
            "cat_salary",
            Some("三月工资"),
        )
        .with_source(TxSource::CsvImport);

        assert!(matches!(tx.source, TxSource::CsvImport));
    }

    #[test]
    fn test_transaction_default() {
        let tx = Transaction::default();

        assert_eq!(tx.amount, Decimal::ZERO);
        assert_eq!(tx.currency, "CNY");
        assert!(matches!(tx.tx_type, TxType::Expense));
        assert!(tx.account_from_id.is_empty());
        assert!(tx.category_id.is_empty());
        assert!(tx.tag_ids.is_empty());
    }

    #[test]
    fn test_transaction_without_optional_fields() {
        let tx = Transaction::new(
            dec!(35),
            "CNY",
            TxType::Expense,
            "acc_alipay",
            None::<String>,
            "cat_food",
            None::<String>,
        );

        assert_eq!(tx.account_to_id, None);
        assert_eq!(tx.description, None);
    }
}
