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
    /// 来源账户
    pub account_from: String,
    /// 去向账户/商户/收入方
    pub account_to: Option<String>,
    /// 分类
    pub category: String,
    /// 原始自然语言或备注
    pub description: Option<String>,
    /// 标签
    pub tags: Vec<String>,
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
        account_from: impl Into<String>,
        account_to: Option<impl Into<String>>,
        category: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Self {
        let now = Datetime::from(Utc::now());
        Self {
            id: None,
            timestamp: now.clone(),
            amount,
            currency: currency.into(),
            tx_type,
            account_from: account_from.into(),
            account_to: account_to.map(Into::into),
            category: category.into(),
            description: description.map(Into::into),
            tags: Vec::new(),
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

    /// 设置标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
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
            account_from: String::new(),
            account_to: None,
            category: "其他".to_string(),
            description: None,
            tags: Vec::new(),
            metadata: None,
            created_at: now.clone(),
            updated_at: None,
            source: TxSource::default(),
        }
    }
}
