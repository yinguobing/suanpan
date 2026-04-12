use chrono::Local;
use clap::Args;
use comfy_table::Table;

use crate::db::surreal::Database;
use crate::error::Result;

/// 输出格式
#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    /// 表格格式
    #[default]
    Table,
    /// CSV 格式
    Csv,
}

/// 将 SurrealDB Datetime 格式化为本地时间字符串（完整格式，用于表格显示）
fn format_datetime(dt: &surrealdb::Datetime) -> String {
    let sql_dt: surrealdb::sql::Datetime = dt.to_owned().into_inner();
    let utc_dt: chrono::DateTime<chrono::Utc> = sql_dt.into();
    let local_dt: chrono::DateTime<Local> = utc_dt.into();
    local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 将 SurrealDB Datetime 格式化为 ISO 8601 格式（用于 CSV 导出）
fn format_datetime_iso(dt: &surrealdb::Datetime) -> String {
    let sql_dt: surrealdb::sql::Datetime = dt.to_owned().into_inner();
    let utc_dt: chrono::DateTime<chrono::Utc> = sql_dt.into();
    // 格式化为带时区的 ISO 8601 格式，例如 2026-04-09T11:25:00+08:00
    utc_dt.with_timezone(&Local).to_rfc3339()
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

    /// 按账户筛选（账户ID或名称）
    #[arg(short, long)]
    pub account: Option<String>,

    /// 模糊搜索描述和备注
    #[arg(short, long)]
    pub search: Option<String>,

    /// 最小金额
    #[arg(long)]
    pub min_amount: Option<rust_decimal::Decimal>,

    /// 最大金额
    #[arg(long)]
    pub max_amount: Option<rust_decimal::Decimal>,

    /// 显示 ID 而非名称
    #[arg(long)]
    pub show_ids: bool,

    /// 输出格式 (table 或 csv)
    #[arg(short, long, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// 导出到文件（支持 .csv 或 .txt）
    #[arg(short, long)]
    pub output: Option<String>,
}

pub async fn execute(db: &Database, args: ListArgs) -> Result<()> {
    // 使用组合查询
    let transactions = db
        .query_transactions(
            args.from.as_deref(),
            args.to.as_deref(),
            args.category.as_deref(),
            args.tx_type.as_deref(),
            args.account.as_deref(),
            args.search.as_deref(),
            args.min_amount,
            args.max_amount,
            Some(args.limit),
        )
        .await?;

    if transactions.is_empty() {
        println!("暂无交易记录");
        return Ok(());
    }

    // 构建账户和分类的名称映射
    let accounts = db.list_accounts().await?;
    let account_map: std::collections::HashMap<_, _> = accounts
        .into_iter()
        .map(|a| (a.id, a.name))
        .collect();
    
    let categories = db.list_categories().await?;
    let category_map: std::collections::HashMap<_, _> = categories
        .into_iter()
        .map(|c| (c.id, c.name))
        .collect();

    // 如果有输出文件，先写入文件
    if let Some(output_path) = args.output {
        let content = generate_output(&transactions, &account_map, &category_map, args.show_ids, args.limit, &args.format);
        std::fs::write(&output_path, content)?;
        println!("[OK] 已导出 {} 条记录到: {}", transactions.len().min(args.limit), output_path);
        return Ok(());
    }

    // 根据格式选择输出方式
    match args.format {
        OutputFormat::Table => {
            output_table(&transactions, &account_map, &category_map, args.show_ids, args.limit).await?;
        }
        OutputFormat::Csv => {
            output_csv(&transactions, &account_map, &category_map, args.show_ids, args.limit).await?;
        }
    }

    Ok(())
}

/// 以表格格式输出
async fn output_table(
    transactions: &[crate::models::transaction::Transaction],
    account_map: &std::collections::HashMap<String, String>,
    category_map: &std::collections::HashMap<String, String>,
    show_ids: bool,
    limit: usize,
) -> Result<()> {
    use comfy_table::CellAlignment;
    
    let mut table = Table::new();
    table.set_header(vec![
        "时间", "类型", "金额", "货币", "账户", "去向", "分类", "备注", "ID",
    ]);
    
    // 设置金额列右对齐
    if let Some(col) = table.column_mut(2) {
        col.set_cell_alignment(CellAlignment::Right);
    }

    // 收集所有金额并计算最大宽度
    let amounts: Vec<String> = transactions.iter().take(limit)
        .map(|tx| format!("{:.2}", tx.amount))
        .collect();
    let max_amount_len = amounts.iter().map(|s| s.len()).max().unwrap_or(0);
    
    for (idx, tx) in transactions.iter().take(limit).enumerate() {
        let time = format_datetime_iso(&tx.timestamp);
        let tx_type = format!("{}", tx.tx_type);
        // 固定2位小数，并填充空格以实现小数点对齐
        let amount = format!("{:>width$}", amounts[idx], width = max_amount_len);
        let currency = &tx.currency;
        
        // 根据 show_ids 参数决定显示名称还是 ID
        let account_from = if show_ids {
            tx.account_from_id.clone()
        } else {
            account_map
                .get(&tx.account_from_id)
                .cloned()
                .unwrap_or_else(|| tx.account_from_id.clone())
        };
        
        let account_to = tx.account_to_id.as_deref().map(|id| {
            if show_ids {
                id.to_string()
            } else {
                account_map.get(id).cloned().unwrap_or_else(|| id.to_string())
            }
        });
        
        let category = if show_ids {
            tx.category_id.clone()
        } else {
            category_map
                .get(&tx.category_id)
                .cloned()
                .unwrap_or_else(|| tx.category_id.clone())
        };
        
        let description = tx.description.as_deref().unwrap_or("-");
        let short_id = format_short_id(&tx.id);

        table.add_row(vec![
            &time,
            &tx_type,
            &amount,
            currency,
            &account_from,
            account_to.as_deref().unwrap_or("-"),
            &category,
            description,
            &short_id,
        ]);
    }

    println!("{}", table);
    println!("共 {} 条记录", transactions.len().min(limit));
    
    Ok(())
}

/// 以 CSV 格式输出
async fn output_csv(
    transactions: &[crate::models::transaction::Transaction],
    account_map: &std::collections::HashMap<String, String>,
    category_map: &std::collections::HashMap<String, String>,
    show_ids: bool,
    limit: usize,
) -> Result<()> {
    // CSV 头部
    println!("时间,类型,金额,货币,账户,去向,分类,备注,ID");

    for tx in transactions.iter().take(limit) {
        let time = format_datetime(&tx.timestamp);
        let tx_type = format!("{}", tx.tx_type);
        let amount = tx.amount.to_string();
        let currency = &tx.currency;
        
        // 根据 show_ids 参数决定显示名称还是 ID
        let account_from = if show_ids {
            tx.account_from_id.clone()
        } else {
            account_map
                .get(&tx.account_from_id)
                .cloned()
                .unwrap_or_else(|| tx.account_from_id.clone())
        };
        
        let account_to = tx.account_to_id.as_deref().map(|id| {
            if show_ids {
                id.to_string()
            } else {
                account_map.get(id).cloned().unwrap_or_else(|| id.to_string())
            }
        }).unwrap_or_else(|| "".to_string());
        
        let category = if show_ids {
            tx.category_id.clone()
        } else {
            category_map
                .get(&tx.category_id)
                .cloned()
                .unwrap_or_else(|| tx.category_id.clone())
        };
        
        let description = tx.description.clone().unwrap_or_default();
        let short_id = format_short_id(&tx.id);

        // 转义包含逗号或引号的字段
        let description_escaped = escape_csv_field(&description);
        
        println!(
            "{},{},{},{},{},{},{},{},{}",
            time,
            tx_type,
            amount,
            currency,
            escape_csv_field(&account_from),
            escape_csv_field(&account_to),
            escape_csv_field(&category),
            description_escaped,
            short_id
        );
    }

    Ok(())
}

/// 生成输出内容（用于文件导出）
fn generate_output(
    transactions: &[crate::models::transaction::Transaction],
    account_map: &std::collections::HashMap<String, String>,
    category_map: &std::collections::HashMap<String, String>,
    show_ids: bool,
    limit: usize,
    format: &OutputFormat,
) -> String {
    let mut output = String::new();

    match format {
        OutputFormat::Table => {
            // 表格头部
            output.push_str(&format!("{:<25} {:<8} {:<12} {:<6} {:<12} {:<12} {:<16} {:<30} {:<12}\n",
                "时间", "类型", "金额", "货币", "账户", "去向", "分类", "备注", "ID"));
            output.push_str(&"-".repeat(146));
            output.push('\n');

            for tx in transactions.iter().take(limit) {
                let time = format_datetime(&tx.timestamp);
                let tx_type = format!("{}", tx.tx_type);
                let amount = tx.amount.to_string();
                let currency = &tx.currency;
                
                let account_from = if show_ids {
                    tx.account_from_id.clone()
                } else {
                    account_map
                        .get(&tx.account_from_id)
                        .cloned()
                        .unwrap_or_else(|| tx.account_from_id.clone())
                };
                
                let account_to = tx.account_to_id.as_deref().map(|id| {
                    if show_ids {
                        id.to_string()
                    } else {
                        account_map.get(id).cloned().unwrap_or_else(|| id.to_string())
                    }
                }).unwrap_or_else(|| "-".to_string());
                
                let category = if show_ids {
                    tx.category_id.clone()
                } else {
                    category_map
                        .get(&tx.category_id)
                        .cloned()
                        .unwrap_or_else(|| tx.category_id.clone())
                };
                
                let description = tx.description.as_deref().unwrap_or("-");
                let short_id = format_short_id(&tx.id);

                // 安全截断（处理多字节字符）
                let safe_truncate = |s: &str, max_len: usize| -> String {
                    if s.chars().count() <= max_len {
                        format!("{:width$}", s, width = max_len)
                    } else {
                        s.chars().take(max_len).collect::<String>()
                    }
                };
                
                output.push_str(&format!("{:<25} {:<8} {:<12} {:<6} {:<12} {:<12} {:<16} {:<30} {:<12}\n",
                    time, tx_type, amount, currency, 
                    safe_truncate(&account_from, 12),
                    safe_truncate(&account_to, 12),
                    safe_truncate(&category, 16),
                    safe_truncate(description, 30),
                    short_id
                ));
            }
        }
        OutputFormat::Csv => {
            // CSV 头部
            output.push_str("时间,类型,金额,货币,账户,去向,分类,备注,ID\n");

            for tx in transactions.iter().take(limit) {
                let time = format_datetime(&tx.timestamp);
                let tx_type = format!("{}", tx.tx_type);
                let amount = tx.amount.to_string();
                let currency = &tx.currency;
                
                let account_from = if show_ids {
                    tx.account_from_id.clone()
                } else {
                    account_map
                        .get(&tx.account_from_id)
                        .cloned()
                        .unwrap_or_else(|| tx.account_from_id.clone())
                };
                
                let account_to = tx.account_to_id.as_deref().map(|id| {
                    if show_ids {
                        id.to_string()
                    } else {
                        account_map.get(id).cloned().unwrap_or_else(|| id.to_string())
                    }
                }).unwrap_or_default();
                
                let category = if show_ids {
                    tx.category_id.clone()
                } else {
                    category_map
                        .get(&tx.category_id)
                        .cloned()
                        .unwrap_or_else(|| tx.category_id.clone())
                };
                
                let description = tx.description.clone().unwrap_or_default();
                let short_id = format_short_id(&tx.id);

                output.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{}\n",
                    time,
                    tx_type,
                    amount,
                    currency,
                    escape_csv_field(&account_from),
                    escape_csv_field(&account_to),
                    escape_csv_field(&category),
                    escape_csv_field(&description),
                    short_id
                ));
            }
        }
    }

    output
}

/// 转义 CSV 字段（处理包含逗号、引号或换行符的情况）
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        field.to_string()
    }
}


