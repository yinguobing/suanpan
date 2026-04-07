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
use crate::models::{Account, Category, Tag, Transaction, TxType};

/// 交易记录更新参数（部分更新）
#[derive(Debug, Default)]
pub struct TransactionUpdate {
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub tx_type: Option<TxType>,
    pub account_from_id: Option<String>,
    pub account_to_id: Option<Option<String>>,
    pub category_id: Option<String>,
    pub description: Option<Option<String>>,
    pub tag_ids: Option<Vec<String>>,
}

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
        // 定义账户表
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS account SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS id ON account TYPE string;
                DEFINE FIELD IF NOT EXISTS name ON account TYPE string;
                DEFINE FIELD IF NOT EXISTS account_type ON account TYPE string;
                DEFINE FIELD IF NOT EXISTS parent_id ON account TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS created_at ON account TYPE datetime;
                DEFINE INDEX IF NOT EXISTS idx_account_id ON account COLUMNS id UNIQUE;
                "#,
            )
            .await?;

        // 定义分类表（支持层级）
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS category SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS id ON category TYPE string;
                DEFINE FIELD IF NOT EXISTS name ON category TYPE string;
                DEFINE FIELD IF NOT EXISTS parent_id ON category TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS full_path ON category TYPE string;
                DEFINE FIELD IF NOT EXISTS level ON category TYPE int;
                DEFINE FIELD IF NOT EXISTS created_at ON category TYPE datetime;
                DEFINE INDEX IF NOT EXISTS idx_category_id ON category COLUMNS id UNIQUE;
                DEFINE INDEX IF NOT EXISTS idx_category_parent ON category COLUMNS parent_id;
                DEFINE INDEX IF NOT EXISTS idx_category_path ON category COLUMNS full_path;
                "#,
            )
            .await?;

        // 定义标签表
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS tag SCHEMAFULL;
                DEFINE FIELD IF NOT EXISTS id ON tag TYPE string;
                DEFINE FIELD IF NOT EXISTS name ON tag TYPE string;
                DEFINE FIELD IF NOT EXISTS color ON tag TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS created_at ON tag TYPE datetime;
                DEFINE INDEX IF NOT EXISTS idx_tag_id ON tag COLUMNS id UNIQUE;
                "#,
            )
            .await?;

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
                DEFINE FIELD IF NOT EXISTS account_from_id ON transaction TYPE string;
                DEFINE FIELD IF NOT EXISTS account_to_id ON transaction TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS category_id ON transaction TYPE string;
                DEFINE FIELD IF NOT EXISTS description ON transaction TYPE option<string>;
                DEFINE FIELD IF NOT EXISTS tag_ids ON transaction TYPE option<array<string>>;
                DEFINE FIELD IF NOT EXISTS metadata ON transaction TYPE option<object>;
                DEFINE FIELD IF NOT EXISTS created_at ON transaction TYPE datetime;
                DEFINE FIELD IF NOT EXISTS updated_at ON transaction TYPE option<datetime>;
                DEFINE FIELD IF NOT EXISTS source ON transaction TYPE string;
                
                DEFINE INDEX IF NOT EXISTS idx_timestamp ON transaction COLUMNS timestamp;
                DEFINE INDEX IF NOT EXISTS idx_tx_category ON transaction COLUMNS category_id;
                DEFINE INDEX IF NOT EXISTS idx_account_from ON transaction COLUMNS account_from_id;
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
                    // TODO: 批次2将添加根据category_id查询category.name
                    *category_breakdown
                        .entry(tx.category_id.clone())
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

    /// 删除交易记录（通过完整 RecordId）
    pub async fn delete_transaction(&self, id: RecordId) -> Result<()> {
        self.db
            .delete::<Option<Transaction>>(id)
            .await
            .map_err(FinanceError::Database)?;
        Ok(())
    }

    /// 根据短 ID 查找交易记录（匹配前 12 位）
    async fn find_by_short_id(&self, short_id: &str) -> Result<Option<(RecordId, Transaction)>> {
        // 短 ID 应该是 12 位字母数字
        if short_id.len() != 12 {
            return Err(FinanceError::Validation(
                "短 ID 应为 12 位字符".to_string(),
            ));
        }

        // 将 RecordId 转为字符串后比较前 12 位
        let sql = "SELECT * FROM transaction WHERE string::starts_with(<string> id, $prefix)";
        let mut result = self
            .db
            .query(sql)
            .bind(("prefix", format!("transaction:{}", short_id)))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;

        if transactions.is_empty() {
            Ok(None)
        } else {
            // 返回第一个匹配的记录
            let tx = transactions.into_iter().next().unwrap();
            let id = tx.id.clone().ok_or_else(|| {
                FinanceError::Unknown("交易记录缺少 ID".to_string())
            })?;
            Ok(Some((id, tx)))
        }
    }

    /// 根据短 ID 删除交易记录
    pub async fn delete_by_short_id(&self, short_id: &str) -> Result<bool> {
        match self.find_by_short_id(short_id).await? {
            Some((id, _)) => {
                self.delete_transaction(id).await?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// 批量删除交易记录
    pub async fn delete_by_short_ids(&self, short_ids: &[String]) -> Result<Vec<(String, bool)>> {
        let mut results = Vec::new();
        for id in short_ids {
            let success = self.delete_by_short_id(id).await?;
            results.push((id.clone(), success));
        }
        Ok(results)
    }

    /// 根据短 ID 更新交易记录
    pub async fn update_by_short_id(
        &self,
        short_id: &str,
        updates: TransactionUpdate,
    ) -> Result<Option<Transaction>> {
        // 查找记录
        let (id, _) = match self.find_by_short_id(short_id).await? {
            Some(result) => result,
            None => return Ok(None),
        };

        // 构建更新内容对象
        #[derive(Serialize)]
        struct UpdateData {
            #[serde(skip_serializing_if = "Option::is_none")]
            amount: Option<Decimal>,
            #[serde(skip_serializing_if = "Option::is_none")]
            currency: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            tx_type: Option<TxType>,
            #[serde(skip_serializing_if = "Option::is_none")]
            account_from_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            account_to_id: Option<Option<String>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            category_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<Option<String>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            tag_ids: Option<Vec<String>>,
            updated_at: Datetime,
        }

        let update_data = UpdateData {
            amount: updates.amount,
            currency: updates.currency,
            tx_type: updates.tx_type,
            account_from_id: updates.account_from_id,
            account_to_id: updates.account_to_id,
            category_id: updates.category_id,
            description: updates.description,
            tag_ids: updates.tag_ids,
            updated_at: Datetime::from(Utc::now()),
        };

        // 使用 MERGE 进行部分更新
        let sql = format!("UPDATE {} MERGE $data", id.to_string());
        
        let mut result = self
            .db
            .query(&sql)
            .bind(("data", update_data))
            .await
            .map_err(FinanceError::Database)?;

        let updated: Option<Transaction> = result.take(0).map_err(FinanceError::Database)?;
        Ok(updated)
    }

    // ==================== 账户管理方法 ====================

    /// 创建账户
    pub async fn create_account(&self, account: Account) -> Result<Account> {
        let id = account.id.clone();
        let sql = r#"CREATE type::thing("account", $id) CONTENT { name: $name, account_type: $account_type, parent_id: $parent_id, created_at: time::now() }"#;
        self.db
            .query(sql)
            .bind(("id", id.clone()))
            .bind(("name", account.name))
            .bind(("account_type", account.account_type))
            .bind(("parent_id", account.parent_id))
            .await
            .map_err(FinanceError::Database)?;
        // 重新查询获取创建的记录
        self.get_account(&id).await?.ok_or_else(|| FinanceError::Unknown("创建账户失败".to_string()))
    }

    /// 根据ID获取账户
    pub async fn get_account(&self, id: &str) -> Result<Option<Account>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, account_type, parent_id, created_at FROM account WHERE id = type::thing('account', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let account: Option<Account> = result.take(0).map_err(FinanceError::Database)?;
        Ok(account)
    }

    /// 根据名称查找账户
    pub async fn find_account_by_name(&self, name: &str) -> Result<Option<Account>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, account_type, parent_id, created_at FROM account WHERE name = $name";
        let mut result = self
            .db
            .query(sql)
            .bind(("name", name.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let account: Option<Account> = result.take(0).map_err(FinanceError::Database)?;
        Ok(account)
    }

    /// 列出所有账户
    pub async fn list_accounts(&self) -> Result<Vec<Account>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, account_type, parent_id, created_at FROM account ORDER BY name";
        let mut result = self.db.query(sql).await.map_err(FinanceError::Database)?;
        let accounts: Vec<Account> = result.take(0).map_err(FinanceError::Database)?;
        Ok(accounts)
    }

    /// 列出子账户
    pub async fn list_child_accounts(&self, parent_id: &str) -> Result<Vec<Account>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, account_type, parent_id, created_at FROM account WHERE parent_id = $parent_id ORDER BY name";
        let mut result = self
            .db
            .query(sql)
            .bind(("parent_id", parent_id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let accounts: Vec<Account> = result.take(0).map_err(FinanceError::Database)?;
        Ok(accounts)
    }

    /// 更新账户名称
    pub async fn update_account(&self, id: &str, name: &str) -> Result<Option<Account>> {
        let sql = "UPDATE account SET name = $name WHERE id = type::thing('account', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("name", name.to_string()))
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let updated: Option<Account> = result.take(0).map_err(FinanceError::Database)?;
        Ok(updated)
    }

    /// 删除账户
    pub async fn delete_account(&self, id: &str) -> Result<bool> {
        let sql = "DELETE FROM account WHERE id = type::thing('account', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let deleted: Option<Account> = result.take(0).map_err(FinanceError::Database)?;
        Ok(deleted.is_some())
    }

    // ==================== 分类管理方法 ====================

    /// 创建分类
    pub async fn create_category(&self, category: Category) -> Result<Category> {
        let id = category.id.clone();
        let sql = r#"CREATE type::thing("category", $id) CONTENT { name: $name, parent_id: $parent_id, full_path: $full_path, level: $level, created_at: time::now() }"#;
        self.db
            .query(sql)
            .bind(("id", id.clone()))
            .bind(("name", category.name))
            .bind(("parent_id", category.parent_id))
            .bind(("full_path", category.full_path))
            .bind(("level", category.level as i64))
            .await
            .map_err(FinanceError::Database)?;
        // 重新查询获取创建的记录
        self.get_category(&id).await?.ok_or_else(|| FinanceError::Unknown("创建分类失败".to_string()))
    }

    /// 根据ID获取分类
    pub async fn get_category(&self, id: &str) -> Result<Option<Category>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, parent_id, full_path, level, created_at FROM category WHERE id = type::thing('category', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let category: Option<Category> = result.take(0).map_err(FinanceError::Database)?;
        Ok(category)
    }

    /// 根据完整路径获取分类
    pub async fn get_category_by_path(&self, full_path: &str) -> Result<Option<Category>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, parent_id, full_path, level, created_at FROM category WHERE full_path = $path";
        let mut result = self
            .db
            .query(sql)
            .bind(("path", full_path.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let category: Option<Category> = result.take(0).map_err(FinanceError::Database)?;
        Ok(category)
    }

    /// 列出所有分类
    pub async fn list_categories(&self) -> Result<Vec<Category>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, parent_id, full_path, level, created_at FROM category ORDER BY full_path";
        let mut result = self.db.query(sql).await.map_err(FinanceError::Database)?;
        let categories: Vec<Category> = result.take(0).map_err(FinanceError::Database)?;
        Ok(categories)
    }

    /// 列出子分类
    pub async fn list_child_categories(&self, parent_id: &str) -> Result<Vec<Category>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, parent_id, full_path, level, created_at FROM category WHERE parent_id = $parent_id ORDER BY name";
        let mut result = self
            .db
            .query(sql)
            .bind(("parent_id", parent_id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let categories: Vec<Category> = result.take(0).map_err(FinanceError::Database)?;
        Ok(categories)
    }

    /// 更新分类名称（需要级联更新full_path）
    pub async fn update_category(&self, id: &str, name: &str) -> Result<Option<Category>> {
        // 先获取原分类
        let old_category = match self.get_category(id).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        // 构建新的full_path
        let new_full_path = if let Some(parent_path) = Category::parent_path(&old_category.full_path) {
            format!("{}/{}", parent_path, name)
        } else {
            name.to_string()
        };

        // 更新当前分类
        let sql = "UPDATE category SET name = $name, full_path = $path WHERE id = type::thing('category', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("name", name.to_string()))
            .bind(("path", new_full_path.clone()))
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let updated: Option<Category> = result.take(0).map_err(FinanceError::Database)?;

        // 级联更新所有子分类的full_path
        self.update_child_category_paths(&old_category.full_path, &new_full_path).await?;

        Ok(updated)
    }

    /// 级联更新子分类路径
    async fn update_child_category_paths(&self, old_prefix: &str, new_prefix: &str) -> Result<()> {
        let sql = r#"
            UPDATE category 
            SET full_path = string::replace(full_path, $old_prefix, $new_prefix)
            WHERE string::starts_with(full_path, $old_prefix_with_slash)
        "#;
        self.db
            .query(sql)
            .bind(("old_prefix", old_prefix.to_string()))
            .bind(("new_prefix", new_prefix.to_string()))
            .bind(("old_prefix_with_slash", format!("{}/", old_prefix)))
            .await
            .map_err(FinanceError::Database)?;
        Ok(())
    }

    /// 删除分类（递归删除子分类）
    pub async fn delete_category(&self, id: &str) -> Result<bool> {
        // 使用栈实现非递归删除
        let mut to_delete = vec![id.to_string()];
        let mut stack = vec![id.to_string()];

        // 收集所有需要删除的分类ID（从叶子到根）
        while let Some(current_id) = stack.pop() {
            let children = self.list_child_categories(&current_id).await?;
            for child in children {
                stack.push(child.id.clone());
                to_delete.push(child.id);
            }
        }

        // 从叶子到根删除
        for cat_id in to_delete.iter().rev() {
            let sql = "DELETE FROM category WHERE id = type::thing('category', $id)";
            self.db
                .query(sql)
                .bind(("id", cat_id.to_string()))
                .await
                .map_err(FinanceError::Database)?;
        }

        Ok(true)
    }

    // ==================== 标签管理方法 ====================

    /// 创建标签
    pub async fn create_tag(&self, tag: Tag) -> Result<Tag> {
        let id = tag.id.clone();
        let sql = r#"CREATE type::thing("tag", $id) CONTENT { name: $name, color: $color, created_at: time::now() }"#;
        self.db
            .query(sql)
            .bind(("id", id.clone()))
            .bind(("name", tag.name))
            .bind(("color", tag.color))
            .await
            .map_err(FinanceError::Database)?;
        // 重新查询获取创建的记录
        self.get_tag(&id).await?.ok_or_else(|| FinanceError::Unknown("创建标签失败".to_string()))
    }

    /// 根据ID获取标签
    pub async fn get_tag(&self, id: &str) -> Result<Option<Tag>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, color, created_at FROM tag WHERE id = type::thing('tag', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let tag: Option<Tag> = result.take(0).map_err(FinanceError::Database)?;
        Ok(tag)
    }

    /// 根据名称查找标签
    pub async fn find_tag_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, color, created_at FROM tag WHERE name = $name";
        let mut result = self
            .db
            .query(sql)
            .bind(("name", name.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let tag: Option<Tag> = result.take(0).map_err(FinanceError::Database)?;
        Ok(tag)
    }

    /// 列出所有标签
    pub async fn list_tags(&self) -> Result<Vec<Tag>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, color, created_at FROM tag ORDER BY name";
        let mut result = self.db.query(sql).await.map_err(FinanceError::Database)?;
        let tags: Vec<Tag> = result.take(0).map_err(FinanceError::Database)?;
        Ok(tags)
    }

    /// 更新标签
    pub async fn update_tag(&self, id: &str, name: &str) -> Result<Option<Tag>> {
        let sql = "UPDATE tag SET name = $name WHERE id = type::thing('tag', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("name", name.to_string()))
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let updated: Option<Tag> = result.take(0).map_err(FinanceError::Database)?;
        Ok(updated)
    }

    /// 删除标签
    pub async fn delete_tag(&self, id: &str) -> Result<bool> {
        // TODO: 从所有Transaction中移除该标签ID
        let sql = "DELETE FROM tag WHERE id = type::thing('tag', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let deleted: Option<Tag> = result.take(0).map_err(FinanceError::Database)?;
        Ok(deleted.is_some())
    }

    // ==================== 关联查询方法 ====================

    /// 批量获取账户（用于显示名称）
    pub async fn get_accounts_by_ids(&self, ids: &[String]) -> Result<Vec<Account>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let sql = "SELECT * FROM account WHERE id INSIDE $ids";
        let mut result = self
            .db
            .query(sql)
            .bind(("ids", ids.to_vec()))
            .await
            .map_err(FinanceError::Database)?;
        let accounts: Vec<Account> = result.take(0).map_err(FinanceError::Database)?;
        Ok(accounts)
    }

    /// 批量获取分类（用于显示名称）
    pub async fn get_categories_by_ids(&self, ids: &[String]) -> Result<Vec<Category>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let sql = "SELECT * FROM category WHERE id INSIDE $ids";
        let mut result = self
            .db
            .query(sql)
            .bind(("ids", ids.to_vec()))
            .await
            .map_err(FinanceError::Database)?;
        let categories: Vec<Category> = result.take(0).map_err(FinanceError::Database)?;
        Ok(categories)
    }

    /// 批量获取标签（用于显示名称）
    pub async fn get_tags_by_ids(&self, ids: &[String]) -> Result<Vec<Tag>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let sql = "SELECT * FROM tag WHERE id INSIDE $ids";
        let mut result = self
            .db
            .query(sql)
            .bind(("ids", ids.to_vec()))
            .await
            .map_err(FinanceError::Database)?;
        let tags: Vec<Tag> = result.take(0).map_err(FinanceError::Database)?;
        Ok(tags)
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
