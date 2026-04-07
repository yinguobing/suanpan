use chrono::Local;
use clap::Args;
use comfy_table::Table;

use crate::db::surreal::Database;
use crate::error::Result;

/// 将 SurrealDB Datetime 格式化为本地时间字符串（完整格式）
fn format_datetime(dt: &surrealdb::Datetime) -> String {
    let sql_dt: surrealdb::sql::Datetime = dt.to_owned().into_inner();
    let utc_dt: chrono::DateTime<chrono::Utc> = sql_dt.into();
    let local_dt: chrono::DateTime<Local> = utc_dt.into();
    local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 从 RecordId 提取短 ID（前12位）
fn format_short_id(id: &Option<surrealdb::RecordId>) -> String {
    match id {
        Some(rid) => {
            let full_id = rid.to_string();
            // RecordId 格式: transaction:xxxxxxxxxxxxx
            // 提取冒号后的前12位
            full_id
                .split(':')
                .nth(1)
                .map(|s| s.chars().take(12).collect())
                .unwrap_or_else(|| "unknown".to_string())
        }
        None => "unknown".to_string(),
    }
}

/// 列出交易记录
#[derive(Args)]
pub struct ListArgs {
    /// 显示条数
    #[arg(short, long, default_value = "20")]
    pub limit: usize,

    /// 起始日期（格式：YYYY-MM-DD）
    #[arg(long)]
    pub from: Option<String>,

    /// 结束日期（格式：YYYY-MM-DD）
    #[arg(long)]
    pub to: Option<String>,

    /// 按分类筛选
    #[arg(short, long)]
    pub category: Option<String>,

    /// 按类型筛选
    #[arg(short, long)]
    pub tx_type: Option<String>,
}

pub async fn execute(db: &Database, args: ListArgs) -> Result<()> {
    let transactions = if let Some(category) = args.category {
        db.query_by_category(&category).await?
    } else if let Some(tx_type) = args.tx_type {
        db.query_by_type(&tx_type).await?
    } else if let (Some(from), Some(to)) = (args.from, args.to) {
        let from_date = parse_date(&from)?.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let to_date = parse_date(&to)?.and_hms_opt(23, 59, 59).unwrap().and_utc();
        db.query_by_date_range(from_date, to_date).await?
    } else {
        db.list_transactions(args.limit).await?
    };

    if transactions.is_empty() {
        println!("暂无交易记录");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec![
        "时间", "类型", "金额", "货币", "账户", "去向", "分类", "描述", "ID",
    ]);

    for tx in transactions.iter().take(args.limit) {
        let time = format_datetime(&tx.timestamp);
        let tx_type = format!("{}", tx.tx_type);
        let amount = tx.amount.to_string();
        let currency = &tx.currency;
        // TODO: 批次2将添加ID到名称的查询
        let account_from = &tx.account_from_id;
        let account_to = tx.account_to_id.as_deref().unwrap_or("-");
        let category = &tx.category_id;
        let description = tx.description.as_deref().unwrap_or("-");
        let short_id = format_short_id(&tx.id);

        table.add_row(vec![
            &time,
            &tx_type,
            &amount,
            currency,
            account_from,
            account_to,
            category,
            description,
            &short_id,
        ]);
    }

    println!("{}", table);
    println!("共 {} 条记录", transactions.len().min(args.limit));

    Ok(())
}

fn parse_date(date_str: &str) -> Result<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| crate::error::FinanceError::Parse(format!("日期格式错误: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_date_valid() {
        let date = parse_date("2025-04-06").unwrap();
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 4);
        assert_eq!(date.day(), 6);
    }

    #[test]
    fn test_parse_date_invalid_format() {
        assert!(parse_date("2025/04/06").is_err());
        assert!(parse_date("06-04-2025").is_err());
        assert!(parse_date("invalid").is_err());
    }

    #[test]
    fn test_parse_date_invalid_date() {
        assert!(parse_date("2025-13-01").is_err()); // 无效月份
        assert!(parse_date("2025-04-32").is_err()); // 无效日期
    }
}
