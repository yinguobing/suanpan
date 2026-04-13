use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::engine::local::SurrealKv;
use surrealdb::Datetime;
use surrealdb::RecordId;
use surrealdb::Surreal;

use crate::error::{FinanceError, Result};
use crate::models::{Account, CategoryRecord, Tag, Transaction, TxType};

/// 生成带前缀的唯一ID
fn generate_id(prefix: &str) -> String {
    format!("{}_{}", prefix, nanoid::nanoid!(8))
}

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

    /// 组合查询交易记录
    #[allow(clippy::too_many_arguments)]
    pub async fn query_transactions(
        &self,
        from: Option<&str>,
        to: Option<&str>,
        category: Option<&str>,
        tx_type: Option<&str>,
        account: Option<&str>,
        search: Option<&str>,
        min_amount: Option<Decimal>,
        max_amount: Option<Decimal>,
        limit: Option<usize>,
    ) -> Result<Vec<Transaction>> {
        let mut conditions = vec![];
        let mut has_from = false;
        let mut has_to = false;
        let mut has_category = false;
        let mut has_tx_type = false;
        let mut has_account = false;
        let mut has_min_amount = false;
        let mut has_max_amount = false;

        // 时间范围
        let from_date = if let Some(from_str) = from {
            let dt = parse_date(from_str)?
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            conditions.push("timestamp >= $from".to_string());
            has_from = true;
            Some(dt)
        } else {
            None
        };

        let to_date = if let Some(to_str) = to {
            let dt = parse_date(to_str)?
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_utc();
            conditions.push("timestamp <= $to".to_string());
            has_to = true;
            Some(dt)
        } else {
            None
        };

        // 分类筛选
        let cat_id = if let Some(cat) = category {
            // 先尝试查找分类ID
            let cats = self.list_categories().await?;
            let id = cats
                .iter()
                .find(|c| c.id == cat || c.name == cat || c.full_path == cat)
                .map(|c| c.id.clone())
                .unwrap_or_else(|| cat.to_string());
            conditions.push("category_id = $category".to_string());
            has_category = true;
            Some(id)
        } else {
            None
        };

        // 类型筛选
        let tx_type_val = if let Some(tx_t) = tx_type {
            conditions.push("tx_type = $tx_type".to_string());
            has_tx_type = true;
            Some(tx_t.to_string())
        } else {
            None
        };

        // 账户筛选
        let acc_id = if let Some(acc) = account {
            // 先尝试查找账户ID
            let id = if let Some(account) = self.get_account(acc).await? {
                account.id
            } else if let Some(account) = self.find_account_by_name(acc).await? {
                account.id
            } else {
                acc.to_string()
            };
            conditions.push("(account_from_id = $account OR account_to_id = $account)".to_string());
            has_account = true;
            Some(id)
        } else {
            None
        };

        // 金额范围
        if min_amount.is_some() {
            conditions.push("amount >= $min_amount".to_string());
            has_min_amount = true;
        }

        if max_amount.is_some() {
            conditions.push("amount <= $max_amount".to_string());
            has_max_amount = true;
        }

        // 构建 SQL
        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();

        let sql = format!(
            "SELECT * FROM transaction {} ORDER BY timestamp DESC{}",
            where_clause, limit_clause
        );

        // 绑定参数
        let mut query = self.db.query(&sql);
        if has_from {
            query = query.bind(("from", Datetime::from(from_date.unwrap())));
        }
        if has_to {
            query = query.bind(("to", Datetime::from(to_date.unwrap())));
        }
        if has_category {
            query = query.bind(("category", cat_id.unwrap()));
        }
        if has_tx_type {
            query = query.bind(("tx_type", tx_type_val.unwrap()));
        }
        if has_account {
            query = query.bind(("account", acc_id.unwrap()));
        }
        if has_min_amount {
            query = query.bind(("min_amount", min_amount.unwrap()));
        }
        if has_max_amount {
            query = query.bind(("max_amount", max_amount.unwrap()));
        }

        let mut result = query.await.map_err(FinanceError::Database)?;
        let mut transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;

        // 模糊搜索（在内存中过滤，因为 SurrealDB 的字符串包含查询语法较复杂）
        if let Some(search_str) = search {
            let search_lower = search_str.to_lowercase();
            transactions.retain(|tx| {
                let desc_match = tx
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&search_lower))
                    .unwrap_or(false);
                // 也可以搜索分类名称（需要额外查询）
                desc_match
            });
        }

        Ok(transactions)
    }

    /// 获取月度统计
    pub async fn get_monthly_stats(&self, year: i32, month: u32) -> Result<MonthlyStats> {
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
        let mut category_breakdown: std::collections::HashMap<String, (String, Decimal)> =
            std::collections::HashMap::new();

        // 查询所有分类以获取名称映射
        let categories = self.list_categories().await?;
        let category_name_map: std::collections::HashMap<String, String> =
            categories.into_iter().map(|c| (c.id, c.name)).collect();

        for tx in &transactions {
            match tx.tx_type {
                crate::models::TxType::Income => total_income += tx.amount,
                crate::models::TxType::Expense => {
                    total_expense += tx.amount;
                    let category_name = category_name_map
                        .get(&tx.category_id)
                        .cloned()
                        .unwrap_or_else(|| tx.category_id.clone());
                    let entry = category_breakdown
                        .entry(tx.category_id.clone())
                        .or_insert_with(|| (category_name.clone(), Decimal::ZERO));
                    entry.1 += tx.amount;
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

    /// 获取自定义日期范围统计
    pub async fn get_stats_by_date_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<PeriodStats> {
        // 使用 SurrealDB 的 Datetime 类型
        let sql = "SELECT * FROM transaction WHERE timestamp >= $from AND timestamp < $to";
        let mut result = self
            .db
            .query(sql)
            .bind(("from", Datetime::from(from)))
            .bind(("to", Datetime::from(to)))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;

        let mut total_income = Decimal::ZERO;
        let mut total_expense = Decimal::ZERO;
        let mut category_breakdown: std::collections::HashMap<String, (String, Decimal)> =
            std::collections::HashMap::new();

        // 查询所有分类以获取名称映射
        let categories = self.list_categories().await?;
        let category_name_map: std::collections::HashMap<String, String> =
            categories.into_iter().map(|c| (c.id, c.name)).collect();

        for tx in &transactions {
            match tx.tx_type {
                crate::models::TxType::Income => total_income += tx.amount,
                crate::models::TxType::Expense => {
                    total_expense += tx.amount;
                    let category_name = category_name_map
                        .get(&tx.category_id)
                        .cloned()
                        .unwrap_or_else(|| tx.category_id.clone());
                    let entry = category_breakdown
                        .entry(tx.category_id.clone())
                        .or_insert_with(|| (category_name.clone(), Decimal::ZERO));
                    entry.1 += tx.amount;
                }
                _ => {}
            }
        }

        Ok(PeriodStats {
            from,
            to,
            total_income,
            total_expense,
            net: total_income - total_expense,
            transaction_count: transactions.len(),
            category_breakdown,
        })
    }

    /// 获取按账户统计
    pub async fn get_stats_by_account(
        &self,
        account_id: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<AccountStats>> {
        use std::collections::HashMap;

        // 构建查询条件
        let mut conditions = vec![];
        if from.is_some() {
            conditions.push("timestamp >= $from".to_string());
        }
        if to.is_some() {
            conditions.push("timestamp < $to".to_string());
        }
        if account_id.is_some() {
            conditions
                .push("(account_from_id = $account_id OR account_to_id = $account_id)".to_string());
        }

        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!("SELECT * FROM transaction {}", where_clause);

        let mut query = self.db.query(&sql);
        if let Some(f) = from {
            query = query.bind(("from", Datetime::from(f)));
        }
        if let Some(t) = to {
            query = query.bind(("to", Datetime::from(t)));
        }
        if let Some(acc) = account_id {
            query = query.bind(("account_id", acc.to_string()));
        }

        let mut result = query.await.map_err(FinanceError::Database)?;
        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;

        // 获取所有账户信息
        let accounts = self.list_accounts().await?;
        let account_map: HashMap<String, String> =
            accounts.into_iter().map(|a| (a.id, a.name)).collect();

        // 统计每个账户的数据
        let mut stats_map: HashMap<String, (String, Decimal, Decimal, usize)> = HashMap::new();

        for tx in &transactions {
            // 处理来源账户
            if let Some(from_name) = account_map.get(&tx.account_from_id) {
                let entry = stats_map
                    .entry(tx.account_from_id.clone())
                    .or_insert_with(|| (from_name.clone(), Decimal::ZERO, Decimal::ZERO, 0));

                match tx.tx_type {
                    crate::models::TxType::Expense | crate::models::TxType::Transfer => {
                        entry.2 += tx.amount; // 支出/转出
                    }
                    _ => {}
                }
                entry.3 += 1;
            }

            // 处理目标账户
            if let Some(to_id) = &tx.account_to_id {
                if let Some(to_name) = account_map.get(to_id) {
                    let entry = stats_map
                        .entry(to_id.clone())
                        .or_insert_with(|| (to_name.clone(), Decimal::ZERO, Decimal::ZERO, 0));

                    match tx.tx_type {
                        crate::models::TxType::Income | crate::models::TxType::Transfer => {
                            entry.1 += tx.amount; // 收入/转入
                        }
                        _ => {}
                    }
                    entry.3 += 1;
                }
            }
        }

        // 转换为 AccountStats 列表
        let mut account_stats: Vec<AccountStats> = stats_map
            .into_iter()
            .map(|(id, (name, income, expense, count))| AccountStats {
                account_id: id,
                account_name: name,
                total_income: income,
                total_expense: expense,
                net_flow: income - expense,
                transaction_count: count,
            })
            .collect();

        // 按净流入排序
        account_stats.sort_by(|a, b| b.net_flow.cmp(&a.net_flow));

        Ok(account_stats)
    }

    /// 获取层级分类统计
    ///
    /// 返回按层级组织的分类统计，子分类金额会自动汇总到父分类
    pub async fn get_hierarchical_category_stats(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<HierarchicalCategoryStats>> {
        use std::collections::HashMap;

        // 1. 获取所有分类
        let categories = self.list_categories().await?;

        // 构建分类ID到分类信息的映射
        let mut category_map: HashMap<String, (String, Option<String>, u32)> = HashMap::new();
        // 构建父分类到子分类列表的映射
        let mut children_map: HashMap<String, Vec<String>> = HashMap::new();

        for cat in &categories {
            category_map.insert(
                cat.id.clone(),
                (cat.name.clone(), cat.parent_id.clone(), cat.level),
            );

            if let Some(ref parent_id) = cat.parent_id {
                children_map
                    .entry(parent_id.clone())
                    .or_default()
                    .push(cat.id.clone());
            }
        }

        // 2. 获取指定时间范围内的交易
        let mut conditions = vec![];
        if from.is_some() {
            conditions.push("timestamp >= $from".to_string());
        }
        if to.is_some() {
            conditions.push("timestamp < $to".to_string());
        }

        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };
        let sql = format!("SELECT * FROM transaction {}", where_clause);

        let mut query = self.db.query(&sql);
        if let Some(f) = from {
            query = query.bind(("from", Datetime::from(f)));
        }
        if let Some(t) = to {
            query = query.bind(("to", Datetime::from(t)));
        }

        let mut result = query.await.map_err(FinanceError::Database)?;
        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;

        // 3. 按分类ID汇总支出金额（只统计 Expense 类型）
        let mut direct_amounts: HashMap<String, Decimal> = HashMap::new();
        for tx in &transactions {
            if matches!(tx.tx_type, crate::models::TxType::Expense) {
                *direct_amounts
                    .entry(tx.category_id.clone())
                    .or_insert(Decimal::ZERO) += tx.amount;
            }
        }

        // 4. 计算总支出（用于计算百分比）
        let total_expense: Decimal = direct_amounts.values().sum();
        let total_expense_abs = total_expense.abs();

        // 5. 构建层级统计（递归函数）
        fn build_hierarchical_stats(
            category_id: &str,
            category_map: &HashMap<String, (String, Option<String>, u32)>,
            children_map: &HashMap<String, Vec<String>>,
            direct_amounts: &HashMap<String, Decimal>,
            total_expense_abs: Decimal,
        ) -> Option<HierarchicalCategoryStats> {
            let (name, _parent_id, level) = category_map.get(category_id)?;

            // 获取直接金额（包含负数退款）
            let direct_amount = *direct_amounts.get(category_id).unwrap_or(&Decimal::ZERO);

            // 递归构建子分类统计
            let mut children = Vec::new();
            let mut total_amount = direct_amount;

            if let Some(child_ids) = children_map.get(category_id) {
                for child_id in child_ids {
                    if let Some(child_stats) = build_hierarchical_stats(
                        child_id,
                        category_map,
                        children_map,
                        direct_amounts,
                        total_expense_abs,
                    ) {
                        total_amount += child_stats.total_amount;
                        children.push(child_stats);
                    }
                }
            }

            // 按金额绝对值降序排序子分类
            children.sort_by(|a, b| b.total_amount.abs().cmp(&a.total_amount.abs()));

            // 计算完整路径
            let full_path = build_full_path(category_id, category_map);

            // 百分比基于绝对值计算
            let percentage = if total_expense_abs > Decimal::ZERO {
                (total_amount / total_expense_abs * Decimal::from(100)).round_dp(1)
            } else {
                Decimal::ZERO
            };

            Some(HierarchicalCategoryStats {
                category_id: category_id.to_string(),
                category_name: name.clone(),
                full_path,
                level: *level,
                direct_amount,
                total_amount,
                percentage,
                children,
            })
        }

        // 辅助函数：构建完整路径
        fn build_full_path(
            category_id: &str,
            category_map: &HashMap<String, (String, Option<String>, u32)>,
        ) -> String {
            let mut parts = Vec::new();
            let mut current_id = Some(category_id.to_string());

            // 收集从当前节点到根节点的路径（反向）
            while let Some(id) = current_id {
                if let Some((name, parent_id, _)) = category_map.get(&id) {
                    parts.push(name.clone());
                    current_id = parent_id.clone();
                } else {
                    break;
                }
            }

            // 反转得到从根到当前节点的路径
            parts.reverse();
            parts.join("/")
        }

        // 6. 从根分类（没有父分类的）开始构建层级统计
        let mut root_stats = Vec::new();
        for cat in &categories {
            if cat.parent_id.is_none() {
                if let Some(stats) = build_hierarchical_stats(
                    &cat.id,
                    &category_map,
                    &children_map,
                    &direct_amounts,
                    total_expense_abs,
                ) {
                    // 只包含有金额或子分类有金额的根分类
                    if stats.total_amount != Decimal::ZERO || !stats.children.is_empty() {
                        root_stats.push(stats);
                    }
                }
            }
        }

        // 7. 按总金额绝对值降序排序根分类
        root_stats.sort_by(|a, b| b.total_amount.abs().cmp(&a.total_amount.abs()));

        Ok(root_stats)
    }

    /// 获取趋势统计
    pub async fn get_trend_stats(
        &self,
        period: TrendPeriod,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<TrendStats>> {
        use std::collections::HashMap;

        // 查询时间范围内的交易
        let sql = "SELECT * FROM transaction WHERE timestamp >= $from AND timestamp <= $to ORDER BY timestamp ASC";
        let mut result = self
            .db
            .query(sql)
            .bind(("from", Datetime::from(from)))
            .bind(("to", Datetime::from(to)))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;

        // 按周期聚合
        let mut period_data: HashMap<String, (Decimal, Decimal, usize)> = HashMap::new();

        for tx in &transactions {
            // surrealdb::Datetime -> surrealdb::sql::Datetime -> chrono::DateTime<Utc>
            let sql_dt: surrealdb::sql::Datetime = tx.timestamp.to_owned().into_inner();
            let chrono_ts: DateTime<Utc> = sql_dt.into();
            let period_key = get_period_key(&chrono_ts, &period);
            let entry =
                period_data
                    .entry(period_key)
                    .or_insert((Decimal::ZERO, Decimal::ZERO, 0usize));

            match tx.tx_type {
                TxType::Income => entry.0 += tx.amount,
                TxType::Expense => entry.1 += tx.amount,
                _ => {}
            }
            entry.2 += 1;
        }

        // 生成完整的时间周期序列（包括没有数据的周期）
        let all_periods = generate_periods(from, to, &period);

        // 合并数据
        let mut stats: Vec<TrendStats> = all_periods
            .into_iter()
            .map(|label| {
                let (income, expense, count) =
                    period_data
                        .get(&label)
                        .copied()
                        .unwrap_or((Decimal::ZERO, Decimal::ZERO, 0));
                TrendStats {
                    period_label: label,
                    income,
                    expense,
                    transaction_count: count,
                }
            })
            .collect();

        // 按时间顺序排序
        stats.sort_by(|a, b| a.period_label.cmp(&b.period_label));

        Ok(stats)
    }

    /// 获取分类趋势统计
    pub async fn get_category_trend_stats(
        &self,
        period: TrendPeriod,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<(String, Vec<(String, Decimal)>)>> {
        use std::collections::HashMap;

        // 获取所有分类
        let categories = self.list_categories().await?;
        let category_map: HashMap<String, String> = categories
            .into_iter()
            .map(|c| (c.id, c.full_path))
            .collect();

        // 查询时间范围内的支出交易
        let sql = "SELECT * FROM transaction WHERE timestamp >= $from AND timestamp <= $to AND tx_type = 'expense' ORDER BY timestamp ASC";
        let mut result = self
            .db
            .query(sql)
            .bind(("from", Datetime::from(from)))
            .bind(("to", Datetime::from(to)))
            .await
            .map_err(FinanceError::Database)?;

        let transactions: Vec<Transaction> = result.take(0).map_err(FinanceError::Database)?;

        // 按分类和周期聚合
        // category_id -> (period_label -> amount)
        let mut category_data: HashMap<String, HashMap<String, Decimal>> = HashMap::new();
        // 所有周期列表
        let mut all_periods_set: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();

        for tx in &transactions {
            // surrealdb::Datetime -> surrealdb::sql::Datetime -> chrono::DateTime<Utc>
            let sql_dt: surrealdb::sql::Datetime = tx.timestamp.to_owned().into_inner();
            let chrono_ts: DateTime<Utc> = sql_dt.into();
            let period_key = get_period_key(&chrono_ts, &period);
            all_periods_set.insert(period_key.clone());

            let category_path = category_map
                .get(&tx.category_id)
                .cloned()
                .unwrap_or_else(|| tx.category_id.clone());

            let cat_entry = category_data.entry(category_path).or_default();
            *cat_entry.entry(period_key).or_insert(Decimal::ZERO) += tx.amount;
        }

        // 转换为返回格式
        let all_periods: Vec<String> = all_periods_set.into_iter().collect();

        let mut result: Vec<(String, Vec<(String, Decimal)>)> = category_data
            .into_iter()
            .map(|(category_id, period_map)| {
                // 为每个周期填充数据
                let data: Vec<(String, Decimal)> = all_periods
                    .iter()
                    .map(|p| {
                        let amount = period_map.get(p).copied().unwrap_or(Decimal::ZERO);
                        (p.clone(), amount)
                    })
                    .collect();
                (category_id, data)
            })
            .collect();

        // 按分类名称排序
        result.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(result)
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
            return Err(FinanceError::Validation("短 ID 应为 12 位字符".to_string()));
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
            let id = tx
                .id
                .clone()
                .ok_or_else(|| FinanceError::Unknown("交易记录缺少 ID".to_string()))?;
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
        let sql = format!("UPDATE {} MERGE $data", id);

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
        self.get_account(&id)
            .await?
            .ok_or_else(|| FinanceError::Unknown("创建账户失败".to_string()))
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

    /// 根据名称查找账户，如果不存在则创建（默认类型为 Other）
    pub async fn find_or_create_account_by_name(&self, name: &str) -> Result<Account> {
        use crate::models::{Account, AccountType};

        // 先查找
        if let Some(account) = self.find_account_by_name(name).await? {
            return Ok(account);
        }

        // 不存在则创建（默认类型为 Other）
        let id = generate_id("acc");
        let account = Account::new(id, name, AccountType::Other);
        self.create_account(account).await
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
    pub async fn create_category(&self, category: CategoryRecord) -> Result<CategoryRecord> {
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
        self.get_category(&id)
            .await?
            .ok_or_else(|| FinanceError::Unknown("创建分类失败".to_string()))
    }

    /// 根据ID获取分类
    pub async fn get_category(&self, id: &str) -> Result<Option<CategoryRecord>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, parent_id, full_path, level, created_at FROM category WHERE id = type::thing('category', $id)";
        let mut result = self
            .db
            .query(sql)
            .bind(("id", id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let category: Option<CategoryRecord> = result.take(0).map_err(FinanceError::Database)?;
        Ok(category)
    }

    /// 根据完整路径获取分类
    pub async fn get_category_by_path(&self, full_path: &str) -> Result<Option<CategoryRecord>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, parent_id, full_path, level, created_at FROM category WHERE full_path = $path";
        let mut result = self
            .db
            .query(sql)
            .bind(("path", full_path.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let category: Option<CategoryRecord> = result.take(0).map_err(FinanceError::Database)?;
        Ok(category)
    }

    /// 根据完整路径查找分类，如果不存在则创建（自动创建父分类）
    pub async fn find_or_create_category_by_path(&self, full_path: &str) -> Result<CategoryRecord> {
        self.find_or_create_category_by_path_impl(full_path).await
    }

    /// 内部实现（boxed to avoid recursion in async）
    fn find_or_create_category_by_path_impl<'a>(
        &'a self,
        full_path: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<CategoryRecord>> + Send + 'a>>
    {
        Box::pin(async move {
            // 先查找
            if let Some(category) = self.get_category_by_path(full_path).await? {
                return Ok(category);
            }

            // 解析路径
            let parts: Vec<&str> = full_path.split('/').collect();
            let name = parts.last().unwrap_or(&"").to_string();

            // 查找或创建父分类
            let parent_id = if parts.len() > 1 {
                let parent_path = parts[..parts.len() - 1].join("/");
                let parent = self
                    .find_or_create_category_by_path_impl(&parent_path)
                    .await?;
                Some(parent.id)
            } else {
                None
            };

            // 创建当前分类
            use crate::models::CategoryRecord;
            use surrealdb::Datetime;

            let level = parts.len() as u32;
            let id = generate_id("cat");
            let category = CategoryRecord {
                id,
                name,
                parent_id,
                full_path: full_path.to_string(),
                level,
                created_at: Datetime::from(chrono::Utc::now()),
            };
            self.create_category(category).await
        })
    }

    /// 列出所有分类
    pub async fn list_categories(&self) -> Result<Vec<CategoryRecord>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, parent_id, full_path, level, created_at FROM category ORDER BY full_path";
        let mut result = self.db.query(sql).await.map_err(FinanceError::Database)?;
        let categories: Vec<CategoryRecord> = result.take(0).map_err(FinanceError::Database)?;
        Ok(categories)
    }

    /// 列出子分类
    pub async fn list_child_categories(&self, parent_id: &str) -> Result<Vec<CategoryRecord>> {
        let sql = "SELECT string::split(<string> id, ':')[1] as id, name, parent_id, full_path, level, created_at FROM category WHERE parent_id = $parent_id ORDER BY name";
        let mut result = self
            .db
            .query(sql)
            .bind(("parent_id", parent_id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        let categories: Vec<CategoryRecord> = result.take(0).map_err(FinanceError::Database)?;
        Ok(categories)
    }

    /// 更新分类名称（需要级联更新full_path）
    pub async fn update_category(&self, id: &str, name: &str) -> Result<Option<CategoryRecord>> {
        // 先获取原分类
        let old_category = match self.get_category(id).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        // 构建新的full_path
        let new_full_path = if let Some(parent_path) =
            crate::models::category_utils::parent_path(&old_category.full_path)
        {
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
        let updated: Option<CategoryRecord> = result.take(0).map_err(FinanceError::Database)?;

        // 级联更新所有子分类的full_path
        self.update_child_category_paths(&old_category.full_path, &new_full_path)
            .await?;

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

    /// 统计引用某分类的交易记录数量
    pub async fn count_transactions_by_category(&self, category_id: &str) -> Result<usize> {
        let sql = "SELECT count() FROM transaction WHERE category_id = $category_id GROUP BY count";
        let mut result = self
            .db
            .query(sql)
            .bind(("category_id", category_id.to_string()))
            .await
            .map_err(FinanceError::Database)?;

        #[derive(Debug, Deserialize)]
        struct CountResult {
            count: usize,
        }

        let counts: Vec<CountResult> = result.take(0).map_err(FinanceError::Database)?;
        Ok(counts.first().map(|c| c.count).unwrap_or(0))
    }

    /// 统计引用某账户的交易记录数量
    pub async fn count_transactions_by_account(&self, account_id: &str) -> Result<usize> {
        let sql = "SELECT count() FROM transaction WHERE account_from_id = $account_id OR account_to_id = $account_id GROUP BY count";
        let mut result = self
            .db
            .query(sql)
            .bind(("account_id", account_id.to_string()))
            .await
            .map_err(FinanceError::Database)?;

        #[derive(Debug, Deserialize)]
        struct CountResult {
            count: usize,
        }

        let counts: Vec<CountResult> = result.take(0).map_err(FinanceError::Database)?;
        Ok(counts.first().map(|c| c.count).unwrap_or(0))
    }

    /// 统计引用某标签的交易记录数量
    pub async fn count_transactions_by_tag(&self, tag_id: &str) -> Result<usize> {
        let sql = "SELECT count() FROM transaction WHERE tag_ids CONTAINS $tag_id GROUP BY count";
        let mut result = self
            .db
            .query(sql)
            .bind(("tag_id", tag_id.to_string()))
            .await
            .map_err(FinanceError::Database)?;

        #[derive(Debug, Deserialize)]
        struct CountResult {
            count: usize,
        }

        let counts: Vec<CountResult> = result.take(0).map_err(FinanceError::Database)?;
        Ok(counts.first().map(|c| c.count).unwrap_or(0))
    }

    /// 从所有交易记录中移除指定标签
    pub async fn remove_tag_from_transactions(&self, tag_id: &str) -> Result<usize> {
        let sql = "UPDATE transaction SET tag_ids = array::difference(tag_ids, [$tag_id]) WHERE tag_ids CONTAINS $tag_id";
        let mut result = self
            .db
            .query(sql)
            .bind(("tag_id", tag_id.to_string()))
            .await
            .map_err(FinanceError::Database)?;
        // SurrealDB UPDATE 返回受影响的记录数
        let affected: Vec<serde_json::Value> = result.take(0).map_err(FinanceError::Database)?;
        Ok(affected.len())
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
        self.get_tag(&id)
            .await?
            .ok_or_else(|| FinanceError::Unknown("创建标签失败".to_string()))
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

    /// 根据名称查找标签，如果不存在则创建
    pub async fn find_or_create_tag_by_name(&self, name: &str) -> Result<Tag> {
        // 先查找
        if let Some(tag) = self.find_tag_by_name(name).await? {
            return Ok(tag);
        }

        // 不存在则创建
        let id = generate_id("tag");
        let sql = "CREATE type::thing('tag', $id) CONTENT { name: $name, created_at: time::now() }";
        let mut result = self
            .db
            .query(sql)
            .bind(("id", id.clone()))
            .bind(("name", name.to_string()))
            .await
            .map_err(FinanceError::Database)?;

        let tag: Option<Tag> = result.take(0).map_err(FinanceError::Database)?;
        tag.ok_or_else(|| FinanceError::Unknown("创建标签失败".to_string()))
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

    /// 删除标签（会自动从所有交易记录中移除关联）
    pub async fn delete_tag(&self, id: &str) -> Result<bool> {
        // 先从所有交易记录中移除该标签ID
        let _removed = self.remove_tag_from_transactions(id).await?;

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
    pub async fn get_categories_by_ids(&self, ids: &[String]) -> Result<Vec<CategoryRecord>> {
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
        let categories: Vec<CategoryRecord> = result.take(0).map_err(FinanceError::Database)?;
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

    /// 数据迁移：将旧格式数据迁移到新模型
    pub async fn migrate_data(&self, dry_run: bool) -> Result<MigrationStats> {
        use std::collections::HashMap;
        use std::collections::HashSet;

        let mut stats = MigrationStats {
            transactions_migrated: 0,
            accounts_created: 0,
            categories_created: 0,
            tags_created: 0,
        };

        // 1. 获取使用旧字段的交易记录
        let old_transactions = self.get_old_transactions().await?;

        if old_transactions.is_empty() {
            return Ok(stats);
        }

        // 2. 收集需要创建的实体
        let mut account_names: HashSet<String> = HashSet::new();
        let mut category_names: HashSet<String> = HashSet::new();
        let mut tag_names: HashSet<String> = HashSet::new();

        for tx in &old_transactions {
            if !tx.account_from.is_empty() {
                account_names.insert(tx.account_from.clone());
            }
            if let Some(ref to) = tx.account_to {
                if !to.is_empty() {
                    account_names.insert(to.clone());
                }
            }
            if !tx.category.is_empty() {
                category_names.insert(tx.category.clone());
            }
            for tag in &tx.tags {
                if !tag.is_empty() {
                    tag_names.insert(tag.clone());
                }
            }
        }

        // 3. 创建账户
        let mut account_map: HashMap<String, String> = HashMap::new();
        for name in account_names {
            if let Some(existing) = self.find_account_by_name(&name).await? {
                account_map.insert(name, existing.id);
            } else if !dry_run {
                let id = format!("acc_{}", nanoid::nanoid!(8));
                let account =
                    crate::models::Account::new(&id, &name, crate::models::AccountType::Other);
                self.create_account(account).await?;
                account_map.insert(name, id);
                stats.accounts_created += 1;
            }
        }

        // 4. 创建分类
        let mut category_map: HashMap<String, String> = HashMap::new();
        for name in category_names {
            if let Some(existing) = self.get_category_by_path(&name).await? {
                category_map.insert(name, existing.id);
            } else if !dry_run {
                let id = format!("cat_{}", nanoid::nanoid!(8));
                let category = CategoryRecord {
                    id: id.clone(),
                    name: name.clone(),
                    parent_id: None,
                    full_path: name.clone(),
                    level: 0,
                    created_at: Datetime::from(chrono::Utc::now()),
                };
                self.create_category(category).await?;
                category_map.insert(name, id);
                stats.categories_created += 1;
            }
        }

        // 5. 创建标签
        let mut tag_map: HashMap<String, String> = HashMap::new();
        for name in tag_names {
            if let Some(existing) = self.find_tag_by_name(&name).await? {
                tag_map.insert(name, existing.id);
            } else if !dry_run {
                let id = format!("tag_{}", nanoid::nanoid!(8));
                let tag = crate::models::Tag::new(&id, &name);
                self.create_tag(tag).await?;
                tag_map.insert(name, id);
                stats.tags_created += 1;
            }
        }

        // 6. 更新交易记录
        if !dry_run {
            for old_tx in old_transactions {
                let account_from_id = account_map
                    .get(&old_tx.account_from)
                    .cloned()
                    .unwrap_or_default();

                let account_to_id = old_tx
                    .account_to
                    .as_ref()
                    .and_then(|n| account_map.get(n).cloned());

                let category_id = category_map
                    .get(&old_tx.category)
                    .cloned()
                    .unwrap_or_default();

                let tag_ids: Vec<String> = old_tx
                    .tags
                    .iter()
                    .filter_map(|t| tag_map.get(t).cloned())
                    .collect();

                // 更新交易记录
                let sql = "UPDATE transaction SET account_from_id = $account_from_id, account_to_id = $account_to_id, category_id = $category_id, tag_ids = $tag_ids WHERE id = $id";
                self.db
                    .query(sql)
                    .bind(("account_from_id", account_from_id))
                    .bind(("account_to_id", account_to_id))
                    .bind(("category_id", category_id))
                    .bind(("tag_ids", tag_ids))
                    .bind(("id", old_tx.id))
                    .await
                    .map_err(FinanceError::Database)?;

                stats.transactions_migrated += 1;
            }
        } else {
            stats.transactions_migrated = old_transactions.len();
        }

        Ok(stats)
    }

    /// 获取使用旧字段的交易记录
    async fn get_old_transactions(&self) -> Result<Vec<OldTransaction>> {
        // 查找同时有旧字段account_from和新字段account_from_id为空的记录
        let sql = r#"
            SELECT id, account_from, account_to, category, tags 
            FROM transaction 
            WHERE account_from IS NOT NULL AND account_from_id IS NONE
        "#;

        let mut result = self.db.query(sql).await.map_err(FinanceError::Database)?;
        let transactions: Vec<OldTransaction> = result.take(0).map_err(FinanceError::Database)?;
        Ok(transactions)
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
    /// (分类名称, 金额) 的列表，保留 ID 作为 key 用于 --show-ids 参数
    pub category_breakdown: std::collections::HashMap<String, (String, Decimal)>,
}

/// 自定义日期范围统计
#[derive(Debug)]
pub struct PeriodStats {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub net: Decimal,
    pub transaction_count: usize,
    /// (分类名称, 金额) 的列表，保留 ID 作为 key 用于 --show-ids 参数
    pub category_breakdown: std::collections::HashMap<String, (String, Decimal)>,
}

/// 账户统计
#[derive(Debug)]
pub struct AccountStats {
    pub account_id: String,
    pub account_name: String,
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub net_flow: Decimal,
    pub transaction_count: usize,
}

/// 数据迁移统计
#[derive(Debug)]
pub struct MigrationStats {
    pub transactions_migrated: usize,
    pub accounts_created: usize,
    pub categories_created: usize,
    pub tags_created: usize,
}

/// 层级分类统计项
#[derive(Debug)]
pub struct HierarchicalCategoryStats {
    pub category_id: String,
    pub category_name: String,
    pub full_path: String,
    pub level: u32,
    pub direct_amount: Decimal, // 直接属于该分类的金额（不含子分类）
    pub total_amount: Decimal,  // 汇总金额（含所有子分类）
    pub percentage: Decimal,    // 占总支出的百分比
    pub children: Vec<HierarchicalCategoryStats>,
}

/// 旧格式交易记录（用于迁移）
#[derive(Debug, serde::Deserialize)]
struct OldTransaction {
    pub id: surrealdb::RecordId,
    pub account_from: String,
    pub account_to: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
}

/// 趋势周期类型
#[derive(Debug, Clone, Copy)]
pub enum TrendPeriod {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

/// 趋势统计项
#[derive(Debug)]
pub struct TrendStats {
    pub period_label: String, // 周期标签，如 "2026-01" 或 "2026-W01"
    pub income: Decimal,
    pub expense: Decimal,
    pub transaction_count: usize,
}

/// 根据时间戳获取周期键
fn get_period_key(timestamp: &DateTime<Utc>, period: &TrendPeriod) -> String {
    match period {
        TrendPeriod::Day => timestamp.format("%Y-%m-%d").to_string(),
        TrendPeriod::Week => {
            let iso_week = timestamp.iso_week();
            format!("{}-W{:02}", iso_week.year(), iso_week.week())
        }
        TrendPeriod::Month => timestamp.format("%Y-%m").to_string(),
        TrendPeriod::Quarter => {
            let quarter = (timestamp.month() - 1) / 3 + 1;
            format!("{}-Q{}", timestamp.year(), quarter)
        }
        TrendPeriod::Year => timestamp.year().to_string(),
    }
}

/// 生成完整的周期序列
fn generate_periods(from: DateTime<Utc>, to: DateTime<Utc>, period: &TrendPeriod) -> Vec<String> {
    let mut periods = Vec::new();
    let mut current = from.date_naive();
    let end = to.date_naive();

    while current <= end {
        let key = match period {
            TrendPeriod::Day => current.format("%Y-%m-%d").to_string(),
            TrendPeriod::Week => {
                let dt = current.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let iso_week = dt.iso_week();
                format!("{}-W{:02}", iso_week.year(), iso_week.week())
            }
            TrendPeriod::Month => current.format("%Y-%m").to_string(),
            TrendPeriod::Quarter => {
                let quarter = (current.month() - 1) / 3 + 1;
                format!("{}-Q{}", current.year(), quarter)
            }
            TrendPeriod::Year => current.year().to_string(),
        };

        // 避免重复添加相同的周期键（如周可能跨越多天）
        if !periods.contains(&key) {
            periods.push(key);
        }

        // 移动到下一个周期
        current = match period {
            TrendPeriod::Day => current.succ_opt().unwrap_or(current),
            TrendPeriod::Week => current
                .checked_add_days(chrono::Days::new(7))
                .unwrap_or(current),
            TrendPeriod::Month => current
                .checked_add_months(chrono::Months::new(1))
                .unwrap_or(current),
            TrendPeriod::Quarter => current
                .checked_add_months(chrono::Months::new(3))
                .unwrap_or(current),
            TrendPeriod::Year => current.with_year(current.year() + 1).unwrap_or(current),
        };
    }

    periods
}

/// 解析日期字符串（YYYY-MM-DD）
fn parse_date(date_str: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| FinanceError::Parse(format!("日期格式错误：'{}'，应为 YYYY-MM-DD", date_str)))
}
