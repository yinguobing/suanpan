use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::Zero;
use surrealdb::Datetime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::SurrealKv;
use surrealdb::RecordId;
use surrealdb::Surreal;
use std::path::Path;

use crate::error::{FinanceError, Result};
use crate::models::Transaction;

/// 数据库封装
#[derive(Debug)]
pub struct Database {
    db: Surreal<surrealdb::engine::local::Db>,
}

impl Database {
    /// 初始化文件数据库
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Surreal::new::<SurrealKv>(path.as_ref()).await?;
        db.use_ns("finance").use_db("finance").await?;

        let db = Self { db };
        db.init().await?;
        Ok(db)
    }

    /// 初始化数据库表结构
    async fn init(&self) -> Result<()> {
        // 定义交易记录表
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS transaction SCHEMAFULL;
                
                DEFINE FIELD IF NOT EXISTS id ON transaction TYPE record;
                DEFINE FIELD IF NOT EXISTS timestamp ON transaction TYPE datetime;
                DEFINE FIELD IF NOT EXISTS amount ON transaction TYPE decimal;
                DEFINE FIELD IF NOT EXISTS currency ON transaction TYPE string;
                DEFINE FIELD IF NOT EXISTS tx_type ON transaction TYPE string;
                DEFINE FIELD IF NOT EXISTS account_from ON transaction TYPE string;
                DEFINE FIELD IF NOT EXISTS account_to ON transaction TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS category ON transaction TYPE string;
                DEFINE FIELD IF NOT EXISTS description ON transaction TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS tags ON transaction TYPE option<array<string>>;
                DEFINE FIELD IF NOT EXISTS metadata ON transaction TYPE option<object>;
                DEFINE FIELD IF NOT EXISTS created_at ON transaction TYPE datetime;
                DEFINE FIELD IF NOT EXISTS updated_at ON transaction TYPE option<datetime>;
                DEFINE FIELD IF NOT EXISTS source ON transaction TYPE string;
                
                DEFINE INDEX IF NOT EXISTS idx_timestamp ON transaction COLUMNS timestamp;
                DEFINE INDEX IF NOT EXISTS idx_category ON transaction COLUMNS category;
                DEFINE INDEX IF NOT EXISTS idx_account_from ON transaction COLUMNS account_from;
                DEFINE INDEX IF NOT EXISTS idx_tx_type ON transaction COLUMNS tx_type;
                "#,
            )
            .await?;
        Ok(())
    }

    /// 创建交易记录
    pub async fn create_transaction(&self, tx: Transaction) -> Result<Transaction> {
        let created: Option<Transaction> = self
            .db
            .create("transaction")
            .content(tx)
            .await
            .map_err(FinanceError::Database)?;

        created.ok_or_else(|| FinanceError::Unknown("创建交易失败".to_string()))
    }

    /// 列出最近的交易记录
    pub async fn list_transactions(&self, limit: usize) -> Result<Vec<Transaction>> {
        let sql = "SELECT * FROM transaction ORDER BY timestamp DESC LIMIT $limit";
        let mut result = self
            .db
            .query(sql)
            .bind(("limit", limit as i64))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;
        Ok(transactions)
    }

    /// 按日期范围查询
    pub async fn query_by_date_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Transaction>> {
        let sql = "SELECT * FROM transaction WHERE timestamp >= $from AND timestamp <= $to ORDER BY timestamp DESC";
        let mut result = self
            .db
            .query(sql)
            .bind(("from", Datetime::from(from)))
            .bind(("to", Datetime::from(to)))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;
        Ok(transactions)
    }

    /// 按分类查询
    pub async fn query_by_category(&self, category: &str) -> Result<Vec<Transaction>> {
        let sql = "SELECT * FROM transaction WHERE category = $category ORDER BY timestamp DESC";
        let mut result = self
            .db
            .query(sql)
            .bind(("category", category.to_string()))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;
        Ok(transactions)
    }

    /// 按交易类型查询
    pub async fn query_by_type(&self, tx_type: &str) -> Result<Vec<Transaction>> {
        let sql = "SELECT * FROM transaction WHERE tx_type = $tx_type ORDER BY timestamp DESC";
        let mut result = self
            .db
            .query(sql)
            .bind(("tx_type", tx_type.to_string()))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;
        Ok(transactions)
    }

    /// 获取月度统计
    pub async fn get_monthly_stats(
        &self,
        year: i32,
        month: u32,
    ) -> Result<MonthlyStats> {
        let start = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| FinanceError::Validation("无效的日期".to_string()))?;
        let end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or_else(|| FinanceError::Validation("无效的日期".to_string()))?;

        let start_dt = start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_dt = end.and_hms_opt(0, 0, 0).unwrap().and_utc();

        // 使用 SurrealDB 的 Datetime 类型
        let sql = "SELECT * FROM transaction WHERE timestamp >= $from AND timestamp < $to";
        let mut result = self
            .db
            .query(sql)
            .bind(("from", Datetime::from(start_dt)))
            .bind(("to", Datetime::from(end_dt)))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;

        let mut total_income = Decimal::ZERO;
        let mut total_expense = Decimal::ZERO;
        let mut category_breakdown: std::collections::HashMap<String, Decimal> =
            std::collections::HashMap::new();

        for tx in &transactions {
            match tx.tx_type {
                crate::models::TxType::Income => total_income += tx.amount,
                crate::models::TxType::Expense => {
                    total_expense += tx.amount;
                    *category_breakdown
                        .entry(tx.category.clone())
                        .or_insert_with(Decimal::zero) += tx.amount;
                }
                _ => {}
            }
        }

        Ok(MonthlyStats {
            year,
            month,
            total_income,
            total_expense,
            net: total_income - total_expense,
            transaction_count: transactions.len(),
            category_breakdown,
        })
    }

    /// 删除交易记录
    pub async fn delete_transaction(&self, id: RecordId) -> Result<()> {
        self.db
            .delete::<Option<Transaction>>(id)
            .await
            .map_err(FinanceError::Database)?;
        Ok(())
    }
}

/// 月度统计
#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyStats {
    pub year: i32,
    pub month: u32,
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub net: Decimal,
    pub transaction_count: usize,
    pub category_breakdown: std::collections::HashMap<String, Decimal>,
}
