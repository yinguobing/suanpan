use calamine::{open_workbook, Data, DataType, Reader, Xls, Xlsx};
use chrono::Local;
use clap::Args;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::path::Path;

use crate::db::surreal::Database;
use crate::error::Result;
use crate::models::transaction::Transaction;
use crate::models::types::{TxSource, TxType};

/// 数据来源类型
#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum ImportSource {
    /// 随手记
    #[default]
    Suishouji,
    /// 支付宝
    Alipay,
    /// 微信支付
    Wechat,
    /// 通用 CSV
    Csv,
}

/// 导入交易记录
#[derive(Args)]
pub struct ImportArgs {
    /// 导入文件路径（支持 .xls, .xlsx, .csv）
    pub file: String,

    /// 数据来源类型
    #[arg(short, long, value_enum, default_value = "suishouji")]
    pub source: ImportSource,

    /// 指定要导入的 sheet 名称（XLS/XLSX 专用，不指定则导入所有 sheet）
    #[arg(long)]
    pub sheet: Option<String>,

    /// 预览模式（不实际导入，只显示识别结果）
    #[arg(long)]
    pub dry_run: bool,

    /// 跳过重复检测
    #[arg(long)]
    pub skip_dedup: bool,
}

/// 解析后的原始交易数据
#[derive(Debug)]
struct ParsedTransaction {
    timestamp: chrono::DateTime<chrono::Utc>,
    amount: Decimal,
    tx_type: TxType,
    account_from: String,
    account_to: Option<String>,
    category: String,
    description: Option<String>,
    currency: String,
}

pub async fn execute(db: &Database, args: ImportArgs) -> Result<()> {
    let path = Path::new(&args.file);
    
    if !path.exists() {
        return Err(crate::error::FinanceError::Parse(
            format!("文件不存在: {}", args.file)
        ));
    }

    // 根据文件扩展名和来源类型选择解析器
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    
    let transactions = match ext.as_str() {
        "xls" | "xlsx" => parse_excel(path, &args).await?,
        "csv" => parse_csv(path, &args).await?,
        _ => {
            // 根据来源类型推断格式
            match args.source {
                ImportSource::Suishouji | ImportSource::Alipay | ImportSource::Wechat => {
                    // 尝试作为 Excel 解析
                    parse_excel(path, &args).await?
                }
                ImportSource::Csv => parse_csv(path, &args).await?,
            }
        }
    };

    if transactions.is_empty() {
        println!("未识别到任何交易记录");
        return Ok(());
    }

    // 显示预览
    println!("\n📋 识别到 {} 条交易记录\n", transactions.len());
    
    if args.dry_run {
        // 预览模式：显示前 10 条
        println!("【预览模式 - 前 10 条】");
        for (i, tx) in transactions.iter().take(10).enumerate() {
            println!(
                "{:2}. {} | {:?} | ¥{} | {} -> {} | {} | {}",
                i + 1,
                tx.timestamp.with_timezone(&Local).format("%Y-%m-%d %H:%M"),
                tx.tx_type,
                tx.amount,
                tx.account_from,
                tx.account_to.as_deref().unwrap_or("-"),
                tx.category,
                tx.description.as_deref().unwrap_or("")
            );
        }
        if transactions.len() > 10 {
            println!("... 还有 {} 条", transactions.len() - 10);
        }
        println!("\n取消 --dry-run 参数以实际导入");
        return Ok(());
    }

    // 检查重复（如果未跳过）
    let mut imported = 0;
    let mut skipped = 0;

    if !args.skip_dedup {
        let existing = db.list_transactions(10000).await?;
        let existing_set: std::collections::HashSet<_> = existing
            .iter()
            .map(|tx| {
                format!(
                    "{}-{}-{}-{}",
                    tx.timestamp.to_string(),
                    tx.amount,
                    tx.account_from_id,
                    tx.description.as_deref().unwrap_or("")
                )
            })
            .collect();

        for tx in transactions {
            let key = format!(
                "{}-{}-{}-{}",
                tx.timestamp.to_string(),
                tx.amount,
                tx.account_from,
                tx.description.as_deref().unwrap_or("")
            );

            if existing_set.contains(&key) {
                skipped += 1;
                continue;
            }

            import_single_transaction(db, tx).await?;
            imported += 1;
        }
    } else {
        // 不检查重复，直接导入
        for tx in transactions {
            import_single_transaction(db, tx).await?;
            imported += 1;
        }
    }

    println!("✅ 导入完成: {} 条成功, {} 条跳过（重复）", imported, skipped);

    Ok(())
}

async fn import_single_transaction(db: &Database, tx: ParsedTransaction) -> Result<()> {
    // 查找或创建账户、分类
    let account_from = db.find_or_create_account_by_name(&tx.account_from).await?;
    let account_to_id = if let Some(ref to) = tx.account_to {
        let acc = db.find_or_create_account_by_name(to).await?;
        Some(acc.id)
    } else {
        None
    };
    let category = db.find_or_create_category_by_path(&tx.category).await?;

    let transaction = Transaction::new(
        tx.amount,
        tx.currency,
        tx.tx_type,
        account_from.id,
        account_to_id,
        category.id,
        tx.description,
    )
    .with_timestamp(tx.timestamp)
    .with_source(TxSource::CsvImport);

    db.create_transaction(transaction).await?;
    
    Ok(())
}

/// 解析 Excel 文件
async fn parse_excel(path: &Path, args: &ImportArgs) -> Result<Vec<ParsedTransaction>> {
    let mut result = Vec::new();

    // 尝试作为 XLSX 打开
    if let Ok(mut workbook) = open_workbook::<Xlsx<_>, _>(path) {
        let sheet_names = workbook.sheet_names().to_vec();
        for sheet_name in &sheet_names {
            if let Some(ref target) = args.sheet {
                if sheet_name != target { continue; }
            }
            if let Ok(range) = workbook.worksheet_range(sheet_name) {
                println!("📄 正在解析 sheet: {}", sheet_name);
                let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
                if !rows.is_empty() {
                    let txs = parse_sheet_rows(&rows, args)?;
                    result.extend(txs);
                }
            }
        }
    } else if let Ok(mut workbook) = open_workbook::<Xls<_>, _>(path) {
        let sheet_names = workbook.sheet_names().to_vec();
        for sheet_name in &sheet_names {
            if let Some(ref target) = args.sheet {
                if sheet_name != target { continue; }
            }
            if let Ok(range) = workbook.worksheet_range(sheet_name) {
                println!("📄 正在解析 sheet: {}", sheet_name);
                let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
                if !rows.is_empty() {
                    let txs = parse_sheet_rows(&rows, args)?;
                    result.extend(txs);
                }
            }
        }
    } else {
        return Err(crate::error::FinanceError::Parse("无法打开 Excel 文件".to_string()));
    }

    Ok(result)
}

/// 解析 sheet 行数据
fn parse_sheet_rows(rows: &[Vec<Data>], args: &ImportArgs) -> Result<Vec<ParsedTransaction>> {
    match args.source {
        ImportSource::Suishouji => parse_suishouji_sheet(rows),
        ImportSource::Alipay => parse_alipay_sheet(rows),
        ImportSource::Wechat => parse_wechat_sheet(rows),
        ImportSource::Csv => parse_generic_sheet(rows),
    }
}

/// 解析 CSV 文件
async fn parse_csv(path: &Path, args: &ImportArgs) -> Result<Vec<ParsedTransaction>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| crate::error::FinanceError::Parse(format!("读取 CSV 失败: {}", e)))?;

    // 简单 CSV 解析（按行分割）
    let lines: Vec<_> = content.lines().collect();
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    // 转换为类似 Excel 的格式
    let rows: Vec<Vec<Data>> = lines
        .iter()
        .map(|line| {
            line.split(',')
                .map(|cell| Data::String(cell.to_string()))
                .collect()
        })
        .collect();

    // 根据来源类型解析
    match args.source {
        ImportSource::Suishouji => parse_suishouji_sheet(&rows),
        ImportSource::Alipay => parse_alipay_sheet(&rows),
        ImportSource::Wechat => parse_wechat_sheet(&rows),
        ImportSource::Csv => parse_generic_sheet(&rows),
    }
}

/// 解析随手记格式
fn parse_suishouji_sheet(rows: &[Vec<Data>]) -> Result<Vec<ParsedTransaction>> {
    let mut result = Vec::new();
    
    // 随手记通常的列：时间, 类型, 金额, 账户, 分类, 项目, 商家, 备注
    // 先找到表头行
    let mut header_idx: HashMap<String, usize> = HashMap::new();
    let mut header_row = 0;

    for (i, row) in rows.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            if let Data::String(s) = cell {
                let s = s.trim();
                if s == "时间" || s == "日期" {
                    header_idx.insert("time".to_string(), j);
                    header_row = i;
                } else if s == "类型" || s == "收支" {
                    header_idx.insert("type".to_string(), j);
                } else if s == "金额" {
                    header_idx.insert("amount".to_string(), j);
                } else if s == "账户" {
                    header_idx.insert("account".to_string(), j);
                } else if s == "分类" || s == "类别" {
                    header_idx.insert("category".to_string(), j);
                } else if s == "备注" || s == "说明" {
                    header_idx.insert("note".to_string(), j);
                } else if s == "项目" || s == "商家" {
                    header_idx.insert("project".to_string(), j);
                }
            }
        }
        if header_idx.contains_key("time") && header_idx.contains_key("amount") {
            break;
        }
    }

    // 解析数据行
    for row in &rows[header_row + 1..] {
        if row.is_empty() {
            continue;
        }

        let get_str = |key: &str| -> Option<String> {
            header_idx.get(key).and_then(|&idx| {
                row.get(idx).map(|cell| match cell {
                    Data::String(s) => s.trim().to_string(),
                    Data::Float(f) => f.to_string(),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(d) => d.to_string(),
                    _ => String::new(),
                })
            })
        };

        let time_str = match get_str("time") {
            Some(s) if !s.is_empty() => s,
            _ => continue,
        };

        let amount_str = match get_str("amount") {
            Some(s) if !s.is_empty() => s,
            _ => continue,
        };

        // 解析时间
        let timestamp = parse_datetime(&time_str)?;

        // 解析金额和类型
        let (amount, tx_type) = parse_amount_and_type(&amount_str, get_str("type").as_deref())?;

        // 账户
        let account = get_str("account").unwrap_or_else(|| "现金".to_string());

        // 分类
        let category = get_str("category").unwrap_or_else(|| "其他".to_string());

        // 备注（合并备注和项目/商家）
        let mut description = get_str("note").unwrap_or_default();
        if let Some(project) = get_str("project") {
            if !project.is_empty() && project != category {
                if !description.is_empty() {
                    description.push_str(" / ");
                }
                description.push_str(&project);
            }
        }

        result.push(ParsedTransaction {
            timestamp,
            amount,
            tx_type,
            account_from: account,
            account_to: None,
            category,
            description: if description.is_empty() { None } else { Some(description) },
            currency: "CNY".to_string(),
        });
    }

    Ok(result)
}

/// 解析支付宝格式（简化版）
fn parse_alipay_sheet(rows: &[Vec<Data>]) -> Result<Vec<ParsedTransaction>> {
    // TODO: 实现支付宝格式解析
    println!("⚠️ 支付宝格式解析暂未实现，使用通用格式");
    parse_generic_sheet(rows)
}

/// 解析微信格式（简化版）
fn parse_wechat_sheet(rows: &[Vec<Data>]) -> Result<Vec<ParsedTransaction>> {
    // TODO: 实现微信格式解析
    println!("⚠️ 微信格式解析暂未实现，使用通用格式");
    parse_generic_sheet(rows)
}

/// 通用格式解析
fn parse_generic_sheet(rows: &[Vec<Data>]) -> Result<Vec<ParsedTransaction>> {
    let mut result = Vec::new();

    // 尝试找到包含时间/金额/类型的列
    for row in rows.iter().skip(1) {
        if row.len() < 3 {
            continue;
        }

        // 简单启发式：找看起来像日期、金额、类型的列
        let mut time_col = None;
        let mut amount_col = None;
        let mut type_col = None;
        let mut desc_col = None;

        for (i, cell) in row.iter().enumerate() {
            let s = match cell {
                Data::String(s) => s.as_str(),
                _ => continue,
            };

            // 检测时间
            if time_col.is_none() && (s.contains('-') || s.contains('/')) && s.len() >= 8 {
                if parse_datetime(s).is_ok() {
                    time_col = Some(i);
                }
            }

            // 检测金额
            if amount_col.is_none() {
                if s.parse::<f64>().is_ok() || s.chars().any(|c| c.is_ascii_digit()) {
                    amount_col = Some(i);
                }
            }

            // 检测类型
            if type_col.is_none() {
                let lower = s.to_lowercase();
                if lower.contains("支出") || lower.contains("收入") || 
                   lower.contains("expense") || lower.contains("income") ||
                   lower.contains("transfer") {
                    type_col = Some(i);
                }
            }

            // 检测描述
            if desc_col.is_none() && s.len() > 2 && s.parse::<f64>().is_err() {
                desc_col = Some(i);
            }
        }

        // 如果找到必要字段，尝试解析
        if let (Some(t_idx), Some(a_idx)) = (time_col, amount_col) {
            if let (Some(time_str), Some(amount_str)) = (
                row.get(t_idx).and_then(|c| c.as_string()),
                row.get(a_idx).and_then(|c| c.as_string())
            ) {
                if let Ok(timestamp) = parse_datetime(&time_str) {
                    if let Ok((amount, tx_type)) = parse_amount_and_type(
                        &amount_str,
                        type_col.and_then(|i| row.get(i)).and_then(|c| c.as_string()).as_deref()
                    ) {
                        result.push(ParsedTransaction {
                            timestamp,
                            amount,
                            tx_type,
                            account_from: "未知账户".to_string(),
                            account_to: None,
                            category: "其他".to_string(),
                            description: desc_col.and_then(|i| row.get(i)).and_then(|c| {
                                c.as_string().map(|s| s.to_string())
                            }),
                            currency: "CNY".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(result)
}

/// 解析时间字符串
fn parse_datetime(s: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    // 尝试多种格式
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%Y年%m月%d日 %H:%M",
        "%Y年%m月%d日",
    ];

    for fmt in &formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(chrono::DateTime::from_naive_utc_and_offset(
                dt,
                chrono::Utc,
            ));
        }
    }

    // 尝试解析日期（没有时间）
    for fmt in &["%Y-%m-%d", "%Y/%m/%d", "%Y年%m月%d日"] {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
            let dt = d.and_hms_opt(0, 0, 0).unwrap();
            return Ok(chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc));
        }
    }

    // 如果是 Excel 序列号（数字）
    if let Ok(days) = s.parse::<f64>() {
        // Excel 日期从 1899-12-30 开始
        let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
        let days = days as i64;
        if let Some(date) = base.checked_add_signed(chrono::Duration::days(days)) {
            let dt = date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc));
        }
    }

    Err(crate::error::FinanceError::Parse(format!("无法解析时间: {}", s)))
}

/// 解析金额和类型
fn parse_amount_and_type(amount_str: &str, type_hint: Option<&str>) -> Result<(Decimal, TxType)> {
    // 清理金额字符串
    let cleaned = amount_str
        .replace("¥", "")
        .replace("￥", "")
        .replace(",", "")
        .replace(" ", "");

    let amount: Decimal = cleaned
        .parse()
        .map_err(|_| crate::error::FinanceError::Parse(format!("无法解析金额: {}", amount_str)))?;

    // 根据类型提示或金额符号判断
    let tx_type = match type_hint {
        Some(t) => {
            let lower = t.to_lowercase();
            if lower.contains("收入") || lower.contains("income") || lower.contains("收款") {
                TxType::Income
            } else if lower.contains("转账") || lower.contains("transfer") {
                TxType::Transfer
            } else {
                TxType::Expense
            }
        }
        None => {
            // 默认支出（随手记通常金额是正数，通过类型列区分收支）
            TxType::Expense
        }
    };

    Ok((amount.abs(), tx_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_datetime_various_formats() {
        // 标准格式
        assert!(parse_datetime("2025-04-09 12:30:00").is_ok());
        assert!(parse_datetime("2025-04-09").is_ok());
        assert!(parse_datetime("2025/04/09 12:30").is_ok());
        
        // 中文格式
        assert!(parse_datetime("2025年04月09日").is_ok());
        
        // 无效格式
        assert!(parse_datetime("invalid").is_err());
    }

    #[test]
    fn test_parse_amount_and_type() {
        // 普通金额
        let (amount, tx_type) = parse_amount_and_type("100.50", Some("支出")).unwrap();
        assert_eq!(amount, Decimal::new(10050, 2));
        assert!(matches!(tx_type, TxType::Expense));

        // 带货币符号
        let (amount, _) = parse_amount_and_type("¥1,234.56", None).unwrap();
        assert_eq!(amount, Decimal::new(123456, 2));

        // 收入
        let (_, tx_type) = parse_amount_and_type("5000", Some("收入")).unwrap();
        assert!(matches!(tx_type, TxType::Income));
    }
}
